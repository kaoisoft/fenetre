use fenetre::{
  button::Button,
  child_window::{
    ChildType,
    ChildWindow,
    Layout,
    LayoutArgs,
    LayoutFill,
    Orientation,
    UserEvent,
  },
  label::Label,
  line_edit::LineEdit,
  MainApp,
  MainAppSize,
  multi_line_edit::MultiLineEdit,
  row_layout::RowLayout,
  scroll_layout::ScrollLayout,
  window_utils::WindowUtils,
};

use winit::{
  event_loop::{
    EventLoop,
    EventLoopProxy,
  },
};

use tiny_skia::Color;

use uuid::Uuid;

use dirs;

use std::{
    cell::RefCell,
    fs::{self, File},
    io::Write,
    process,
    rc::Rc,
};
use fenetre::status_bar::StatusBar;

// Application-specific user events
static SAVE_FILE: u64 = 1;

// File being edited
static mut FILE_NAME: String = String::new();

struct Editor {
  event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
  main_app: RefCell<MainApp>,
  main_win_id: Uuid,                      // Main window's ID
  status_bar_rc: Rc<RefCell<StatusBar>>,  // Main window's status bar
  about_id: Uuid,                         // ID of the About dialog
  open_id: Uuid,                          // ID of the Open file dialog
  save_changes_dialog_id: Uuid,           // ID of the save file changes dialog
  message_dialog_id: Uuid,                // ID of the pop-up message dialog
  // Values being passed into and out of a dialog.
  dialog_values: Rc<RefCell<Vec<String>>>,
  editor_rc: Option<Rc<RefCell<MultiLineEdit>>>,
  editor_scroll_rc: Option<Rc<RefCell<ScrollLayout>>>,
  save_to_id: Uuid,                       // ID of the Save To dialog
  program_ending: bool,                   // Indicates that we are in the process of ending
}

impl Editor {
  
  fn new(event_loop_proxy: Rc<EventLoopProxy<UserEvent>>) -> Self {

    // Initialize the file name
    unsafe {
      FILE_NAME = "New".to_string();
    }

    let main_win_id = Uuid::new_v4();

    // Create the main window
    let main = MainApp::new(
      main_win_id,
      "Simple Text Editor",
      MainAppSize::Relative(0.25, 0.25, 0.5, 0.5),
      event_loop_proxy.clone(),
      move || {}
    );

    // Enable the status bar
    let status_bar_rc;
    {
      let mut main_ref = main.borrow_mut();
      main_ref.enable_statusbar(event_loop_proxy.clone());
      main_ref.set_status_message("Ready".to_string());
      status_bar_rc = main_ref.get_status_bar().unwrap().clone();
    }

    Self {
      event_loop_proxy,
      main_app: main,
      main_win_id,
      status_bar_rc,
      about_id: Uuid::new_v4(),
      open_id: Uuid::new_v4(),
      save_changes_dialog_id: Uuid::new_v4(),
      message_dialog_id: Uuid::new_v4(),
      dialog_values: Rc::new(RefCell::new(Vec::new())),
      editor_rc: None,
      editor_scroll_rc: None,
      save_to_id: Uuid::new_v4(),
      program_ending: false,
    }
  }

  fn create_editor_window(&mut self) -> Rc<RefCell<ScrollLayout>> {
    
    // Create the editor window
    let editor = MultiLineEdit::new(
      self.event_loop_proxy.clone(),
      self.main_win_id,
      2
    );
    let editor_rc = Rc::new(RefCell::new(editor));
    self.editor_rc = Some(editor_rc.clone());
    
    // Create the ScrollLayout into which the editor will be displayed
    let mut scroll_layout = ScrollLayout::new(
      self.event_loop_proxy.clone(),
      self.main_win_id,
    );
    
    // Add the editor to the scroll layout
    match scroll_layout.add_child(editor_rc, LayoutArgs::None) {
      
      Ok(_) => {},
      
      Err(err) => println!("Cannot add editor to the scroll layout: {}", err),
    }
    
    let scroll_rc = Rc::new(RefCell::new(scroll_layout));
    self.editor_scroll_rc = Some(scroll_rc.clone());
    
    scroll_rc.clone()
  }
  
  fn create_main_menu(&mut self) {

    // Enable the main menu
    let mut main_ref = self.main_app.borrow_mut();
    main_ref.enable_menubar(self.event_loop_proxy.clone());

    // Add the Open menu item
    let event_loop_proxy_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    let open_uuid = self.open_id;
    main_ref.add_menu_item(
      "Open".to_string(),
      move || {
        match event_loop_proxy_clone.send_event(UserEvent::CreateWindow(
          main_win_id,
          open_uuid,
          100.0,
          100.0,
          200.0,
          65.0,
          true,
        )) {
          Ok(_) => {},
          Err(err) => println!("Cannot send UserEvent event: {err}"),
        }
      }
    );

    // Add the Save menu item
    let event_loop_proxy_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    let save_to_id = self.save_to_id;
    let editor_rc_clone = self.editor_rc.clone();
    let message_dialog_id = self.message_dialog_id;
    let dialog_values_clone = self.dialog_values.clone();
    main_ref.add_menu_item(
      "Save".to_string(),
      move || {
        
        let filename;
        unsafe {
          filename = FILE_NAME.clone();
        }
        
        // If this is not a new file, save it.
        if filename != "New" {
          
          match &editor_rc_clone {
            
            Some(editor_rc) => {
              let mut editor_ref = editor_rc.borrow_mut();
              let contents = editor_ref.get_text().unwrap();

              // Write the editor's contents to the file
              write_to_file(filename.clone(), contents);
              
              editor_ref.set_modified(false);
            },

            None => {

              // Display an error message
              let mut msg_vec = dialog_values_clone.borrow_mut();
              msg_vec.clear();
              msg_vec.push("Error!".to_string());
              msg_vec.push(format!("Cannot retrieve contents of editor"));
              show_message(
                event_loop_proxy_clone.clone(),
                main_win_id,
                message_dialog_id
              );
            },
          }
        } else {
          
          // Prompt the user for the file to save to
          match event_loop_proxy_clone.send_event(UserEvent::CreateWindow(
            main_win_id,
            save_to_id,
            100.0,
            100.0,
            200.0,
            65.0,
            true,
          )) {
            Ok(_) => {},
            Err(err) => println!("Cannot send UserEvent event: {err}"),
          }
        }
      }
    );

    // Add the "About" menu item
    let event_loop_proxy_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    let about_uuid = self.about_id;
    main_ref.add_menu_item(
      "About".to_string(),
      move || {
        match event_loop_proxy_clone.send_event(UserEvent::CreateWindow(
          main_win_id,
          about_uuid,
          100.0,
          100.0,
          200.0,
          50.0,
          false
        )) {
          Ok(_) => {},
          Err(err) => println!("Cannot send UserEvent event: {err}"),
        }
      }
    );
  }

  fn create_main_window_contents(&mut self) {
    let main_win_id = self.main_win_id;

    // Create the main layout
    let mut main_layout = RowLayout::new(
      self.event_loop_proxy.clone(),
      main_win_id,
      Orientation::Vertical,
      5.0
    );

    // Create the multi-line editor
    let editor_scroll_rc = self.create_editor_window();
    match main_layout.add_layout(editor_scroll_rc.clone(), LayoutArgs::None) {
      
      Ok(_) => {},
      
      Err(err) => {
        println!("Cannot add editor window to main window: {err}");
        return;
      }
    }
    
    // Fill the main window with the editor's scroll layout
    let editor_scroll_ref = editor_scroll_rc.borrow();
    let editor_scroll_id = editor_scroll_ref.get_uuid();
    main_layout.set_fill(Box::new(LayoutFill::Single(editor_scroll_id)));

    // Set up the main window' contents.
    // Note: do not call redraw() on MainApp, it will be drawn when the event loop starts.
    let mut main_ref = self.main_app.borrow_mut();
    main_ref.set_contents(ChildType::Layout(Rc::new(RefCell::new(main_layout))));
  }

  // Defines the handler for caret moved events
  fn define_caret_moved_handler(&self) {

    let mut main_ref = self.main_app.borrow_mut();

    let editor_rc = match self.editor_rc {
      
      Some(ref editor) => editor,
      
      None => {
        return;
      }
    };
    let editor_id = editor_rc.borrow().get_uuid();
    
    let status_bar_rc = self.status_bar_rc.clone();
    main_ref.set_caret_moved_event_callback(Box::new(move |source_uuid, line, col| {

      // Is this event from the multi-line editor?
      if source_uuid == editor_id {

        // Update the status bar with the new location
        let msg = format!("Line: {}, col: {} | Ready", line + 1, col + 1);

        let mut status_bar_ref = status_bar_rc.borrow_mut();
        status_bar_ref.set_message(msg);
      }
    }));
  }

  // Defines a handler for user-defined events
  fn define_custom_events_handler(&mut self) {

    let mut main_ref = self.main_app.borrow_mut();

    let event_loop_proxy_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    let save_to_id = self.save_to_id;
    let program_ending = self.program_ending;
    let editor_rc_clone = self.editor_rc.clone();
    main_ref.set_user_defined_event_callback(Box::new(move|event_id, args| {
      
      if SAVE_FILE == event_id {

        // If this is a new file, prompt the user for the file into which to save.
        let filename;
        unsafe {
          filename = FILE_NAME.clone();
        }
        
        if "New" == filename {
          
          // Create the Save To dialog. Once the dialog's Ok button is clicked,
          // this custom event will be fired again.
          match event_loop_proxy_clone.send_event(UserEvent::CreateWindow(
            main_win_id,
            save_to_id,
            400.0,
            300.0,
            200.0,
            65.0,
            true
          )) {
            Ok(_) => {},
            Err(err) => {
              println!("Cannot send UserEvent event: {err}");
            },
          }
        } else {
          
          // Save the data to the file
          write_to_file(filename.clone(), args[0].clone());

          match &editor_rc_clone {
          
            Some(editor_rc) => {
              let mut editor_ref = editor_rc.borrow_mut();
              editor_ref.set_modified(false);
            },
            
            None => {},
          }
          
          // If we are in the process of ending the program, do so.
          if program_ending {
            process::exit(0);
          }
        }
      }
    }));
  }
      
    // Defines the end program handler
  fn define_end_program_handler(&mut self) {
      
    self.program_ending = true;
      
    let mut main_ref = self.main_app.borrow_mut();

    let editor_rc_option = self.editor_rc.clone();
    let event_loop_proxy_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    let save_changes_dialog_id = self.save_changes_dialog_id;
    main_ref.set_end_program_handler(Box::new(move || -> bool {

      // If the file being edited has unsaved changes, prompt the user to save them.
      match editor_rc_option {
        
        Some(ref editor_rc) => {
          
          let editor_ref = editor_rc.borrow();
          if editor_ref.is_modified() {

            // Create the save changes prompt dialog
            match event_loop_proxy_clone.send_event(UserEvent::CreateWindow(
              main_win_id,
              save_changes_dialog_id,
              400.0,
              300.0,
              200.0,
              65.0,
              true
            )) {
              Ok(_) => false,
              Err(err) => {
                println!("Cannot send UserEvent event: {err}");
                true
              },
            }
          } else {
            true
          }
        },
        
        None => true,
      }
    }));
  }

  // Define the window created handler
  fn define_window_created_handler(&self) {

    let mut main_ref = self.main_app.borrow_mut();

    let event_proxy_rc_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    let about_id_clone = self.about_id;
    let open_id_clone = self.open_id;
    let message_dialog_id = self.message_dialog_id;
    let editor_rc_clone = self.editor_rc.clone();
    let editor_scroll_rc_clone = self.editor_scroll_rc.clone();
    let save_changes_dialog_id = self.save_changes_dialog_id;
    let save_to_id = self.save_to_id;
    let dialog_values_clone = self.dialog_values.clone();
    let program_ending = self.program_ending;
    let home_dir = self.get_home_dir();
    main_ref.set_window_created_callback(Box::new(move |popup_rc| {

      // Which window was created?
      let mut popup = popup_rc.borrow_mut();
      let window_uuid = popup.get_uuid();
      if window_uuid == about_id_clone {        // About dialog
        popup.set_title("About Simple Text Editor");

        let popup_id = popup.get_uuid();

        // Create the layout that will contain the PopUp's contents
        let mut v_layout = RowLayout::new(
          event_proxy_rc_clone.clone(),
          popup_id,
          Orientation::Vertical,
          5.0
        );

        // Create the message
        let label = Label::new(
          event_proxy_rc_clone.clone(),
          popup_id,
          "Simple Text Editor".to_string(),
          Color::BLACK,
          Color::WHITE
        );
        let label_rc = Rc::new(RefCell::new(label));
        match v_layout.add_child(label_rc, LayoutArgs::None) {
          Ok(_) => {},
          Err(err) => println!("Failed to add message to About box: {err}"),
        }

        // Create the version
        let version = Label::new(
          event_proxy_rc_clone.clone(),
          popup_id,
          "Version 1.0".to_string(),
          Color::BLACK,
          Color::WHITE
        );
        let version_rc = Rc::new(RefCell::new(version));
        match v_layout.add_child(version_rc, LayoutArgs::None) {
          Ok(_) => {},
          Err(err) => println!("Failed to add version to About box: {err}"),
        }

        // Set the contents
        let v_layout_rc = Rc::new(RefCell::new(v_layout));
        popup.set_contents(ChildType::Layout(v_layout_rc.clone()));
      } else if window_uuid == open_id_clone {            // Open file dialog
        popup.set_title("Open File");

        // Create the layout that will contain the PopUp's contents
        let mut v_layout = RowLayout::new(
          event_proxy_rc_clone.clone(),
          main_win_id,
          Orientation::Vertical,
          5.0
        );

        // Add a label
        let label = Label::new(
          event_proxy_rc_clone.clone(),
          window_uuid,
          "File to edit:".to_string(),
          Color::BLACK,
          Color::WHITE
        );
        match v_layout.add_child(Rc::new(RefCell::new(label)), LayoutArgs::None) {
          Ok(_) => {},

          Err(err) => {
            println!("Could not add file entry field's label: {err}");
          },
        }

        // Add a line edit for the path name of the file
        let path_field = LineEdit::new(
          event_proxy_rc_clone.clone(),
          window_uuid,
          home_dir.clone()
        );
        let path_field_rc = Rc::new(RefCell::new(path_field));
        match v_layout.add_child(path_field_rc.clone(), LayoutArgs::None) {
          Ok(_) => {},

          Err(err) => {
            println!("Could not add file path entry field: {err}");
          },
        }

        // Add the buttons
        let btn_bg_color = Color::from_rgba8(31, 191, 191, 255);
        let mut btn_layout = RowLayout::new(
          event_proxy_rc_clone.clone(),
          main_win_id,
          Orientation::Horizontal,
          5.0
        );
        let event_proxy_clone_clone = event_proxy_rc_clone.clone();
        let popup_window_id = popup.get_window_id();
        let path_field_rc_clone = path_field_rc.clone();
        let editor_rc_clone_clone = editor_rc_clone.clone();
        let editor_scroll_rc_clone_clone = editor_scroll_rc_clone.clone();
        let ok_btn = Button::new(
          event_proxy_rc_clone.clone(),
          window_uuid,
          Some("Ok".to_string()),
          None,
          None,
          btn_bg_color,
          move || {

            // Close the pop-up
            WindowUtils::fire_user_event(
              event_proxy_clone_clone.clone(),
              UserEvent::ClosePopUp(main_win_id, popup_window_id)
            );

            // Read in the file
            let path_field_ref = path_field_rc_clone.borrow();
            let contents = match path_field_ref.get_text() {
              Some(path) => {

                // Save the file's name
                unsafe {
                  FILE_NAME = path.clone();
                }
                
                read_file(path)
              },

              None => "".to_string(),
            };

            // Populate the editor with the file's contents
            match &editor_rc_clone_clone {
              Some(ref editor_rc) => {
                let mut editor_ref = editor_rc.borrow_mut();
                editor_ref.set_text(contents);
              },

              None => {},
            }

            // Redraw the ScrollLayout
            match &editor_scroll_rc_clone_clone {
              Some(scroll_rc) => {
                let mut layout_ref = scroll_rc.borrow_mut();
                let (x, y) = layout_ref.get_location();
                let (width, height) = layout_ref.get_size();
                layout_ref.layout(x, y, width, height);
              },

              None => {},
            }
          }
        );
        match btn_layout.add_child(Rc::new(RefCell::new(ok_btn)), LayoutArgs::None) {
          Ok(_) => {},

          Err(err) => {
            println!("Could not add OK button: {err}");
          },
        }
        let event_loop_proxy_clone_clone = event_proxy_rc_clone.clone();
        let cancel_btn = Button::new(
          event_loop_proxy_clone_clone.clone(),
          window_uuid,
          Some("Cancel".to_string()),
          None,
          None,
          btn_bg_color,
          move || {

            // Close the pop-up
            WindowUtils::fire_user_event(
              event_loop_proxy_clone_clone.clone(),
              UserEvent::ClosePopUp(main_win_id, popup_window_id)
            );
          }
        );
        match btn_layout.add_child(Rc::new(RefCell::new(cancel_btn)), LayoutArgs::None) {
          Ok(_) => {},

          Err(err) => {
            println!("Could not add Cancel button: {err}");
          },
        }
        match v_layout.add_layout(Rc::new(RefCell::new(btn_layout)), LayoutArgs::None) {
          Ok(_) => {},

          Err(err) => {
            println!("Could not add button layout: {err}");
          },
        }

        // Set the contents
        let v_layout_rc = Rc::new(RefCell::new(v_layout));
        popup.set_contents(ChildType::Layout(v_layout_rc.clone()));
      } else if window_uuid == save_changes_dialog_id {   // Save changes prompt

        popup.set_title("File has unsaved changes");

        let popup_id = popup.get_uuid();

        // Create the layout that will contain the PopUp's contents
        let mut v_layout = RowLayout::new(
          event_proxy_rc_clone.clone(),
          popup_id,
          Orientation::Vertical,
          5.0
        );

        // Create the prompt
        let label = Label::new(
          event_proxy_rc_clone.clone(),
          popup_id,
          "Save changes before exiting?".to_string(),
          Color::BLACK,
          Color::WHITE
        );
        let label_rc = Rc::new(RefCell::new(label));
        match v_layout.add_child(label_rc, LayoutArgs::None) {
          Ok(_) => {},
          Err(err) => println!("Failed to add prompt to dialog: {err}"),
        }

        // Create the layout that will contain the buttons
        let mut btn_layout = RowLayout::new(
          event_proxy_rc_clone.clone(),
          popup_id,
          Orientation::Horizontal,
          5.0
        );

        // Create the Yes button
        let event_proxy_rc_clone_clone = event_proxy_rc_clone.clone();
        let popup_window_id = popup.get_window_id();
        let editor_rc_clone_clone = editor_rc_clone.clone();
        let dialog_values_clone_clone = dialog_values_clone.clone();
        let ok_btn = Button::new(
          event_proxy_rc_clone.clone(),
          popup_id,
          Some("Yes".to_string()),
          None,
          None,
          Color::from_rgba8(0, 64, 255, 255),
          move || {

            // Save the editor's contents to disk
            match &editor_rc_clone_clone {
              
              Some(editor_rc) => {
              
                // Fire the custom event to save the editor's contents
                let editor_ref = editor_rc.borrow();
                match editor_ref.get_text() {
                  
                  Some(data) => {

                    // Build the event's arguments
                    let mut args: Vec<String> = Vec::new();
                    args.push(data);

                    // Fire the custom event
                    WindowUtils::fire_user_event(
                      event_proxy_rc_clone_clone.clone(),
                      UserEvent::UserDefined(
                        main_win_id,
                        SAVE_FILE,
                        args
                      )
                    );
                  },
                  
                  None => {

                    // Display an error message
                    let mut msg_vec = dialog_values_clone_clone.borrow_mut();
                    msg_vec.clear();
                    msg_vec.push("Error!".to_string());
                    msg_vec.push(format!("Cannot save contents to file"));
                    show_message(
                      event_proxy_rc_clone_clone.clone(),
                      main_win_id,
                      message_dialog_id
                    );

                    // End the program
                    process::exit(0);
                  },
                };
              },
              
              None => {

                // End the program
                process::exit(0);
              },
            }
            
            // Close the dialog
            WindowUtils::fire_user_event(
              event_proxy_rc_clone_clone.clone(),
              UserEvent::ClosePopUp(popup_id, popup_window_id)
            );
          }
        );

        // Create the No button
        let event_proxy_rc_clone_clone = event_proxy_rc_clone.clone();
        let popup_window_id = popup.get_window_id();
        let cancel_btn = Button::new(
          event_proxy_rc_clone.clone(),
          popup_id,
          Some("No".to_string()),
          None,
          None,
          Color::from_rgba8(0, 64, 255, 255),
          move || {

            // Close the dialog
            WindowUtils::fire_user_event(
              event_proxy_rc_clone_clone.clone(),
              UserEvent::ClosePopUp(popup_id, popup_window_id)
            );
            
            // End the program
            process::exit(0);
          }
        );

        // Add the buttons
        match btn_layout.add_child(Rc::new(RefCell::new(ok_btn)), LayoutArgs::None) {
          Ok(_) => {},
          Err(err) => println!("Failed to add Yes button to layout: {err}"),
        }
        match btn_layout.add_child(Rc::new(RefCell::new(cancel_btn)), LayoutArgs::None) {
          Ok(_) => {},
          Err(err) => println!("Failed to add No button to layout: {err}"),
        }

        // Add the buttons' layout to the outer layout
        match v_layout.add_layout(
          Rc::new(RefCell::new(btn_layout)),
          LayoutArgs::None
        ) {
          Ok(_) => {},
          Err(err) => println!("Failed to add button layout to dialog: {err}"),
        }

        // Set the contents
        let v_layout_rc = Rc::new(RefCell::new(v_layout));
        popup.set_contents(ChildType::Layout(v_layout_rc.clone()));
      } else if window_uuid == save_to_id {            // Save To file dialog
        popup.set_title("Save To");

        // Create the layout that will contain the PopUp's contents
        let mut v_layout = RowLayout::new(
          event_proxy_rc_clone.clone(),
          main_win_id,
          Orientation::Vertical,
          5.0
        );

        // Add a label
        let label = Label::new(
          event_proxy_rc_clone.clone(),
          window_uuid,
          "File:".to_string(),
          Color::BLACK,
          Color::WHITE
        );
        match v_layout.add_child(Rc::new(RefCell::new(label)), LayoutArgs::None) {
          Ok(_) => {},

          Err(err) => {
            println!("Could not add label: {err}");
          },
        }

        // Add a line edit for the path name of the file
        let path_field = LineEdit::new(
          event_proxy_rc_clone.clone(),
          window_uuid,
          home_dir.clone()
        );
        let path_field_rc = Rc::new(RefCell::new(path_field));
        match v_layout.add_child(path_field_rc.clone(), LayoutArgs::None) {
          Ok(_) => {},

          Err(err) => {
            println!("Could not add file path entry field: {err}");
          },
        }

        // Add the buttons
        let btn_bg_color = Color::from_rgba8(31, 191, 191, 255);
        let mut btn_layout = RowLayout::new(
          event_proxy_rc_clone.clone(),
          main_win_id,
          Orientation::Horizontal,
          5.0
        );
        let event_proxy_clone_clone = event_proxy_rc_clone.clone();
        let popup_window_id = popup.get_window_id();
        let path_field_rc_clone = path_field_rc.clone();
        let editor_rc_clone_clone = editor_rc_clone.clone();
        let ok_btn = Button::new(
          event_proxy_rc_clone.clone(),
          window_uuid,
          Some("Ok".to_string()),
          None,
          None,
          btn_bg_color,
          move || {

            // Save the file's name
            let path_field_rc_clone_clone = path_field_rc_clone.clone();
            let path_field_ref = path_field_rc_clone_clone.borrow();
            let filename = path_field_ref.get_text().unwrap();
            unsafe {
              FILE_NAME = filename.clone();
            }
            
            // Close the pop-up
            WindowUtils::fire_user_event(
              event_proxy_clone_clone.clone(),
              UserEvent::ClosePopUp(main_win_id, popup_window_id)
            );

            // Get the editor's contents
            match editor_rc_clone_clone.clone() {
              
              Some(editor_rc) => {
            
                let mut editor_ref = editor_rc.borrow_mut();
                let contents = editor_ref.get_text().unwrap();

                // Write the editor's contents to the file
                write_to_file(filename.clone(), contents);
                editor_ref.set_modified(false);

                // If we are in the process of ending the program, do so.
                if program_ending {
                  process::exit(0);
                }
              },
              
              None => {

                // If we are in the process of ending the program, do so.
                if program_ending {
                  process::exit(0);
                }
              },
            }
          }
        );
        match btn_layout.add_child(Rc::new(RefCell::new(ok_btn)), LayoutArgs::None) {
          Ok(_) => {},

          Err(err) => {
            println!("Could not add OK button: {err}");
          },
        }
        let event_loop_proxy_clone_clone = event_proxy_rc_clone.clone();
        let cancel_btn = Button::new(
          event_loop_proxy_clone_clone.clone(),
          window_uuid,
          Some("Cancel".to_string()),
          None,
          None,
          btn_bg_color,
          move || {

            // Close the pop-up
            WindowUtils::fire_user_event(
              event_loop_proxy_clone_clone.clone(),
              UserEvent::ClosePopUp(main_win_id, popup_window_id)
            );

            // If we are in the process of ending the program, do so.
            if program_ending {
              process::exit(0);
            }
          }
        );
        match btn_layout.add_child(Rc::new(RefCell::new(cancel_btn)), LayoutArgs::None) {
          Ok(_) => {},

          Err(err) => {
            println!("Could not add Cancel button: {err}");
          },
        }
        match v_layout.add_layout(Rc::new(RefCell::new(btn_layout)), LayoutArgs::None) {
          Ok(_) => {},

          Err(err) => {
            println!("Could not add button layout: {err}");
          },
        }

        // Set the contents
        let v_layout_rc = Rc::new(RefCell::new(v_layout));
        popup.set_contents(ChildType::Layout(v_layout_rc.clone()));
      } else if window_uuid == message_dialog_id {  // Message dialog

        let dialog_values_ref = dialog_values_clone.borrow();

        popup.set_title(&dialog_values_ref[0].clone());

        let popup_id = popup.get_uuid();

        // Create the layout that will contain the PopUp's contents
        let mut v_layout = RowLayout::new(
          event_proxy_rc_clone.clone(),
          popup_id,
          Orientation::Vertical,
          5.0
        );

        // Create the message field
        let label = Label::new(
          event_proxy_rc_clone.clone(),
          popup_id,
          dialog_values_ref[1].clone(),
          Color::BLACK,
          Color::WHITE
        );
        let label_rc = Rc::new(RefCell::new(label));
        match v_layout.add_child(label_rc, LayoutArgs::None) {
          Ok(_) => {},
          Err(err) => println!("Failed to add message to About box: {err}"),
        }

        // Set the contents
        let v_layout_rc = Rc::new(RefCell::new(v_layout));
        popup.set_contents(ChildType::Layout(v_layout_rc.clone()));
      }
    }));
  }

  fn get_home_dir(&self) -> String {

    match dirs::home_dir() {
      Some(dir) => dir.to_str().unwrap().to_string(),
      None => "/".to_string(),
    }
  }

  fn run_event_loop(&mut self, event_loop: EventLoop<UserEvent>) {
    let mut main_ref = self.main_app.borrow_mut();
    main_ref.run_event_loop(event_loop);
  }
}

fn main() {

  // Create the event loop
  let event_loop = match EventLoop::<UserEvent>::with_user_event().build() {
    Ok(event_loop) => event_loop,
    Err(err) => {
      println!("Cannot create an event loop: {err}");
      process::exit(-1);
    },
  };
  let event_loop_proxy = Rc::new(event_loop.create_proxy());

  // Create the main application object
  let mut editor = Editor::new(event_loop_proxy.clone());

  // Create the main window's contents.
  // We need to create the contents before the main menu, because the 'Save'
  // menu item needs to access the MultiLineEdit.
  editor.create_main_window_contents();

  // Set up the menu
  editor.create_main_menu();

  // Define the event handlers
  editor.define_end_program_handler();
  editor.define_window_created_handler();
  editor.define_caret_moved_handler();
  editor.define_custom_events_handler();

  // Run the event loop
  editor.run_event_loop(event_loop);
}

fn read_file(file: String) -> String {

  match fs::read_to_string(file.clone()) {

    Ok(contents) => contents,

    Err(err) => {
      println!("Failed to read file {file}: {err}");
      "".to_string()
    },
  }
}

fn show_message(
  event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
  main_win_id: Uuid,
  message_dialog_id: Uuid
) {
  match event_loop_proxy.send_event(UserEvent::CreateWindow(
    main_win_id,
    message_dialog_id,
    100.0,
    100.0,
    200.0,
    30.0,
    false
  )) {
    Ok(_) => {},
    Err(err) => println!("Cannot send UserEvent event: {err}"),
  }
}

// Replaces the contents of a file
fn write_to_file(filename: String, contents: String) {

  // Create or truncate the file
  match File::create(filename.clone()) {

    Ok(mut file) => {

      // Save the data to the file
      match file.write_all(contents.as_bytes()) {

        Ok(_) => {},

        Err(err) => println!("Cannot write to file: {err}"),
      }
    },

    Err(err) => println!("Cannot open file: {err}"),
  }
}
