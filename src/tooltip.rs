use winit::{
  dpi::{LogicalSize, PhysicalPosition},
  event::{WindowEvent},
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

use crate::child_window::{
  ChildWindow,
  Layout,
  LayoutArgs,
  Orientation,
};
use crate::label::Label;
use crate::row_layout::RowLayout;
use crate::UserEvent;
use crate::WindowUtils;

/// A pop-up tooltip window
pub struct ToolTip {
  uuid: Uuid,
  main_win_uuid: Uuid,
  event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
  window: Window,
  pixmap: Pixmap,
  width: f64,
  height: f64,
  layout: Box<dyn Layout>,
  initial_draw_performed: bool,
  label_rc: Rc<RefCell<Label>>,
}

impl ToolTip {

  pub fn new(
        uuid: Uuid,
        main_win_uuid: Uuid,
        window: Window,
        event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
        width: f64,
        height: f64
  ) -> ToolTip {

    // Set the layout
    let mut layout = Box::new(RowLayout::new(
            event_loop_proxy.clone(),
            main_win_uuid.clone(),
            Orientation::Vertical,
            5.0
    ));

    // Create the Label that will display the text
    let label = Label::new(
          event_loop_proxy.clone(),
          main_win_uuid,
          "".to_string(),
          Color::BLACK,
          Color::WHITE
    );
    let layout_rc = Rc::new(RefCell::new(label));
    let _ = layout.add_child(layout_rc.clone(), LayoutArgs::None);

    // Hide the window
    window.set_visible(false);

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
      label_rc: layout_rc,
    }
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

      WindowEvent::RedrawRequested => {

        // Set the initially drawn flag
        if !self.initial_draw_performed {
          self.initial_draw_performed = true;
        }

        self.redraw();
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

  fn set_size(&mut self, width: f64, height: f64) {

    let size = LogicalSize::new(width, height);

    self.window.set_min_inner_size(Some(size));
    self.window.set_max_inner_size(Some(size));
  }

  pub fn set_text(&mut self, text: String) {

    let child_width;
    let child_height;
    {
      // Set the text
      let mut label_ref = self.label_rc.borrow_mut();
      label_ref.set_text(text);
      (child_width, child_height) = label_ref.get_drawing_size();
    }

    // Resize the window to accomodate the new text
    self.set_size(child_width + 4.0, child_height + 4.0);
  }

  pub fn set_visible(&self, flag: bool) {
    self.window.set_visible(flag);
  }
}

impl Debug for ToolTip {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {

    let visible = match self.window.is_visible() {
      Some(visible) => visible.to_string(),
      None => "unknown".to_string(),
    };

    write!(fmt, "ToolTip; visibility: {}",
          visible
    )
   }
}
