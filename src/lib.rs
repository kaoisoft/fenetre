//! This is a GUI library for Rust that can be used to create GUI-based applications.

use winit::{
  application::ApplicationHandler,
  dpi::{
    LogicalPosition,
    LogicalSize,
    PhysicalSize,
    Position,
  },
  event::{
    ElementState,
    KeyEvent,
    MouseButton,
    WindowEvent,
  },
  event_loop::{
    ActiveEventLoop,
    ControlFlow,
    EventLoop,
    EventLoopProxy,
  },
  monitor::MonitorHandle,
  window::{ Window, WindowAttributes, WindowId },
};
use softbuffer::{ Context, Surface };
use tiny_skia::{ Color, Pixmap, PixmapPaint, Transform };

use std::{
  cell::RefCell,
  collections::HashMap,
  num::NonZeroU32,
  rc::Rc,
  time::{Duration, Instant},
};

use uuid::Uuid;

pub mod child_window;
use crate::child_window::{
  BorderLocation,
  ChildType,
  ChildWindow,
  Layout,
  LayoutArgs,
  LayoutType,
  UserEvent,
};
pub mod border_layout;
pub mod button;
pub mod context_menu;
pub mod image_view;
pub mod label;
pub mod layout_base;
pub mod line_edit;
pub mod list;
pub mod menubar;
pub mod multi_line_edit;
pub mod popup;
pub mod row_layout;
pub mod scroll_bar;
pub mod scroll_layout;
pub mod slider;
pub mod status_bar;
pub mod tab_layout;
pub mod table;
pub mod text_font;
pub mod tooltip;
pub mod window_base;
pub mod window_utils;
use crate::border_layout::BorderLayout;
use crate::child_window::Orientation;
use crate::context_menu::ContextMenu;
use crate::menubar::MenuBar;
use crate::popup::PopUp;
use crate::status_bar::StatusBar;
use crate::tooltip::ToolTip;
use crate::window_utils::WindowUtils;

const DOUBLE_CLICK_TIME: u64 = 500;

pub enum MainAppSize {
  Actual(f64, f64, f64, f64),   // Actual location and size (x, y, width, height)
  Relative(f64, f64, f64, f64), // Relative location and size (percentages)
}

/// Main window for a GUI application
pub struct MainApp {
  id: Uuid,
  window_id: Option<WindowId>,
  event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
  startup_callback: Box<dyn Fn()>,
  monitor: Option<MonitorHandle>,
  location_and_size: MainAppSize,
  title: String,
  window: Option<Window>,
  layout: Box<dyn Layout>,
  menubar: Option<Rc<RefCell<MenuBar>>>,
  statusbar: Option<Rc<RefCell<StatusBar>>>,
  focus_window: Option<Rc<RefCell<dyn ChildWindow>>>,
  modal_window: Option<Rc<RefCell<PopUp>>>,
  context_menu: Option<Rc<RefCell<ContextMenu>>>,   // there is only one context menu for the entire application
  context_menu_id: WindowId,
  tooltip_popup: Option<Rc<RefCell<ToolTip>>>,    // there is only one tooltip pop-up for the entire application
  tooltip_popup_id: WindowId,
  cursor_x: f64,                  // Current mouse location within MainApp
  cursor_y: f64,
  mouse_left_button_down: bool,
  pixmap: Pixmap,
  x: f64,
  y: f64,
  width: f64,
  height: f64,
  dragging: bool,
  drag_start_win_x: f64,
  drag_start_win_y: f64,
  popups: HashMap<WindowId, Rc<RefCell<PopUp>>>,
  log_unhandled_events: bool,
  initial_draw_performed: bool,
  last_mouse_left_click: Instant,
  caret_moved_event_callback: Option<Box<dyn Fn(Uuid, usize, usize)>>,
  close_tab_event_callback: Option<Box<dyn Fn(Uuid, Uuid)>>,
  create_context_menu_event_callback: Option<Box<dyn Fn(Uuid, f64, f64)>>,
  delete_items_event_callback: Option<Box<dyn Fn(Uuid, Uuid)>>,
  process_selected_items_event_callback: Option<Box<dyn Fn(Uuid)>>,
  redraw_event_callback: Option<Box<dyn Fn(f64, f64, Pixmap)>>,
  redraw_all_event_callback: Option<Box<dyn Fn()>>,
  selection_changed_event_callback: Option<Box<dyn Fn(Uuid)>>,
  set_list_event_callback: Option<Box<dyn Fn(Uuid, Vec<String>)>>,
  slider_value_changed_event_callback: Option<Box<dyn Fn(f64)>>,
  user_defined_event_callback: Option<Box<dyn Fn(u64, Vec<String>)>>,
  window_created_callback: Option<Box<dyn Fn(Rc<RefCell<PopUp>>)>>,
  end_program_handler: Option<Box<dyn Fn() -> bool>>,
}

impl MainApp {

  /// Creates an instance of MainApp
  ///
  /// The startup_callback is called when the main window is drawn for the first time
  pub fn new<F: Fn() + 'static>(
    id: Uuid,
    title: &str,
    location_and_size: MainAppSize,
    event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
    startup_callback: F,
  ) -> RefCell<MainApp> {

    // Set the layout
    let mut layout = Box::new(BorderLayout::new(event_loop_proxy.clone(), id));
    layout.set_name("MainApp's BorderLayout".to_string());

    // Ignore the drawing events from the children until the event loop is started
    WindowUtils::set_ignore_events(true);

    let inst = MainApp {
      id: id,
      event_loop_proxy: event_loop_proxy,
      window_id: None,
      startup_callback: Box::new(startup_callback),
      monitor: None,
      location_and_size: location_and_size,
      title: title.to_string(),
      window: None,
      layout: layout,
      menubar: None,
      statusbar: None,
      focus_window: None,
      modal_window: None,
      context_menu: None,   // This will be created when the window is created
      context_menu_id: WindowId::dummy(),
      tooltip_popup: None,  // This will be created when the window is created
      tooltip_popup_id: WindowId::dummy(),
      cursor_x: 0.0,
      cursor_y: 0.0,
      mouse_left_button_down: false,
      pixmap: Pixmap::new(800, 600).unwrap(),
      x: 0.0,
      y: 0.0,
      width: 800.0,
      height: 600.0,
      dragging: false,
      drag_start_win_x: 0.0,
      drag_start_win_y: 0.0,
      popups: HashMap::new(),
      log_unhandled_events: true,
      initial_draw_performed: false,
      last_mouse_left_click: Instant::now(),
      caret_moved_event_callback: None,
      close_tab_event_callback: None,
      create_context_menu_event_callback: None,
      delete_items_event_callback: None,
      process_selected_items_event_callback: None,
      redraw_event_callback: None,
      redraw_all_event_callback: None,
      selection_changed_event_callback: None,
      set_list_event_callback: None,
      slider_value_changed_event_callback: None,
      user_defined_event_callback: None,
      window_created_callback: None,
      end_program_handler: None,
    };

    RefCell::new(inst)
  }

  /// Adds an item to the menubar
  pub fn add_menu_item<F: Fn() + 'static>(&self, label: String, callback: F) {

    match &self.menubar {
      Some(menubar) => {
        let mut menubar_ref = menubar.borrow_mut();
        menubar_ref.add_item(label, callback);
      },
      None => println!("Menu bar has not been enabled"),
    };
  }

  // Creates a pop-up context menu that is initally hidden.
  fn create_context_menu(
        &mut self,
        event_loop: &ActiveEventLoop,
        x: f64,
        y: f64,
  ) {

    let width = 200.0;
    let height = 100.0;

    let window_attributes = WindowAttributes::default()
            .with_inner_size(LogicalSize::new(width, height))
            .with_position(Position::Logical(LogicalPosition::new(x, y)))
            .with_decorations(false)
            .with_visible(false);   // with_visible() and set_visible() are not supported on all platforms

    match event_loop.create_window(window_attributes) {

      Ok(window) => {
        self.context_menu_id = window.id();

        // Create a context menu for this window
        let context_id = Uuid::new_v4();
        let context_menu = ContextMenu::new(
              context_id,
              self.id,
              window,
              self.event_loop_proxy.clone(),
              width,
              height
        );
        self.context_menu = Some(Rc::new(RefCell::new(context_menu)));

        // Initially hide the context menu
        self.set_context_menu_visible(false);
      },

      Err(err) => {
        println!("Failed to create context menu: {err}");
      },
    }
  }

  // Creates a pop-up window
  fn create_popup(
        &mut self,
        popup_uuid: Uuid,
        event_loop: &ActiveEventLoop,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        modal: bool,
  ) {

    let window_attributes = WindowAttributes::default()
            .with_inner_size(LogicalSize::new(width, height))
            .with_position(Position::Logical(LogicalPosition::new(x, y)))
            .with_visible(true);

    match event_loop.create_window(window_attributes) {

      Ok(window) => {

        // Create a PopUp for this window
        let window_id = window.id();
        let popup = PopUp::new(
              popup_uuid,
              self.id,
              window,
              self.event_loop_proxy.clone(),
              width,
              height
        );
        let popup_rc = Rc::new(RefCell::new(popup));

        // Save the PopUp in the map
        self.popups.insert(window_id, popup_rc.clone());
        if modal {
          self.modal_window = Some(popup_rc.clone());
        }

        // Execute the successful creation handler, if one was set.
        match &self.window_created_callback {

          Some(callback) => callback(popup_rc.clone()),

          None => {
            println!("Window created callback has not been set");
          },
        }
      },

      Err(err) => {
        println!("Failed to create pop-up window: {err}");
      },
    }
  }

  // Creates a pop-up tooltip window that is initally hidden.
  fn create_tooltip(
        &mut self,
        event_loop: &ActiveEventLoop,
        x: f64,
        y: f64,
  ) {

    let width = 200.0;
    let height = 100.0;

    let window_attributes = WindowAttributes::default()
            .with_inner_size(LogicalSize::new(width, height))
            .with_position(Position::Logical(LogicalPosition::new(x, y)))
            .with_decorations(false)
            .with_visible(false);   // with_visible() and set_visible() are not supported on all platforms

    match event_loop.create_window(window_attributes) {

      Ok(window) => {
        self.tooltip_popup_id = window.id();

        // Create a tooltip pop-up window
        let tooltip_popup_id = Uuid::new_v4();
        let tooltip_popup = ToolTip::new(
              tooltip_popup_id,
              self.id,
              window,
              self.event_loop_proxy.clone(),
              width,
              height
        );

        self.tooltip_popup = Some(Rc::new(RefCell::new(tooltip_popup)));

        // Initially hide the tooltip window
        self.set_tooltip_visible(false);
      },

      Err(err) => {
        println!("Failed to create tooltip window: {err}");
      },
    }
  }

  // Displays the Pixmap onto the screen
  fn display_pixmap(&mut self) {

    match &self.window {

      Some(window) => {

        // Get the size of the window
        let width = self.pixmap.width();
        let height = self.pixmap.height();

        // Create the drawing surface
        let context = (Context::new(&window)).unwrap();
        let mut surface = (Surface::new(&context, &window)).unwrap();
        surface
            .resize(NonZeroU32::new(width).unwrap(), NonZeroU32::new(height).unwrap())
            .unwrap();

        // Copy the pixmap onto the drawing surface
        let mut buffer = surface.buffer_mut().unwrap();
        for index in 0..(width * height) as usize {
          buffer[index] =
              (self.pixmap.data()[index * 4 + 2] as u32) |
              ((self.pixmap.data()[index * 4 + 1] as u32) << 8) |
              ((self.pixmap.data()[index * 4] as u32) << 16);
        }

        buffer.present().unwrap();
      },

      None => {},
    }
  }

  /// Adds a menu bar to the main window
  pub fn enable_menubar(
        &mut self,
        event_loop: Rc<EventLoopProxy<UserEvent>>
  ) {

    let menubar = MenuBar::new(event_loop, self.id);
    let menubar_rc = Rc::new(RefCell::new(menubar));
    match self.layout.add_child(menubar_rc.clone(), LayoutArgs::BORDER(BorderLocation::TOP)) {
      Ok(_) => self.menubar = Some(menubar_rc.clone()),
      Err(err) => println!("Could not add menu bar to main window: {err}"),
    }
  }

  /// Adds a status bar to the main window
  pub fn enable_statusbar(
        &mut self,
        event_loop: Rc<EventLoopProxy<UserEvent>>
  ) {

    let statusbar = StatusBar::new(
          event_loop,
          self.id,
          Color::from_rgba8(191, 191, 191, 255)
    );
    let statusbar_rc = Rc::new(RefCell::new(statusbar));
    match self.layout.add_child(
          statusbar_rc.clone(),
          LayoutArgs::BORDER(BorderLocation::BOTTOM)
    ) {
      Ok(_) => self.statusbar = Some(statusbar_rc.clone()),
      Err(err) => println!("Could not add status bar to main window: {err}"),
    }
  }

  pub fn get_location(&self) -> (f64, f64) {
    (self.x, self.y)
  }

  pub fn get_monitor(&self) -> Option<MonitorHandle> {
    self.monitor.clone()
  }

  pub fn get_size(&self) -> (f64, f64) {

    match &self.window {

      Some(window) => {

        // Get the size of the window
        let size = window.inner_size();
        (size.width as f64, size.height as f64)
      },

      None => (self.width, self.height),
    }
  }

  pub fn get_status_bar(&self) -> Option<Rc<RefCell<StatusBar>>> {
    match &self.statusbar {
      Some(status_bar) => Some(status_bar.clone()),
      None => None,
    }
  }

  pub fn get_uuid(&self) -> Uuid {
    self.id
  }

  pub fn get_window_id(&self) -> Option<WindowId> {
    self.window_id
  }

  fn handle_keyboard_pressed_event(&mut self, _event: KeyEvent) {
  }
  fn handle_keyboard_released_event(&mut self, _event: KeyEvent) {
  }

  fn handle_mouse_pressed(&mut self, _button: MouseButton) {
  }
  fn handle_mouse_released(&mut self, _button: MouseButton) {
  }

  /// Sets the state of the log unhandled events flag
  pub fn log_unhandled_events(&mut self, flag: bool) {
    self.log_unhandled_events = flag;
  }

  pub fn redraw(&mut self) {

    // Get the size of the window
    let (width, height) = self.get_size();

    // Fill the pixmap with the background color
    self.pixmap.fill(Color::WHITE);

    // Get the layout's pixmap
    let layout_pixmap = self.layout.layout(0.0, 0.0, width as f64, height as f64);

    let paint = PixmapPaint::default();

    // Copy the layout's pixmap image onto the full pixmap
    self.pixmap.draw_pixmap(
        0,
        0,
        layout_pixmap.as_ref(),
        &paint,
        Transform::identity(),
        None,
    );

    // Display the pixmap
    self.display_pixmap();
  }

  pub fn run_event_loop(&mut self, event_loop: EventLoop<UserEvent>) {

    // Stop ignoring the draw events from the children
    WindowUtils::set_ignore_events(false);

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    //event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    match event_loop.run_app(self) {
      Ok(_) => {},
      Err(err) => {
        println!("Could not run event loop: {err}");
      },
    };
  }

  pub fn set_close_tab_event_callback(&mut self, callback: Box<dyn Fn(Uuid, Uuid)>) {
    self.close_tab_event_callback = Some(callback);
  }

  // Hides or displays the context menu
  fn set_context_menu_visible(&self, visible: bool) {

        match &self.context_menu {

          Some(context_menu_rc) => {

            let context_menu = context_menu_rc.borrow();
            context_menu.set_visible(visible);
          },

          None => {},
        }
  }

  pub fn set_caret_moved_event_callback(&mut self, callback: Box<dyn Fn(Uuid, usize, usize)>) {
    self.caret_moved_event_callback = Some(callback);
  }

  pub fn set_create_context_menu_event_callback(&mut self, callback: Box<dyn Fn(Uuid, f64, f64)>) {
    self.create_context_menu_event_callback = Some(callback);
  }

  pub fn set_contents(&mut self, contents: ChildType) {

    match contents {
      ChildType::Window(window) => match self.layout.add_child(window, LayoutArgs::BORDER(BorderLocation::CENTER)) {
        Ok(_) => {},
        Err(err) => println!("Cannot set contents: {err}"),
      },
      ChildType::Layout(layout) => match self.layout.add_layout(layout, LayoutArgs::BORDER(BorderLocation::CENTER)) {
        Ok(_) => {},
        Err(err) => println!("Cannot set contents: {err}"),
      },
    }
  }

  pub fn set_delete_items_event_callback(&mut self, callback: Box<dyn Fn(Uuid, Uuid)>) {
    self.delete_items_event_callback = Some(callback);
  }

  pub fn set_end_program_handler(&mut self, handler: Box<dyn Fn() -> bool>) {
    self.end_program_handler = Some(handler);
  }

  pub fn set_modal(&mut self, child: Option<Rc<RefCell<PopUp>>>) {
    self.modal_window = child;
  }

  pub fn set_process_selected_items_event_callback(&mut self, callback: Box<dyn Fn(Uuid)>) {
    self.process_selected_items_event_callback = Some(callback);
  }

  pub fn set_selection_changed_event_callback(&mut self, callback: Box<dyn Fn(Uuid)>) {
    self.selection_changed_event_callback = Some(callback);
  }

  pub fn set_set_list_event_callback(&mut self, callback: Box<dyn Fn(Uuid, Vec<String>)>) {
    self.set_list_event_callback = Some(callback);
  }

  pub fn set_slider_value_changed_event_callback(&mut self, callback: Box<dyn Fn(f64)>) {
    self.slider_value_changed_event_callback = Some(callback);
  }

  pub fn set_status_message(&self, message: String) {

    match &self.statusbar {
      Some(statusbar) => {

        let mut statusbar_ref = statusbar.borrow_mut();
        statusbar_ref.set_message(message);
      },
      None => {},
    }
  }

  // Hides or displays the tooltip window
  fn set_tooltip_visible(&self, visible: bool) {

    match &self.tooltip_popup {

      Some(tooltip_rc) => {

        let tooltip_ref = tooltip_rc.borrow();
        tooltip_ref.set_visible(visible);
      },

      None => {},
    }
}

  pub fn set_redraw_event_callback(&mut self, callback: Box<dyn Fn(f64, f64, Pixmap)>) {
    self.redraw_event_callback = Some(callback);
  }

  pub fn set_redraw_all_event_callback(&mut self, callback: Box<dyn Fn()>) {
    self.redraw_all_event_callback = Some(callback);
  }

  pub fn set_user_defined_event_callback(&mut self, callback: Box<dyn Fn(u64, Vec<String>)>) {
    self.user_defined_event_callback = Some(callback);
  }

  pub fn set_window_created_callback(&mut self, callback: Box<dyn Fn(Rc<RefCell<PopUp>>)>) {
    self.window_created_callback = Some(callback);
  }

  // Sets the visiblity of the context menu, and tells the displaying
  // window to populate it, if it is being made visible.
  fn show_context_menu(&mut self, flag: bool, source_uuid: Uuid, x: f64, y: f64) {

    match &self.context_menu {

      Some(context_menu_rc) => {

        if true == flag {   // Menu is being made visible

          // Move the menu
          {     // limit the scope of the borrow

            // Move the menu's window
            let context_menu = context_menu_rc.borrow();
            context_menu.move_window(self.x + x, self.y + y); // Convert to screen coordinates
          }

          // Tell the window to populate it
          match self.layout.get_child_with_id(source_uuid) {

            Some(child_rc) => {

              let child = child_rc.borrow();
              child.populate_context_menu(context_menu_rc.clone());
            },

            None => {},
          }
        }

        // Set the menu's visibility
        self.set_context_menu_visible(flag);
      },

      None => {
      },
    }
  }

  // Show the specified text in a pop-up.
  fn show_tooltip(&mut self, text: String, x: f64, y: f64) {

    match &self.tooltip_popup {

      Some(tooltip_rc) => {

        // Move the tooltip window
        {     // limit the scope of the borrow

          // Move the menu's window
          let mut tooltip_ref = tooltip_rc.borrow_mut();
          tooltip_ref.move_window(self.x + x, self.y + y); // Convert to screen coordinates

          // Set the tooltip's text
          tooltip_ref.set_text(text);
        }

        // Start a timer thread to hide the tooltip
        // debug

        // Set the tooltip's visibility
        self.set_tooltip_visible(true);
      },

      None => {
      },
    }
  }
}

impl ApplicationHandler<UserEvent> for MainApp {

  fn resumed(&mut self, event_loop: &ActiveEventLoop) {

    self.monitor = event_loop.primary_monitor();

    // Calculate the location and size of the window
    match self.location_and_size {
      MainAppSize::Actual(actual_x, actual_y, actual_width, actual_height) => {
        self.x = actual_x;
        self.y = actual_y;
        self.width = actual_width;
        self.height = actual_height;
      },

      MainAppSize::Relative(percent_x, percent_y, percent_width, percent_height) => {

        let monitor_size = match &self.monitor {
          Some(monitor) => monitor.size(),
          None => PhysicalSize::new(800, 600),
        };

        self.x = monitor_size.width as f64 * percent_x;
        self.y = monitor_size.height as f64 * percent_y;
        self.width = monitor_size.width as f64 * percent_width;
        self.height = monitor_size.height as f64 * percent_height;
      },
    }

    self.pixmap = Pixmap::new(self.width as u32, self.height as u32).unwrap();

    let window_attributes = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_inner_size(LogicalSize::new(self.width, self.height))
            .with_position(Position::Logical(LogicalPosition::new(self.x, self.y)))
            .with_visible(true);
    self.window = match event_loop.create_window(window_attributes) {
      Ok(window) => {
        self.window_id = Some(window.id());
        Some(window)
      },
      Err(err) => {
        println!("In MainApp::resume(), cannot create main window: {err}");
        None
      },
    };

    // Create the initially hidden context menu
    self.create_context_menu(event_loop, self.x, self.y);

    // Create the initially hidden tooltip window
    self.create_tooltip(event_loop, self.x, self.y);
  }

  // Called when a UserEvent is processed by the event loop
  fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {

    match event {

      UserEvent::CaretMoved(source_uuid, line_index, char_index) => {

        match &self.caret_moved_event_callback {

          Some(callback) => callback(source_uuid, line_index, char_index),

          None => {},
        }
      },

      UserEvent::ClosePopUp(_main_win_uuid, window_id) => {

        // Remove the popup from the map
        self.popups.remove(&window_id);

        // If this PopUp is the modal window, clear the modal window.
        let mut matches = false;
        if let Some(modal_window) = &self.modal_window {

          let modal_window_ref = modal_window.borrow();
          if modal_window_ref.get_window_id() == window_id {
            matches = true;
          }
        }

        if matches {
          self.modal_window = None;
        }
      },

      UserEvent::CloseTab(_main_win_uuid, source_uuid, tab_uuid) => {

        match &self.close_tab_event_callback {

          Some(callback) => callback(source_uuid, tab_uuid),

          None => {

            if self.log_unhandled_events {
              println!("Received an unhandled CloseTab event: tab_uuid = {tab_uuid}");
            }
          },
        }
      },

      UserEvent::CreateWindow(
        _main_win_uuid,
        uuid,
        x,
        y,
        width,
        height,
        modal
      ) => {

        self.create_popup(uuid, event_loop, x, y, width, height, modal);
      },

      UserEvent::DeleteItems(main_win_id, source_id) => {

        match &self.delete_items_event_callback {

          Some(callback) => callback(main_win_id, source_id),

          None => {},
        }
      },

      UserEvent::HideContextMenu(_main_win_uuid) => {
        self.set_context_menu_visible(false);
      },

      UserEvent::ProcessSelectedItems(_main_win_uuid, source) => {

        match &self.process_selected_items_event_callback {

          Some(callback) => callback(source),

          None => {

            if self.log_unhandled_events {
              println!("Received an unhandled ProcessSelectedItems event: source = {source}");
            }
          },
        }
      },

      UserEvent::Redraw(main_win_uuid, x, y, ref pixmap) => {

        // If the top-level window associated with this event is one of
        // the pop-ups, pass the event to it.
        for (_popup_window_id, popup_rc) in self.popups.clone().into_iter() {

          let mut popup_ref = popup_rc.borrow_mut();
          if popup_ref.get_uuid() == main_win_uuid {
            popup_ref.process_user_event(event_loop, event);
            return;
          }
        }

        match &self.redraw_event_callback {

          Some(callback) => callback(x, y, pixmap.clone()),

          None => {

            // Update the MainApp's pixmap
            self.pixmap.draw_pixmap(
                x as i32,
                y as i32,
                pixmap.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                None,
            );

            // Display the updated pixmap
            self.display_pixmap();
          },
        }
      },

      UserEvent::RedrawAll(_main_win_uuid) => {

        match &self.redraw_all_event_callback {

          Some(callback) => callback(),

          None => self.redraw(),
        }
      },

      UserEvent::ScrollValueChanged(
            main_win_uuid,
            scroll_bar_uuid,
            orientation,
            value
      ) => {

        // If the top-level window associated with this event is one of
        // the pop-ups, pass the event to it.
        for (_popup_window_id, popup_rc) in self.popups.clone().into_iter() {

          let popup_ref = popup_rc.borrow();
          if popup_ref.get_uuid() == main_win_uuid {
            popup_ref.scroll_value_changed(scroll_bar_uuid, orientation, value);
            return;
          }
        }

        // Find the ScrollLayout that contains the ScrollBar
        for scroll_layout in self.layout.get_layouts_of_type(LayoutType::ScrollLayout) {

          let scroll_layout_ref = scroll_layout.borrow();

          // See if one of the scroll bars matches
          let mut scroll_child_rc: Option<Rc<RefCell<dyn ChildWindow>>> = None;
          let (scroll_width, scroll_height) = scroll_layout_ref.get_size();
          match scroll_layout_ref.get_child_at(1.0, scroll_height - 1.0) {  // horizontal scroll bar

            Some(scroll_bar) => {

              // Get the window being scrolled
              scroll_child_rc = Some(scroll_bar);
            },

            None => {},
          }
          match scroll_layout_ref.get_child_at(scroll_width - 1.0, 1.0) {  // vertical scroll bar

            Some(scroll_bar) => {

              // Get the window being scrolled
              scroll_child_rc = Some(scroll_bar);
            },

            None => {},
          }

          // Was the scrolling child found?
          match scroll_child_rc {

            Some(scroll_child) => {

              let mut scroll_child_ref = scroll_child.borrow_mut();

              // Set the appropriate scroll value
              match orientation {

                Orientation::Horizontal => scroll_child_ref.set_x_scroll(value),

                Orientation::Vertical => scroll_child_ref.set_y_scroll(value),
              }
            },

            None => {},
          }
        }
      },

      UserEvent::SelectionChanged(_main_win_uuid, source) => {

        match &self.selection_changed_event_callback {

          Some(callback) => callback(source),

          None => {

            if self.log_unhandled_events {
              println!("Received an unhandled SelectionChanged event: source = {source}");
            }
          },
        }
      },

      UserEvent::SetList(_main_win_uuid, dest, list) => {

        match &self.set_list_event_callback {

          Some(callback) => callback(dest, list),

          None => {

            if self.log_unhandled_events {
              println!("Received an unhandled SetList event: destination = {dest}, list = {:?}", list);
            }
          },
        }
      },

      UserEvent::ShowContextMenu(_main_win_uuid, source_uuid, x, y) => {

        match &self.create_context_menu_event_callback {

          Some(callback) => callback(source_uuid, x, y),

          None => {

            self.show_context_menu(true, source_uuid, x, y);
          },
        }
      },

      UserEvent::ShowToolTip(main_win_uuid, source_uuid, text) => {

        // Currently, tooltips only works for windows within the main window
        if main_win_uuid == self.id {

          // If the window that fired the event has a tooltip, display
          // it in the tooltip window.
          match self.layout.get_child_with_id(source_uuid) {

            Some(child_rc) => {

              let child_ref = child_rc.borrow();
              let (child_x, child_y) = child_ref.get_location();
              self.show_tooltip(text, child_x, child_y);
            },

            None => {},
          }
        }
      },

      UserEvent::SliderValueChange(value) => {

        match &self.slider_value_changed_event_callback {

          Some(callback) => callback(value),

          None => {

            if self.log_unhandled_events {
              println!("Received an unhandled SliderValueChanged event: value = {value}");
            }
          },
        }
      },

      // This event cannot be overridden with a callback
      UserEvent::TabSelected(_main_win_uuid, tab_layout_uuid, tab_uuid) => {

        // Get the TabLayout
        match self.layout.get_layout_with_id(tab_layout_uuid) {

          Some(layout_rc) => {

            // Set the layout's active tab
            let mut tab_layout = layout_rc.borrow_mut();
            tab_layout.set_active_tab_by_uuid(tab_uuid);
          },

          None => {
          },
        }
      },

      UserEvent::UpdateScroller(main_win_uuid, scroll_layout_uuid) => {

        // If the top-level window associated with this event is one of
        // the pop-ups, pass the event to it.
        for (_popup_window_id, popup_rc) in self.popups.clone().into_iter() {

          let mut popup_ref = popup_rc.borrow_mut();
          if popup_ref.get_uuid() == main_win_uuid {
            popup_ref.process_user_event(event_loop, event);
            return;
          }
        }

        // Tell the ScrollLayout to redraw
        match self.layout.get_layout_with_id(scroll_layout_uuid) {

          Some(layout) => {

            let mut layout_ref = layout.borrow_mut();
            let (x, y) = layout_ref.get_location();
            let (width, height) = layout_ref.get_size();
            layout_ref.layout(x, y, width, height);
          },

          None => {},
        }
      },

      UserEvent::UserDefined(_main_win_uuid, msg_no, data) => {

        match &self.user_defined_event_callback {

          Some(callback) => callback(msg_no, data),

          None => {

            if self.log_unhandled_events {
              println!("Received an unhandled UserDefined event: msg_no = {msg_no}, data = {:?}", data);
            }
          },
        }
      },
    }
  }

  // Called when a WindowEvent is processed by the event loop
  fn window_event(&mut self,
        event_loop: &ActiveEventLoop,
        id: WindowId,
        event: WindowEvent
  ) {

    // If this event is for one of the popups or a context menu, pass it on.
    match self.window_id {

      Some(window_id) => {

        if window_id != id {

          match self.popups.get(&id) {

            Some(popup_rc) => {
              let mut popup = popup_rc.borrow_mut();
              popup.process_event(event);
              return;
            },

            None => {},
          }

          if id == self.context_menu_id {

            match &self.context_menu {

              Some(context_menu_rc) => {
                let mut context_menu = context_menu_rc.borrow_mut();
                context_menu.process_event(event);
              },

              None => {},
            }
          }

          // Don't do any further processing of this event
          return;
        }
      },

      None => {},
    }

    // This event is for this window
    match event {
        WindowEvent::CloseRequested => {

          let do_continue = match &self.end_program_handler {

            Some(end_program_handler) => end_program_handler(),

            None => true,
          };

          // Exit the event loop, which will end the program
          if do_continue {
            event_loop.exit();
          }
        },

        WindowEvent::CursorMoved{position, ..} => {

          // Save the new position, which is relative to the main window
          self.cursor_x = position.x;
          self.cursor_y = position.y;

          match &self.modal_window {

            Some(window) => {

              // Pass the event to the modal window
              let mut child_ref = window.borrow_mut();
              if self.mouse_left_button_down {
                child_ref.handle_mouse_drag(position.x, position.y);
              } else {
                child_ref.handle_mouse_movement(position.x, position.y);
              }
            },
            None => {

              // Pass the event to the window under the mouse
              match &self.layout.get_child_at(position.x, position.y) {
                  Some(window) => {
                    let mut child_ref = window.borrow_mut();
                    if self.mouse_left_button_down {
                      child_ref.handle_mouse_drag(position.x, position.y);
                    } else {
                      child_ref.handle_mouse_movement(position.x, position.y);
                    }
                  },
                  None => {},
                };
            },
          }
        },

        WindowEvent::KeyboardInput{device_id: _, event, is_synthetic: _} => {

          let key_pressed = event.state == ElementState::Pressed;

          match &self.modal_window {

            Some(window) => {
              let mut child_ref = window.borrow_mut();
              if key_pressed {
                child_ref.handle_keyboard_pressed_event(event);
              } else {
                child_ref.handle_keyboard_released_event(event);
              }
            },
            None => {
              match &self.focus_window {
                  Some(window) => {
                    let mut child_ref = window.borrow_mut();
                    child_ref.set_focused(true);
                    if key_pressed {
                      child_ref.handle_keyboard_pressed_event(event);
                    } else {
                      child_ref.handle_keyboard_released_event(event);
                    }
                  },
                  None => {
                    if key_pressed {
                      self.handle_keyboard_pressed_event(event);
                    } else {
                      self.handle_keyboard_released_event(event);
                    }
                  },
                };
            },
          }
        },

        WindowEvent::MouseInput{state, button, ..} => {

          match state {

            ElementState::Pressed => {

              self.mouse_left_button_down = true;

              if MouseButton::Left == button {
                self.dragging = true;
                self.drag_start_win_x = self.cursor_x;
                self.drag_start_win_y = self.cursor_y;

                // Is this the second part of a double click?
                let timenow = Instant::now();
                if timenow.duration_since(self.last_mouse_left_click) <
                      Duration::from_millis(DOUBLE_CLICK_TIME) {

                  // Pass a double-click event
                } else {

                  // Start timer to pass the event
                }
              }

              match &self.modal_window {

                Some(window) => {
                  let mut child_ref = window.borrow_mut();
                  child_ref.handle_mouse_pressed(button, self.cursor_x, self.cursor_y);
                },
                None => {

                  // Remove focus from the child that currently has it
                  match &self.focus_window {
                    Some(window) => {
                      let mut child_ref = window.borrow_mut();
                      child_ref.set_focused(false);
                    },
                    None => {},
                  };

                  // Give focus to the window that the cursor is on top of
                  self.focus_window = self.layout.get_child_at(self.cursor_x as f64, self.cursor_y as f64);
                  match &self.focus_window {
                    Some(window) => {
                      let mut child_ref = window.borrow_mut();
                      child_ref.set_focused(true);
                      child_ref.handle_mouse_pressed(button, self.cursor_x, self.cursor_y);
                      child_ref.handle_mouse_drag_start(self.cursor_x, self.cursor_y);
                    },
                    None => {
                      self.handle_mouse_pressed(button);
                    },
                  };
                },
              }
            },

            ElementState::Released => {

              self.mouse_left_button_down = false;

              // Was dragging being performed?
              if MouseButton::Left == button &&
                    (self.cursor_x != self.drag_start_win_x ||
                    self.cursor_y != self.drag_start_win_y) {

                match &self.focus_window {
                  Some(window) => {

                    // Pass the events to the window with focus
                    let mut child_ref = window.borrow_mut();
                    child_ref.handle_mouse_drag(self.cursor_x, self.cursor_y);
                    child_ref.handle_mouse_drag_end(self.cursor_x, self.cursor_y);

                    return;
                  },

                  None => {},
                }
              }
              self.dragging = false;

              match &self.modal_window {

                Some(window) => {

                  // Pass the event to the modal window
                  let mut child_ref = window.borrow_mut();
                  child_ref.handle_mouse_released(button, self.cursor_x, self.cursor_y);
                },
                None => {

                  match &self.focus_window {
                    Some(window) => {

                      // Pass the event to the window with focus
                      let mut child_ref = window.borrow_mut();
                      child_ref.handle_mouse_released(button, self.cursor_x, self.cursor_y);
                    },

                    None => self.handle_mouse_released(button),
                  }
                },
              }
            },
          };
        },

        WindowEvent::MouseWheel{device_id: _, delta, phase} => {

          match &self.modal_window {

            Some(window) => {
              let mut child_ref = window.borrow_mut();
              child_ref.handle_mouse_wheel(delta, phase);
            },
            None => {
              // Give focus to the window that the cursor is on top of
              self.focus_window = self.layout.get_child_at(self.cursor_x as f64, self.cursor_y as f64);
              match &self.focus_window {
                Some(window) => {

                  // Give the window focus
                  let mut child_ref = window.borrow_mut();
                  child_ref.set_focused(true);

                  // Pass the event to the window
                  child_ref.handle_mouse_wheel(delta, phase);
                },
                None => {},
              };
            },
          }
        },

        WindowEvent::Moved(position) => {

          // Save the new location
          self.x = position.x as f64;
          self.y = position.y as f64;
        },

        WindowEvent::RedrawRequested => {

          // Set the initially drawn flag
          if !self.initial_draw_performed {
            self.initial_draw_performed = true;

            // Call the startup callback
            (self.startup_callback)();
          }

          self.redraw();
        },

        WindowEvent::Resized(size) => {

          // If the size of the window has changed, create a new Pixmap for drawing.
          if size.width as f64 != self.width || size.height as f64 != self.height {

            // Save the new size
            self.width = size.width as f64;
            self.height = size.height as f64;

            // Create the pixmap into which we will draw
            self.pixmap = Pixmap::new(self.width as u32, self.height as u32).unwrap();


            // On intial start-up, the main window will receive this event
            // followed by the RedrawRequested event. We only need to redraw
            // on the second event.
            if self.initial_draw_performed {

              // Redraw the window
              self.redraw();
            }
          }
        },

        _ => {},
    }
  }
}
