use fenetre::{
  button::Button,
  child_window::{
    ChildType,
    ChildWindow,
    ContextMenuItem,
    Layout,
    LayoutArgs,
    LayoutFill,
    Orientation,
    UserEvent,
  },
  image_view::{DisplayType, ImageView},
  label::Label,
  line_edit::LineEdit,
  list::{List, SelectionMode},
  MainApp,
  MainAppSize,
  row_layout::RowLayout,
  scroll_layout::ScrollLayout,
  status_bar::StatusBar,
  window_utils::WindowUtils,
};

use winit::{
  event_loop::{EventLoop, EventLoopProxy},
};

use tiny_skia::Color;

use uuid::Uuid;

use dirs;

use std::{
  cell::RefCell,
  fs,
  process,
  rc::Rc,
};

struct ImageViewer {
  event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
  main_app: RefCell<MainApp>,
  main_win_id: Uuid,                      // Main window's ID
  status_bar_rc: Rc<RefCell<StatusBar>>,  // Main window's status bar
  about_id: Uuid,                         // ID of the About dialog
  message_dialog_id: Uuid,                // ID of the pop-up message dialog
  // Values being passed into and out of a dialog.
  dialog_values: Rc<RefCell<Vec<String>>>,
  directory_listing_id: Uuid,                         // ID of the directory listing window
  directory_listing_rc: Option<Rc<RefCell<List>>>,    // directory listing window
  directory_listing_scroll_rc: Option<Rc<RefCell<ScrollLayout>>>, // scroller for directory listing
  directory_entry_rc: Option<Rc<RefCell<LineEdit>>>,  // directory path entry field
  image_view_rc: Option<Rc<RefCell<ImageView>>>,      // ImageView
  full_image_id: Uuid,                                // ID of the full image pop-up window
}

impl ImageViewer {

  fn new(event_loop_proxy: Rc<EventLoopProxy<UserEvent>>) -> Self {

    let main_win_id = Uuid::new_v4();

    // Create the main window
    let main = MainApp::new(
      main_win_id,
      "Image Viewer",
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
      message_dialog_id: Uuid::new_v4(),
      dialog_values: Rc::new(RefCell::new(Vec::new())),
      directory_listing_id: Uuid::new_v4(),
      directory_listing_rc: None,
      directory_listing_scroll_rc: None,
      directory_entry_rc: None,
      image_view_rc: None,
      full_image_id: Uuid::new_v4(),
    }
  }

  fn create_directory_listing_window(&mut self) -> Rc<RefCell<RowLayout>> {

    // Create a layout to hold the directory listing fields
    let mut directory_fields_layout = RowLayout::new(
      self.event_loop_proxy.clone(),
      self.main_win_id,
      Orientation::Vertical,
      5.0
    );

    // Create a List that will display the directory listing
    let mut list = List::new(self.event_loop_proxy.clone(), self.main_win_id);
    list.set_selection_mode(SelectionMode::Single);
    let list_uuid = list.get_uuid();
    self.directory_listing_id = list_uuid;
    let list_rc = Rc::new(RefCell::new(list));
    self.directory_listing_rc = Some(list_rc.clone());

    // Create a ScrollLayout to hold the list
    let mut list_scroll_layout = ScrollLayout::new(self.event_loop_proxy.clone(), self.main_win_id);
    let list_scroll_layout_uuid = list_scroll_layout.get_uuid();

    // Add the list to the scroll layout
    match list_scroll_layout.add_child(list_rc.clone(), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => {
        println!("Failed to add directory listing to scroll layout: {err}");
      },
    }

    // Set the starting directory
    let dir_path = format!("{}/Pictures", self.get_home_dir());

    // Create the layout for the entry fields
    let mut entry_fields_layout = RowLayout::new(
      self.event_loop_proxy.clone(),
      self.main_win_id,
      Orientation::Horizontal, 5.0
    );

    // Create the directory entry field
    let dir_entry = LineEdit::new(
      self.event_loop_proxy.clone(),
      self.main_win_id,
      dir_path.clone()
    );
    let dir_entry_uuid = dir_entry.get_uuid();
    let dir_entry_rc = Rc::new(RefCell::new(dir_entry));
    self.directory_entry_rc = Some(dir_entry_rc.clone());

    // Create an Up button to display the parent directory
    let dir_entry_rc_clone = dir_entry_rc.clone();
    let event_loop_proxy_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    let list_uuid_clone = list_uuid.clone();
    let up_btn = Button::new(
      event_loop_proxy_clone.clone(),
      main_win_id,
      Some("Up".to_string()),
      None,
      None,
      Color::WHITE,
      move || {
        let mut dir_entry_ref = dir_entry_rc_clone.borrow_mut();
        let new_dir = match dir_entry_ref.get_text() {
          Some(path) => {
            // Find the index of the last '/'
            let index = match path.rfind('/') {
              Some(index) => index,
              None => 0,
            };

            // Return the parent's path
            (&path[0..index]).to_string()
          },
          None => "/".to_string(),
        };

        // Update the entry field
        dir_entry_ref.set_text(new_dir.clone());

        // Update the list
        populate_directory_listing(
          main_win_id,
          list_uuid_clone,
          new_dir,
          event_loop_proxy_clone.clone()
        );
      }
    );

    // Add the Up button to its layout
    match entry_fields_layout.add_child(Rc::new(RefCell::new(up_btn)), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => {
        println!("Failed to add Up button: {err}");
      },
    }

    // Add the directory entry field to the layout
    match entry_fields_layout.add_child(dir_entry_rc.clone(), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => {
        println!("Failed to add directory entry field: {err}");
      },
    }
    entry_fields_layout.set_fill(Box::new(LayoutFill::Single(dir_entry_uuid)));

    // Create a Go button to process the directory entry field
    let dir_entry_rc_clone = dir_entry_rc.clone();
    let event_loop_proxy_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    let list_uuid_clone = list_uuid.clone();
    let go_btn = Button::new(
      event_loop_proxy_clone.clone(),
      main_win_id,
      Some("Go".to_string()),
      None,
      None,
      Color::WHITE,
      move || {
        let dir_entry = dir_entry_rc_clone.borrow();
        match dir_entry.get_text() {
          Some(path) => {
            populate_directory_listing(
              main_win_id,
              list_uuid_clone,
              path,
              event_loop_proxy_clone.clone()
            );
          },
          None => {},
        }
      }
    );

    // Add the Go button to the layout
    match entry_fields_layout.add_child(Rc::new(RefCell::new(go_btn)), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => {
        println!("Failed to add Go button: {err}");
      },
    }

    // Add the entry fields layout to the directory listing layout
    match directory_fields_layout.add_layout(Rc::new(RefCell::new(entry_fields_layout)), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => {
        println!("Failed to add directory listing: {err}");
      },
    }

    // Add the list layout to the directory listing layout
    let list_scroll_layout_rc = Rc::new(RefCell::new(list_scroll_layout));
    self.directory_listing_scroll_rc = Some(list_scroll_layout_rc.clone());
    match directory_fields_layout.add_layout(list_scroll_layout_rc.clone(), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => {
        println!("Failed to add directory listing: {err}");
      },
    }
    directory_fields_layout.set_fill(Box::new(LayoutFill::Single(list_scroll_layout_uuid)));

    Rc::new(RefCell::new(directory_fields_layout))
  }

  fn create_image_window(&mut self) -> Rc<RefCell<RowLayout>> {

    // Read in the data for the rotate buttons' images
    let rotate_left_image_data = include_bytes!("../resources/rotate_left.png").try_into().unwrap();
    let rotate_right_image_data = include_bytes!("../resources/rotate_right.png").try_into().unwrap();

    // Create an image view
    let mut image_view = ImageView::new(
      self.event_loop_proxy.clone(),
      self.main_win_id,
      DisplayType::ScaleSmaller
    );
    let image_view_uuid = image_view.get_uuid();

    // Set the ImageView's context menu
    let full_image_uuid = self.full_image_id;
    let event_loop_proxy_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    let context_menu_item = Box::new(ContextMenuItem {
      label: "Show full size".to_string(),
      callback: Box::new(move || {

        // Create a pop-up window in which the full sized image will be displayed
        WindowUtils::fire_user_event(
          event_loop_proxy_clone.clone(),
          UserEvent::CreateWindow(
            main_win_id,
            full_image_uuid,
            100.0,
            100.0,
            800.0,
            600.0,
            true
          )
        );
      })
    });
    image_view.add_context_menu_item(context_menu_item);

    // Create the rotate image buttons
    let image_view_rc = Rc::new(RefCell::new(image_view));
    self.image_view_rc = Some(image_view_rc.clone());
    let image_view_rc_clone_1 = image_view_rc.clone();
    let mut rotate_clockwise_btn = Button::new(
      self.event_loop_proxy.clone(),
      self.main_win_id,
      None,
      None,
      Some(rotate_right_image_data),
      Color::WHITE,
      move || {

        // Rotate the image 90 degrees clockwise
        let mut image_view_ref = image_view_rc_clone_1.borrow_mut();
        image_view_ref.rotate_clockwise();
      }
    );
    rotate_clockwise_btn.set_tooltip_text("Rotate image clockwise 90 degrees".to_string());
    let image_view_rc_clone_2 = image_view_rc.clone();
    let rotate_counter_clockwise_btn = Button::new(
      self.event_loop_proxy.clone(),
      self.main_win_id,
      None,
      None,
      Some(rotate_left_image_data),
      Color::WHITE,
      move || {

        // Rotate the image 90 degrees clockwise
        let mut image_view_ref = image_view_rc_clone_2.borrow_mut();
        image_view_ref.rotate_counter_clockwise();
      }
    );

    // Create a horizontal layout to hold the image manipulation buttons
    let mut image_btns_layout = RowLayout::new(
      self.event_loop_proxy.clone(),
      self.main_win_id,
      Orientation::Horizontal,
      5.0
    );

    // Add the image manipulation buttons to the layout
    match image_btns_layout.add_child(Rc::new(RefCell::new(rotate_counter_clockwise_btn)), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => {
        println!("Failed to add image rotate counter clockwise button: {err}");
      },
    }
    match image_btns_layout.add_child(Rc::new(RefCell::new(rotate_clockwise_btn)), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => {
        println!("Failed to add image rotate clockwise button: {err}");
      },
    }

    // Create a vertical layout to hold the image manipulation buttons and the image view
    let mut image_layout = RowLayout::new(
      self.event_loop_proxy.clone(),
      self.main_win_id,
      Orientation::Vertical,
      5.0
    );
    match image_layout.add_layout(Rc::new(RefCell::new(image_btns_layout)), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => {
        println!("Failed to add image manipulation buttons: {err}");
      },
    }
    match image_layout.add_child(image_view_rc.clone(), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => {
        println!("Failed to add image view: {err}");
      },
    }
    image_layout.set_fill(Box::new(LayoutFill::Single(image_view_uuid)));

    Rc::new(RefCell::new(image_layout))
  }

  fn create_main_menu(&mut self) {

    // Enable the main menu
    let mut main_ref = self.main_app.borrow_mut();
    main_ref.enable_menubar(self.event_loop_proxy.clone());

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
      Orientation::Horizontal,
      5.0
    );

    // Create the directory listing
    let dir_listing_rc = self.create_directory_listing_window();
    match main_layout.add_layout(dir_listing_rc.clone(), LayoutArgs::None) {

      Ok(_) => {},

      Err(err) => {
        println!("Cannot add directory listing window to main window: {err}");
        return;
      }
    }

    // Create the image display
    let image_scroll_rc = self.create_image_window();
    match main_layout.add_layout(image_scroll_rc.clone(), LayoutArgs::None) {

      Ok(_) => {},

      Err(err) => {
        println!("Cannot add image display window to main window: {err}");
        return;
      }
    }

    // Fill the main window with the image display's scroll layout
    let image_scroll_ref = image_scroll_rc.borrow();
    let image_scroll_id = image_scroll_ref.get_uuid();
    main_layout.set_fill(Box::new(LayoutFill::Single(image_scroll_id)));

    // Set up the main window' contents.
    // Note: do not call redraw() on MainApp, it will be drawn when the event loop starts.
    let mut main_ref = self.main_app.borrow_mut();
    main_ref.set_contents(ChildType::Layout(Rc::new(RefCell::new(main_layout))));
  }

  fn define_selection_changed_handler(&mut self) {

    let mut main_ref = self.main_app.borrow_mut();

    let dir_listing_clone = self.directory_listing_rc.as_ref().unwrap().clone();
    let dir_entry_rc_clone = self.directory_entry_rc.as_ref().unwrap().clone();
    let image_view_rc_clone = self.image_view_rc.as_ref().unwrap().clone();
    let list_id = self.directory_listing_id;
    let event_proxy_rc_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    main_ref.set_selection_changed_event_callback(Box::new(move |source_uuid| {

      // Is this event from the directory listing?
      let dir_listing = dir_listing_clone.borrow();
      if dir_listing.get_uuid() == source_uuid {

        // Get the directory path
        let mut dir_entry_ref = dir_entry_rc_clone.borrow_mut();
        let dir_path = match dir_entry_ref.get_text() {
          Some(path) => path,
          None => "".to_string(),
        };

        // Get the indices of the currently selected directory listing
        // items. There should only be one.
        let selected = dir_listing.get_selected_items();
        match selected.get(0) {
          Some(index) => {
            match dir_listing.get_item_by_index(*index) {
              Some(item) => {

                // Is the selection a directory?
                if item.starts_with("<") && item.ends_with(">") {

                  let full_path = format!("{dir_path}/{}",
                                          &item[1..(item.len() - 1)]);

                  // display the new directory's contents
                  dir_entry_ref.set_text(full_path.clone());
                  populate_directory_listing(
                    main_win_id,
                    list_id,
                    full_path,
                    event_proxy_rc_clone.clone()
                  );
                } else {    // The selection is a file

                  let full_path = format!("{dir_path}/{item}");

                  let mut image_view_ref = image_view_rc_clone.borrow_mut();
                  image_view_ref.load(&full_path);
                }
              },
              None => {},
            }
          },
          None => {},
        }
      }
    }));
  }

  fn define_set_list_event_handler(&mut self) {

    let mut main_ref = self.main_app.borrow_mut();

    let dir_listing_clone = self.directory_listing_rc.as_ref().unwrap().clone();
    let list_scroll_layout_rc_clone = self.directory_listing_scroll_rc.as_ref().unwrap().clone();
    main_ref.set_set_list_event_callback(Box::new(move |dest_uuid, data| {

      // Is the source of the event the directory listing?
      let mut is_listing = false;
      {
        let dir_listing = dir_listing_clone.borrow();
        if dir_listing.get_uuid() == dest_uuid {
          is_listing = true;
        }
      }

      if is_listing {

        {
          let mut dir_listing = dir_listing_clone.borrow_mut();

          // Set the listing contents
          let data_len = data.len();
          dir_listing.set_items(data);

          // Get the maximum number of visible items
          let max_visible = dir_listing.get_max_vertical_visible_items() as usize;

          // Update the vertical scroll range
          let max;
          if data_len > max_visible {
            max = data_len - 1 - max_visible;
          } else {
            max = 0;
          }
          dir_listing.set_y_scroll_min(0.0);
          dir_listing.set_y_scroll_max(max as f64);
          dir_listing.set_y_scroll(0.0);
        }
      }
    }));
  }

  fn define_window_created_handler(&mut self) {

    let mut main_ref = self.main_app.borrow_mut();

    let event_proxy_rc_clone = self.event_loop_proxy.clone();
    let main_win_id = self.main_win_id;
    let image_view_rc_clone_3 = self.image_view_rc.as_ref().unwrap().clone();
    let about_id = self.about_id;
    let full_image_id = self.full_image_id;
    main_ref.set_window_created_callback(Box::new(move |popup_rc| {

      // Which window was created?
      let mut popup = popup_rc.borrow_mut();
      let window_uuid = popup.get_uuid();
      if window_uuid == about_id {        // About dialog
        popup.set_title("About Image Viewer");

        // Create the layout that will contain the PopUp's contents
        let mut v_layout = RowLayout::new(
          event_proxy_rc_clone.clone(),
          main_win_id,
          Orientation::Vertical,
          5.0
        );

        // Create the message
        let label = Label::new(
          event_proxy_rc_clone.clone(),
          main_win_id,
          "Image Viewer Application".to_string(),
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
          main_win_id,
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
      } else if window_uuid == full_image_id {    // Full image view

        // Create the ScrollLayout in which to display the image
        let mut scroll_layout = ScrollLayout::new(
          event_proxy_rc_clone.clone(),
          popup.get_uuid()
        );
        let scroll_layout_rc = Rc::new(RefCell::new(scroll_layout));

        // Set the window's title to the path of the image
        let image_view_rc = image_view_rc_clone_3.borrow();
        match image_view_rc.get_image_path() {

          Some(path) => {

            // Set the title
            popup.set_title(path.as_str());

            let popup_uuid = popup.get_uuid();

            // Create a new ImageView that is not scaled
            let mut full_image_view = ImageView::new(
              event_proxy_rc_clone.clone(),
              popup_uuid,
              DisplayType::Actual
            );
            full_image_view.load(&path);
            let (image_width, image_height) = full_image_view.get_image_size();

            // Get the maximum number of visible pixels
            let max_horz_visible = full_image_view.get_max_horizontal_visible_items() as usize;
            let max_vert_visible = full_image_view.get_max_vertical_visible_items() as usize;

            // Update the horizontal scroll range
            let horz_max;
            if image_width as usize > max_horz_visible {
              horz_max = (image_width as usize) - 1 - max_horz_visible;
            } else {
              horz_max = 0;
            }
            full_image_view.set_x_scroll_min(0.0);
            full_image_view.set_x_scroll_max(horz_max as f64);
            full_image_view.set_x_scroll(0.0);

            // Update the vertical scroll range
            let vert_max;
            if image_height as usize > max_vert_visible {
              vert_max = (image_height as usize) - 1 - max_vert_visible;
            } else {
              vert_max = 0;
            }
            full_image_view.set_y_scroll_min(0.0);
            full_image_view.set_y_scroll_max(vert_max as f64);
            full_image_view.set_y_scroll(0.0);
            let full_image_view_rc = Rc::new(RefCell::new(full_image_view));

            // Add the image to the layout
            let scroll_layout_rc_clone = scroll_layout_rc.clone();
            let mut scroll_layout_ref = scroll_layout_rc_clone.borrow_mut();
            scroll_layout_ref.add_child(full_image_view_rc.clone(), LayoutArgs::None);

            // Set the scroll layout as the parent of the ImageView
            {
              let mut full_image_view_ref = full_image_view_rc.borrow_mut();
              full_image_view_ref.set_parent(Some(ChildType::Layout(scroll_layout_rc.clone())));
            }
          },

          None => popup.set_title("Unnamed Image"),
        }

        // Add the layout to the pop-up
        popup.set_contents(ChildType::Layout(scroll_layout_rc.clone()));
      }
    }));
  }

  fn get_directory(&self) -> String {
    
    match &self.directory_entry_rc {
      
      Some(dir_entry_rc) => {
        
        let dir_entry = dir_entry_rc.borrow();
        dir_entry.get_text().unwrap()
      },
      
      None => "".to_string(),
    }  
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
  let mut image_viewer = ImageViewer::new(event_loop_proxy.clone());

  // Create the main window's contents.
  // We need to create the contents before the main menu, because the 'Save'
  // menu item needs to access the MultiLineEdit.
  image_viewer.create_main_window_contents();

  // Set up the menu
  image_viewer.create_main_menu();

  // Define the event handlers
  image_viewer.define_window_created_handler();
  image_viewer.define_selection_changed_handler();
  image_viewer.define_set_list_event_handler();

  // Populate the directory listing
  populate_directory_listing(
    image_viewer.main_win_id,
    image_viewer.directory_listing_id,
    image_viewer.get_directory(),
    event_loop_proxy.clone()
  );

  // Run the event loop
  image_viewer.run_event_loop(event_loop);
}

fn populate_directory_listing(
      main_win_uuid: Uuid,
      list_uuid: Uuid,
      dir_path: String,
      event_loop_proxy_rc: Rc<EventLoopProxy<UserEvent>>
) {

  // Populate the list with the current directories contents
  let mut list: Vec<String> = Vec::new();
  match fs::read_dir(dir_path) {

    Ok(entries) => {

      // Separate the directories from the files
      let mut dirs: Vec<String> = Vec::new();
      let mut files: Vec<String> = Vec::new();
      for entry in entries {

        match entry {

          Ok(dir_entry) => {

            let path = dir_entry.path();
            let name = dir_entry.file_name().into_string().unwrap();
            if path.is_dir() {
              dirs.push(format!("<{name}>"));
            } else {
              files.push(name);
            }
          },

          Err(err) => println!("Could not retrieve a directory entry: {err}"),
        }
      }

      // Sort the lists
      dirs.sort();
      files.sort();

      // Add the entries to the List, directories first
      for dir in dirs {
        list.push(dir);
      }
      for file in files {
        list.push(file);
      }
    },

    Err(err) => println!("Cannot get directory contents: {err}"),
  }

  // Fire the event
  match event_loop_proxy_rc.send_event(UserEvent::SetList(
        main_win_uuid,
        list_uuid,
        list
  )) {
    Ok(_) => {},
    Err(err) => println!("Cannot send UserEvent event: {err}"),
  }
}
