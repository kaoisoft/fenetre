use fenetre::{
    child_window::{
        ChildType,
        Layout,
        LayoutArgs,
        Orientation,
        UserEvent,
    },
    label::Label,
    MainApp,
    MainAppSize,
    row_layout::RowLayout,
};

use tiny_skia::Color;

use winit::event_loop::{EventLoop, EventLoopProxy};

use uuid::Uuid;

use std::{
    process,
    rc::Rc,
};
use std::cell::RefCell;

struct SimpleWindow {
    event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
    main_app: RefCell<MainApp>,
    main_win_id: Uuid,                      // Main window's ID
    about_id: Uuid,
}

impl SimpleWindow {

    fn new(event_loop_proxy: Rc<EventLoopProxy<UserEvent>>) -> Self {

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
        {
            let mut main_ref = main.borrow_mut();
            main_ref.enable_statusbar(event_loop_proxy.clone());
            main_ref.set_status_message("Ready".to_string());
        }

        Self {
            event_loop_proxy,
            main_app: main,
            main_win_id,
            about_id: Uuid::new_v4(),
        }
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

    // Define the window created handler
    fn define_window_created_handler(&self) {

        let mut main_ref = self.main_app.borrow_mut();

        let event_proxy_rc_clone = self.event_loop_proxy.clone();
        let about_id_clone = self.about_id;
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
            }
        }));
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
    let mut simple_window = SimpleWindow::new(event_loop_proxy.clone());

    // Set up the menu
    simple_window.create_main_menu();

    // Define the event handlers
    simple_window.define_window_created_handler();

    // Run the event loop
    simple_window.run_event_loop(event_loop);
}
