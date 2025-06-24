use winit::{
  dpi::{LogicalSize, PhysicalPosition},
  event::{ElementState, WindowEvent},
  event_loop::EventLoopProxy,
  window::{ Window },
};

use softbuffer::{
  Context,
  Surface,
};

use tiny_skia::{
  Color,
  Pixmap,
  PixmapPaint,
  Transform,
};

use uuid::Uuid;

use std:: {
  cell::RefCell,
  fmt::Debug,
  num::NonZeroU32,
  rc::Rc,
};
use crate::button::Button;
use crate::child_window::{
  Layout,
  LayoutArgs,
  Orientation,
};
use crate::row_layout::RowLayout;
use crate::UserEvent;
use crate::WindowUtils;

/// A pop-up context menu
pub struct ContextMenu {
  uuid: Uuid,
  main_win_uuid: Uuid,
  event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
  window: Window,
  pixmap: Pixmap,
  width: f64,
  height: f64,
  layout: Box<dyn Layout>,
  initial_draw_performed: bool,
  cursor_x: f64,                  // Current mouse location within the window
  cursor_y: f64,
}

impl ContextMenu {

  pub fn new(
        uuid: Uuid,
        main_win_uuid: Uuid,
        window: Window,
        event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
        width: f64,
        height: f64
  ) -> ContextMenu {

    // Set the layout
    let layout = Box::new(RowLayout::new(
            event_loop_proxy.clone(),
            main_win_uuid.clone(),
            Orientation::Vertical,
            5.0
    ));

    // Make the window visible
    window.set_visible(true);

    Self {
      uuid: uuid,
      main_win_uuid: main_win_uuid,
      event_loop_proxy: event_loop_proxy.clone(),
      window: window,
      pixmap: Pixmap::new(width as u32, height as u32).unwrap(),
      width: width,
      height: height,
      layout: layout,
      initial_draw_performed: false,
      cursor_x: 0.0,
      cursor_y: 0.0,
    }
  }

  /// Adds an item to the menu
  pub fn add_item(&mut self, button: Rc<RefCell<Button>>) {

    // Add the button to the layout
    match self.layout.add_child(button, LayoutArgs::None) {

      Ok(_) => {},

      Err(err) => {
        println!("Failed to add item to menu: {err}");
      },
    }
  }
  
  /// Removes all items from the menu
  pub fn clear(&mut self) {
    self.layout.clear();
  }

  // Displays the Pixmap onto the screen
  fn display_pixmap(&mut self) {

    // Get the size of the window
    let width = self.pixmap.width();
    let height = self.pixmap.height();

    // Create the drawing surface
    let context = (Context::new(&self.window)).unwrap();
    let mut surface = (Surface::new(&context, &self.window)).unwrap();
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
  }

  pub fn get_size(&self) -> (f64, f64) {
    (self.width, self.height)
  }

  pub fn get_uuid(&self) -> Uuid {
    self.uuid
  }

  /// Moves the window
  pub fn move_window(&self, x: f64, y: f64) {
    self.window.set_outer_position(PhysicalPosition::new(x, y));
  }

  /// Called by the main window when there is an event for the context menu.
  pub fn process_event(&mut self, event: WindowEvent) {

    match event {
      WindowEvent::CloseRequested => {
        // Notify MainApp, so that the context menu can be hidden
        WindowUtils::fire_user_event(
              self.event_loop_proxy.clone(),
              UserEvent::HideContextMenu(self.main_win_uuid)
        );
      },

      WindowEvent::CursorMoved{position, ..} => {

        // Save the new position, which is relative to the main window
        self.cursor_x = position.x;
        self.cursor_y = position.y;
      },

      WindowEvent::MouseInput{state, button, ..} => {

        match state {

          ElementState::Pressed => {

            // Find the button that is under the mouse
            match self.layout.get_child_at(self.cursor_x, self.cursor_y) {

              Some(child_rc) => {

                // Hide the context menu's window
                WindowUtils::fire_user_event(
                      self.event_loop_proxy.clone(),
                      UserEvent::HideContextMenu(self.main_win_uuid)
                );

                // Pass the event to the button
                let mut child = child_rc.borrow_mut();
                child.handle_mouse_pressed(button, self.cursor_x, self.cursor_y);
              },

              None => {},
            }
          },

          ElementState::Released => {
          },
        };
      },

      WindowEvent::RedrawRequested => {

        // Set the initially drawn flag
        if !self.initial_draw_performed {
          self.initial_draw_performed = true;
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

  fn redraw(&mut self) {

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

  pub fn set_size(&mut self, width: f64, height: f64) {

    let size = LogicalSize::new(width, height);

    self.window.set_min_inner_size(Some(size));
    self.window.set_max_inner_size(Some(size));
  }

  pub fn set_visible(&self, flag: bool) {
    self.window.set_visible(flag);
  }
}

impl Debug for ContextMenu {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {

    let visible = match self.window.is_visible() {
      Some(visible) => visible.to_string(),
      None => "unknown".to_string(),
    };

    write!(fmt, "ContextMenu; visibility: {}",
          visible
    )
   }
}
