use winit::{
  event::{
    ElementState,
    KeyEvent,
    MouseButton,
    MouseScrollDelta,
    TouchPhase,
  },
  event_loop::EventLoopProxy,
  keyboard::{Key, NamedKey},
  window::Window,
};

use tiny_skia::{
  Color,
  Paint,
  Pixmap,
};

use uuid::Uuid;

use std::{
  cell::RefCell,
  fmt::Debug,
  rc::Rc,
};

use crate::ChildType;
use crate::ChildWindow;
use crate::child_window::ContextMenuItem;
use crate::context_menu::ContextMenu;
use crate::text_font::TextFont;
use crate::UserEvent;
use crate::window_base::WindowBase;
use crate::window_utils::WindowUtils;

/// A child window which contains a single line of editable text
pub struct LineEdit {
  window_base: WindowBase,
  font: Option<TextFont>,
  char_width: u32,
  insertion_point: usize,
  caret_color: Color,
  internal_padding: f64,
  modified: bool,
}

impl LineEdit {

  pub fn new(
    event_loop: Rc<EventLoopProxy<UserEvent>>,
    main_win_uuid: Uuid,
    text: String,
  ) -> Self {

    // Load the font
    let mut char_width = 20;
    let mut char_height = 18;
    let font = match TextFont::new("../resources/FreeMonoBold.ttf", 14.0) {
      Ok(font) => {
        let (bounds_width, _bounds_height) = font.get_bounds("W", None);
        char_width = bounds_width;
        let (_bounds_width, bounds_height) = font.get_bounds("Wy", None);
        char_height = bounds_height;
        Some(font)
      },
      Err(_err) => None,
    };

    // Calculate the size of the window
    let height: f64 = char_height as f64 + 4.0;

    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_window_type("LineEdit".to_string());
    window_base.set_size(200.0, height);
    window_base.set_min_size(30.0, height);
    window_base.set_max_size(-1.0, height);    // max width is never used within LineEdit
    window_base.set_text(text.clone());

    // Set the insert point to be at the end of the string
    let insertion_point = text.len();

    // Create the instance
    Self {
      window_base: window_base,
      font: font,
      char_width: char_width,
      insertion_point: insertion_point,
      caret_color: Color::from_rgba8(64, 64, 255, 255),
      internal_padding: 4.0,
      modified: false,
    }
  }

  fn draw(&mut self) {

    let (_unused_width, height) = self.window_base.get_drawing_size();
    let width = self.window_base.get_width();

    // Create the pixmap into which we will draw
    let mut pixmap = match Pixmap::new(width as u32, height as u32) {
      Some(pixmap) => pixmap,
      None => {
        println!("Cannot create pixmap with size {width} x {height}, using default size 100 x 30");
        Pixmap::new(100, 30).unwrap()
      },
    };

    // Fill the pixmap with the background color
    pixmap.fill(self.window_base.get_background_color());

    // Set the border's color
    let mut paint = Paint::default();
    paint.set_color_rgba8(200, 0, 0, 220);
    paint.anti_alias = true;

    // Draw the border
    WindowUtils::draw_border(&mut pixmap, width, height, &paint);

    // Display the text
    match self.window_base.get_text() {

      Some(text) => {

        let x_scroll = self.window_base.get_x_scroll();

        if self.window_base.get_focused() {
          match &self.font {
            // Indicate focus by highlighting the insertion point
            Some(font) => {
              font.draw_text(
                &text[(x_scroll as usize)..],
                &mut pixmap,
                2,
                2,
                Color::BLACK,
                Color::WHITE,
                (self.insertion_point as i64) - (x_scroll as i64),
                self.caret_color,
                Some(self.char_width)
              );
            },
            None => {},
          };
        } else {
          match &self.font {
            // Indicate lack of focus by not highlighting the insertion point
            Some(font) => {
              font.draw_text(
                &text[(x_scroll as usize)..],
                &mut pixmap,
                2,
                2,
                Color::BLACK,
                Color::WHITE,
                -1,             // tells TextFont to not draw the caret
                Color::BLACK,
                Some(self.char_width)
              );
            },
            None => {},
          };
        }
      },

      None => {},
    }

    self.window_base.set_pixmap(pixmap);
  }
  
  /// Returns true if the text has been modified
  pub fn is_modified(&self) -> bool {
    self.modified
  }

  /// Sets the modified flag
  pub fn set_modified(&mut self, modified: bool) {
    self.modified = modified;
  }
}

impl Debug for LineEdit {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    match self.window_base.get_text() {
      Some(text) => write!(fmt, "LineEdit; UUID: {}, text: '{}'", self.get_uuid(), text),
      None => write!(fmt, "LineEdit; UUID: {}, text: ''", self.get_uuid()),
    }
   }
}

impl ChildWindow for LineEdit {

  fn add_context_menu_item(&mut self, item: Box<ContextMenuItem>) {
    self.window_base.add_context_menu_item(item);
  }
  fn add_context_menu_separator(&mut self) {
    self.window_base.add_context_menu_separator();
  }

  fn created_window(&self, _window: Window) {
  }

  fn get_uuid(&self) -> Uuid {
    self.window_base.get_uuid()
  }
  fn set_uuid(&mut self, uuid: Uuid) {
    self.window_base.set_uuid(uuid);
  }
  fn get_main_win_uuid(&self) -> Uuid {
    self.window_base.get_main_win_uuid()
  }

  fn get_pixmap(&self) -> Pixmap {
    self.window_base.get_pixmap()
  }

  fn get_name(&self) -> String {
    self.window_base.get_name()
  }
  fn set_name(&mut self, name: String) {
    self.window_base.set_name(name);
  }

  fn get_window_type(&self) -> String {
    self.window_base.get_window_type()
  }
  fn set_window_type(&mut self, window_type: String) {
    self.window_base.set_window_type(window_type);
  }

  fn get_event_loop(&self) -> Rc<EventLoopProxy<UserEvent>> {
    self.window_base.get_event_loop()
  }
  fn set_event_loop(&mut self, event_loop: Rc<EventLoopProxy<UserEvent>>) {
    self.window_base.set_event_loop(event_loop);
  }

  fn get_enabled(&self) -> bool {
    self.window_base.get_enabled()
  }
  fn set_enabled(&mut self, enabled: bool) {
    self.window_base.set_enabled(enabled);

    // Request a redraw
    let (x, y) = self.window_base.get_location();
    WindowUtils::request_redraw(
          self.window_base.get_event_loop().clone(),
          self.window_base.get_main_win_uuid(),
          x,
          y,
          self.window_base.get_pixmap()
    );
  }

  fn get_focused(&self) -> bool {
    self.window_base.get_focused()
  }
  fn set_focused(&mut self, focused: bool) {

    self.window_base.set_focused(focused);

    self.draw();

    // Request a redraw
    let (x, y) = self.window_base.get_location();
    WindowUtils::request_redraw(
          self.window_base.get_event_loop().clone(),
          self.window_base.get_main_win_uuid(),
          x,
          y,
          self.window_base.get_pixmap()
    );
  }

  fn get_location(&self) -> (f64, f64) {
    self.window_base.get_location()
  }
  fn set_location(&mut self, x: f64, y: f64) {
    self.window_base.set_location(x, y);
  }

  fn get_layout_location(&self) -> (f64, f64) {
    self.window_base.get_layout_location()
  }
  fn set_layout_location(&mut self, x: f64, y: f64) {
    self.window_base.set_layout_location(x, y);
  }

  fn get_width(&self) -> f64 {
    self.window_base.get_width()
  }
  fn set_width(&mut self, width: f64) {
    self.window_base.set_width(width);
  }
  fn get_height(&self) -> f64 {
    self.window_base.get_height()
  }
  fn set_height(&mut self, height: f64) {
    self.window_base.set_height(height);
  }

  fn get_min_size(&self) -> Option<(f64, f64)> {
    self.window_base.get_min_size()
  }
  fn set_min_size(&mut self, width: f64, height: f64) {
    self.window_base.set_min_size(width, height);
  }

  fn get_max_size(&self) -> Option<(f64, f64)> {

    match self.window_base.get_max_size() {

      Some((width, mut height)) => {

        // Set the max height to the height of the text in the current font
        height = match &self.font {
          Some(font) => {
            // Find the size of the text
            let (_, text_height) = match self.window_base.get_text() {
              Some(text) => font.get_bounds(&text, Some(self.char_width)),
              None => (0, 0),
            };

            // Add internal padding
            text_height as f64 + self.internal_padding
          },
          None => 1.0
        };

        Some((width, height))
      },
      None => None,
    }
  }
  fn set_max_size(&mut self, width: f64, height: f64) {
    self.window_base.set_max_size(width, height);
  }

  fn get_drawing_size(&self) -> (f64, f64) {

    let (_width, height) = self.window_base.get_drawing_size();

    (self.window_base.get_width(), height)    // Use the saved width, which is set in redraw()
  }

  fn get_x_scroll(&self) -> f64 {
    self.window_base.get_x_scroll()
  }
  fn set_x_scroll(&mut self, x_scroll: f64) {
    self.window_base.set_x_scroll(x_scroll);
  }
  fn get_x_scroll_min(&self) -> f64 {
    self.window_base.get_x_scroll_min()
  }
  fn set_x_scroll_min(&mut self, value: f64) {
    self.window_base.set_x_scroll_min(value);
  }
  fn get_x_scroll_max(&self) -> f64 {
    self.window_base.get_x_scroll_max()
  }
  fn set_x_scroll_max(&mut self, value: f64) {
    self.window_base.set_x_scroll_max(value);
  }

  fn get_y_scroll(&self) -> f64 {
    self.window_base.get_y_scroll()
  }
  fn set_y_scroll(&mut self, y_scroll: f64) {
    self.window_base.set_y_scroll(y_scroll);
  }
  fn get_y_scroll_min(&self) -> f64 {
    self.window_base.get_y_scroll_min()
  }
  fn set_y_scroll_min(&mut self, value: f64) {
    self.window_base.set_y_scroll_min(value);
  }
  fn get_y_scroll_max(&self) -> f64 {
    self.window_base.get_y_scroll_max()
  }
  fn set_y_scroll_max(&mut self, value: f64) {
    self.window_base.set_y_scroll_max(value);
  }

  fn get_max_horizontal_visible_items(&self) -> f64 {
    0.0
  }
  fn get_max_vertical_visible_items(&self) -> f64 {
    0.0
  }

  fn get_text(&self) -> Option<String> {
    self.window_base.get_text()
  }
  fn set_text(&mut self, text: String) {
    
    self.modified = false;
    
    self.window_base.set_text(text);

    self.draw();

    let (x, y) = self.window_base.get_location();
    WindowUtils::request_redraw(
          self.window_base.get_event_loop().clone(),
          self.window_base.get_main_win_uuid(),
          x,
          y,
          self.window_base.get_pixmap()
    );
  }

  fn handle_keyboard_pressed_event(&mut self, event: KeyEvent) {

    match self.window_base.get_text() {

      Some(mut text) => {

        let mut text_modified = false;

        // Ignore key released events
        if ElementState::Released == event.state {
          return;
        }

        let mut processed = false;
        match event.logical_key {

          // Caret movement
          Key::Named(NamedKey::ArrowLeft) => {
            if self.insertion_point > 0 {
              self.insertion_point -= 1;
            }

            // Set the flag in order to scroll, if necessary.
            text_modified = true;

            processed = true;
          },
          Key::Named(NamedKey::ArrowRight) => {
            if self.insertion_point < text.len() {
              self.insertion_point += 1;
            }

            // Set the flag in order to scroll, if necessary.
            text_modified = true;

            processed = true;
          },
          Key::Named(NamedKey::Home) => {
            self.insertion_point = 0;

            // Set the flag in order to scroll, if necessary.
            text_modified = true;

            processed = true;
          },
          Key::Named(NamedKey::End) => {

            // Move the insertion point to the end of the text
            self.insertion_point = text.len();

            // Set the flag in order to scroll, if necessary.
            text_modified = true;

            processed = true;
          },

          // Editing keys
          Key::Named(NamedKey::Backspace) => {

            // If the insertion point is already at the beginning, there is nothing to do.
            if self.insertion_point > 0 {

              // Move the insertion point back one character
              self.insertion_point -= 1;

              // Delete this character
              let mut part1 = text[0..self.insertion_point].to_string();
              part1.push_str(&text[self.insertion_point + 1..]);
              text = part1;

              text_modified = true;
            }

            processed = true;
          },
          Key::Named(NamedKey::Delete) => {

            // If the insertion point is past the end of the text, there is nothing to do.
            if self.insertion_point < text.len() {

              // Delete this character
              let mut part1 = text[0..self.insertion_point].to_string();
              part1.push_str(&text[self.insertion_point + 1..]);
              text = part1;

              text_modified = true;
            }

            processed = true;
          },

          // All others
          _ => {
          },
        };

        // If the event has not been processed yet, try to insert the text.
        if !processed {
          match event.text {
            Some(event_text) => {

              // Ignore carriage returns and linefeeds
              if event_text != "\r" && event_text != "\n" {

                // Insert the entered text at the insertion point
                text.insert_str(self.insertion_point, event_text.as_str());
                self.insertion_point += event_text.len();

                text_modified = true;
              }
            },
            None => {},
          }
        }

        if text_modified {
          
          self.modified = true;
          
          self.window_base.set_text(text);

          // Calculate the width of the window in characters
          let width = self.window_base.get_width();
          let win_char_width = width / self.char_width as f64;

          // If the caret is no longer visible, scroll the window so that it is.
          let x_scroll = self.window_base.get_x_scroll();
          if (self.insertion_point as f64) < x_scroll {
            self.window_base.set_x_scroll(self.insertion_point as f64);
          } else if (self.insertion_point as f64) + x_scroll >= win_char_width {
            self.window_base.set_x_scroll((self.insertion_point as f64) - (win_char_width - 1.0));
          }
        }

        // Redraw this window's contents within the main window
        let x = self.window_base.get_x();
        let y = self.window_base.get_y();
        self.draw();
        WindowUtils::request_redraw(
              self.window_base.get_event_loop().clone(),
              self.window_base.get_main_win_uuid(),
              x,
              y,
              self.window_base.get_pixmap()
        );
      },

      None => self.insertion_point = 0,
    }
  }
  fn handle_keyboard_released_event(&mut self, _event: KeyEvent) {
  }

  fn handle_mouse_pressed(&mut self, button: MouseButton,
      mouse_x: f64, _mouse_y: f64) {

    if button == MouseButton::Left {

      match &self.font {

        Some(font) => {

          match self.window_base.get_text() {

            Some(text) => {

              let text_len = text.len();

              // Find the character that starts closest to, but not after, the mouse location
              let (text_width, _text_height) =
                  font.get_bounds(text.as_str(), Some(self.char_width));
              let char_width; 
              if 0 < text_len {
                char_width = text_width / text_len as u32;
              } else {
                char_width = text_width;
              }
              let mut insertion_point = ((mouse_x - self.window_base.get_x()) / char_width as f64) as usize;
              if insertion_point > text_len {
                insertion_point = text_len;
              }

              // Set the insertion point and caret
              self.insertion_point = insertion_point;

              self.draw();
              WindowUtils::request_redraw(
                    self.window_base.get_event_loop().clone(),
                    self.window_base.get_main_win_uuid(),
                    self.window_base.get_x(),
                    self.window_base.get_y(),
                    self.window_base.get_pixmap()
              );
            },

            None => {},
          }
        },

        None => {},
      }
    }
  }
  fn handle_mouse_released(&mut self, _button: MouseButton,
        _mouse_x: f64, _mouse_y: f64) {
  }

  fn handle_mouse_drag(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }
  fn handle_mouse_drag_start(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }
  fn handle_mouse_drag_end(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }

  fn handle_mouse_movement(&mut self, _x: f64, _y: f64) {
  }

  fn handle_mouse_wheel(&mut self, _delta: MouseScrollDelta, _phase: TouchPhase) {
  }

  fn populate_context_menu(&self, context_menu_rc: Rc<RefCell<ContextMenu>>) {
    self.window_base.populate_context_menu(context_menu_rc);
  }

  fn redraw(&mut self, x: f64, y: f64, width: f64, height: f64, force: bool) -> Pixmap {

    // Save the width of the window, but not the height. It is set in new()
    // based on the text size.
    self.window_base.set_width(width);

    // Save the location of the window
    self.window_base.set_location(x, y);

    // Ensure that the height doesn't exceed the maximum height
    let new_height = match self.window_base.get_max_size() {
      Some((_max_width, max_height)) => {

        // Adjust the height
        if height > max_height {
          max_height
        } else {
          height
        }
      },
      None => height,
    };

    // Has the size changed?
    let (win_width, win_height) = self.window_base.get_drawing_size();
    if force || width != win_width || new_height != win_height {

      // Save the new size
      self.window_base.set_size(width, new_height);

      // Redraw the window's contents
      self.draw();
    }

    self.window_base.get_pixmap()
  }

  fn get_background_color(&self) -> Color {
    self.window_base.get_background_color()
  }
  fn set_background_color(&mut self, color: Color) {
    self.window_base.set_background_color(color);
  }

  fn get_parent(&self) -> Option<ChildType> {
    self.window_base.get_parent()
  }
  fn set_parent(&mut self, parent: Option<ChildType>) {
    self.window_base.set_parent(parent);
  }

  fn get_tooltip_text(&self) -> Option<String> {
    self.window_base.get_tooltip_text()
  }
  fn set_tooltip_text(&mut self, text: String) {
    self.window_base.set_tooltip_text(text);
  }

  fn update(&mut self) {

    self.draw();

    self.window_base.update();
  }
}
