use winit::{
  event::{
    ElementState,
    MouseButton,
    WindowEvent,
  },
  event_loop::{ActiveEventLoop, EventLoopProxy},
  window::{ Window, WindowId},
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
  num::NonZeroU32,
  rc::Rc,
};
use winit::event::{KeyEvent, MouseScrollDelta, TouchPhase};
use crate::BorderLayout;
use crate::ChildWindow;
use crate::child_window::{
  BorderLocation,
  ChildType,
  Layout,
  LayoutArgs,
  LayoutType,
  Orientation,
};
use crate::UserEvent;
use crate::WindowUtils;

/// A pop-up window
pub struct PopUp {
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
  focus_window: Option<Rc<RefCell<dyn ChildWindow>>>,
  dragging: bool,
  drag_start_win_x: f64,
  drag_start_win_y: f64,
  mouse_left_button_down: bool,
}

impl PopUp {

  pub fn new(
        uuid: Uuid,
        main_win_uuid: Uuid,
        window: Window,
        event_loop_proxy: Rc<EventLoopProxy<UserEvent>>,
        width: f64,
        height: f64
  ) -> PopUp {

    // Set the layout
    let mut layout = Box::new(BorderLayout::new(event_loop_proxy.clone(), uuid));
    layout.set_name("PopUp's BorderLayout".to_string());

    Self {
      uuid: uuid,
      main_win_uuid: main_win_uuid,
      event_loop_proxy: event_loop_proxy,
      window: window,
      pixmap: Pixmap::new(width as u32, height as u32).unwrap(),
      width: width,
      height: height,
      layout: layout,
      initial_draw_performed: false,
      cursor_x: 0.0,
      cursor_y: 0.0,
      focus_window: None,
      dragging: false,
      drag_start_win_x: 0.0,
      drag_start_win_y: 0.0,
      mouse_left_button_down: false,
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

  pub fn get_window_id(&self) -> WindowId {
    self.window.id()
  }

  /// Processes keyboard events when this window has focus
  pub fn handle_keyboard_pressed_event(&mut self, _event: KeyEvent) {
  }
  pub fn handle_keyboard_released_event(&mut self, _event: KeyEvent) {
  }

  /// Processes mouse click events when this window has focus
  ///
  /// mouse_x and mouse_y are relative to the main window
  pub fn handle_mouse_pressed(&mut self, _button: MouseButton,
                          _mouse_x: f64, _mouse_y: f64) {
  }
  pub fn handle_mouse_released(&mut self, _button: MouseButton,
                           _mouse_x: f64, _mouse_y: f64) {
  }

  /// Processes mouse drag event
  pub fn handle_mouse_drag(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }
  pub fn handle_mouse_drag_start(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }
  pub fn handle_mouse_drag_end(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }

  /// Processes mouse movement events when this window has focus
  pub fn handle_mouse_movement(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }

  /// Processes mouse wheel events when this window has focus
  pub fn handle_mouse_wheel(&mut self, _delta: MouseScrollDelta, _phase: TouchPhase) {
  }

  pub fn process_event(&mut self, event: WindowEvent) {

    match event {
      WindowEvent::CloseRequested => {
        // Notify MainApp, so that this window can be removed from map and the window dropped
        WindowUtils::fire_user_event(
          self.event_loop_proxy.clone(),
          UserEvent::ClosePopUp(
                self.main_win_uuid,
                self.window.id()
          )
        );
      },

      WindowEvent::CursorMoved{position, ..} => {

        // Save the new position, which is relative to the main window
        self.cursor_x = position.x;
        self.cursor_y = position.y;

        // Pass the event to the window with focus
        match &self.focus_window {

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

      WindowEvent::KeyboardInput{device_id: _, event, is_synthetic: _} => {

        let key_pressed = event.state == ElementState::Pressed;

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
          None => {},
        };
      },

      WindowEvent::MouseInput{state, button, ..} => {

        match state {

          ElementState::Pressed => {

            self.mouse_left_button_down = true;

            if MouseButton::Left == button {
              self.dragging = true;
              self.drag_start_win_x = self.cursor_x;
              self.drag_start_win_y = self.cursor_y;
            }

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
              None => {},
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

            match &self.focus_window {
              Some(window) => {

                // Pass the event to the window with focus
                let mut child_ref = window.borrow_mut();
                child_ref.handle_mouse_released(button, self.cursor_x, self.cursor_y);
              },

              None => {},
            }
          },
        };
      },

      WindowEvent::MouseWheel{device_id: _, delta, phase} => {

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

  pub fn process_user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {

    match event {

      UserEvent::Redraw(_main_win_uuid, x, y, pixmap) => {

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

      UserEvent::UpdateScroller(_main_win_uuid, scroll_layout_uuid) => {

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

  pub fn scroll_value_changed(
        &self,
        _scroll_bar_uuid: Uuid,
        orientation: Orientation,
        value: f64
  ) {

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

  pub fn set_title(&self, title: &str) {
    self.window.set_title(title);
  }

  pub fn set_visible(&self, visible: bool) {

    self.window.set_visible(visible);
  }
}
