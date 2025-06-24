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
  Pixmap,
};

use uuid::Uuid;

use std::{
  cell::RefCell,
  collections::BTreeMap,
  fmt::Debug,
  rc::Rc,
};

use crate::ChildType;
use crate::ChildWindow;
use crate::child_window::{ContextMenuItem, LayoutType};
use crate::context_menu::ContextMenu;
use crate::text_font::TextFont;
use crate::UserEvent;
use crate::window_base::WindowBase;
use crate::window_utils::WindowUtils;

const LINE_PADDING: u32 = 2;

/// A child window which contains multiple lines of editable text.
///
/// Note: tabs are automatically converted to spaces
pub struct MultiLineEdit {
  window_base: WindowBase,
  font: Option<TextFont>,
  char_width: u32,
  char_height: u32,
  insertion_line: usize,  // Line that the caret is on
  insertion_point: usize, // Character that the caret is on
  caret_color: Color,
  lines: Vec<String>,
  top_line: usize,        // zero-based index of the first visible line
  tab_size: usize,        // Tab size, in characters
  line_locations: BTreeMap<u64, usize>,  // key is Y coordinate of line, value is line's index
  modified: bool,
}

impl MultiLineEdit {

  pub fn new(
    event_loop: Rc<EventLoopProxy<UserEvent>>,
    main_win_uuid: Uuid,
    tab_size: usize
  ) -> Self {

    // Load the font
    let mut char_width = 20;
    let mut char_height = 18;
    let font = match TextFont::new("../resources/FreeMonoBold.ttf", 14.0) {
      Ok(font) => {
        let (bounds_width, _bounds_height) = font.get_bounds("W", None);
        char_width = bounds_width;
        let (_bounds_width, bounds_height) = font.get_bounds("Wy", None);
        char_height = bounds_height + LINE_PADDING;  // add padding
        Some(font)
      },
      Err(_err) => None,
    };

    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_window_type("MultiLineEdit".to_string());

    // Create a single, empty line for a new file
    let mut lines: Vec<String> = Vec::new();
    lines.push("".to_string());

    // Create the instance
    Self {
      window_base: window_base,
      font: font,
      char_width: char_width,
      char_height: char_height,
      insertion_line: 0,
      insertion_point: 0,
      caret_color: Color::from_rgba8(64, 64, 255, 255),
      lines: lines,
      top_line: 0,
      tab_size: tab_size,
      line_locations: BTreeMap::new(),
      modified: false,
    }
  }

  fn calculate_visible_lines(&self) -> usize {

    let (_width, height) = self.window_base.get_drawing_size();

    (height / self.char_height as f64) as usize
  }

  fn delete_char_at_caret(&mut self) {

    // If the insertion point is past the end of the line,
    // append the next line to this one.
    let mut current_line = self.lines[self.insertion_line].clone();
    if self.insertion_point < current_line.len() {

      // Delete this character
      let mut part1 = current_line[0..self.insertion_point].to_string();
      part1.push_str(&current_line[self.insertion_point + 1..]);
      self.lines[self.insertion_line] = part1;
    } else if self.insertion_line < self.lines.len() - 1 {

      let text = self.lines[self.insertion_line + 1].clone();
      current_line += text.as_str();
      self.lines[self.insertion_line] = current_line;
      self.lines.remove(self.insertion_line + 1);
    }
  }

  fn draw(&mut self) {

    let (width, height) = self.window_base.get_drawing_size();

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

    // Draw each visible line
    match &self.font {

      Some(font) => {

        let visible_count = self.calculate_visible_lines();

        let x_scroll = self.window_base.get_x_scroll();

        // Clear the Y coordinates map
        self.line_locations.clear();

        let mut y: i32 = 0;
        let mut line_index = self.top_line;
        let line_count = self.lines.len();
        let mut count = 0;
        while line_index < line_count && count < visible_count {

          // Get the size of this line's text
          let line = &self.lines[line_index];
          let visible_line;
          if line.len() > x_scroll as usize {
            visible_line = &line[(x_scroll as usize)..];
          } else {
            visible_line = "";
          }

          // Does this line contain the caret?
          let caret_index;
          if self.insertion_line == line_index {
            caret_index = (self.insertion_point as i64) - (x_scroll as i64);

            // Draw this line's text
            if 0 != visible_line.len() {    // Not a blank line
              font.draw_text(
                visible_line,
                &mut pixmap,
                0,
                y,
                Color::BLACK,
                Color::WHITE,
                caret_index,
                self.caret_color,
                Some(self.char_width)
              );
            } else {

              // Draw a single space as the text
              font.draw_text(
                " ",
                &mut pixmap,
                0,
                y,
                Color::BLACK,
                Color::WHITE,
                caret_index,
                self.caret_color,
                Some(self.char_width)
              );
            }
          } else {
            caret_index = -1;

            // Draw this line's text
            font.draw_text(
              visible_line,
              &mut pixmap,
              0,
              y,
              Color::BLACK,
              Color::WHITE,
              caret_index,
              self.caret_color,
              Some(self.char_width)
            );
          }

          // Add this line's Y coordinate to the map
          self.line_locations.insert(y as u64, line_index);

          // Update the Y coordinate for the next line
          y += self.char_height as i32;
          if y >= height as i32 {
            break;
          }

          // Move to the next line
          line_index += 1;
          count += 1;
        }
      },

      None => {},
    }

    self.window_base.set_pixmap(pixmap);
  }

  fn fire_caret_moved_event(&self) {
    WindowUtils::fire_user_event(
          self.window_base.get_event_loop(),
          UserEvent::CaretMoved(
                self.window_base.get_uuid(),
                self.insertion_line,
                self.insertion_point
          )
    );
  }

  // Inserts the appropriate number of spaces to represent a tab at the
  // specified file location.
  fn insert_tab(&mut self, line_index: usize, col_index: usize) {

    // Determine the next tab stop after the specified location
    let tab_stop = ((col_index / self.tab_size) + 1) * self.tab_size;

    // Create the string of spaces
    let spaces = " ".repeat(tab_stop - col_index);

    // Insert the spaces into the line
    self.insert_text(line_index, col_index, &spaces, true, true);
  }

  fn insert_text(
        &mut self,
        line_index: usize,
        col_index: usize,
        text: &str,
        move_caret: bool,
        redraw: bool
  ) {

    // Insert the text at the specified file location
    let mut line = self.lines[line_index].clone();
    line.insert_str(col_index, text);
    self.lines[line_index] = line;

    if move_caret {
      self.move_caret(
            line_index,
            col_index + text.len(),
            redraw
      );
    }
  }

  /// Returns true if the text has been modified
  pub fn is_modified(&self) -> bool {
    self.modified
  }

  fn move_caret(&mut self, line: usize, col: usize, redraw: bool) {

    // Save the line index
    self.insertion_line = line;

    // Get the number of visible lines
    let visible_count = self.calculate_visible_lines();

    // If the line is not visible scroll so that it is.
    let mut scroll_vertically = false;
    if self.insertion_line < self.top_line {
      self.top_line = self.insertion_line;
      scroll_vertically = true;
    } else if self.insertion_line >= self.top_line + visible_count - 1 {
      self.top_line = self.insertion_line + 1 - visible_count;
      scroll_vertically = true;
    }

    // Save the character index. If the index is beyond the end of the
    // line, adjust it.
    if col > self.lines[self.insertion_line].len() {
      self.insertion_point = self.lines[self.insertion_line].len();
    } else {
      self.insertion_point = col;
    }

    // Calculate the width of the window in characters
    let width = self.window_base.get_width();
    let win_char_width = width / self.char_width as f64;

    // If the caret is no longer visible, scroll the window so that it is.
    let x_scroll = self.window_base.get_x_scroll();
    if (self.insertion_point as f64) < x_scroll {
      self.window_base.set_x_scroll(self.insertion_point as f64);
    } else if (self.insertion_point as f64) + x_scroll > win_char_width {
      self.window_base.set_x_scroll((self.insertion_point as f64) - win_char_width);
    }
    if scroll_vertically {
      self.set_y_scroll(self.top_line as f64);
    }

    // Fire the CaretMoved event
    self.fire_caret_moved_event();

    // Redraw this window's contents within the main window
    if redraw {

      // If this window is inside a ScrollLayout, tell the layout to update;
      // otherwise, redraw this window's pixmap.
      let mut layout_updated = false;
      match self.get_parent() {

        Some(child_type) => {

          match child_type {

            ChildType::Window(_child_window) => {},

            ChildType::Layout(layout) => {

              let layout_ref = layout.borrow();
              if LayoutType::ScrollLayout == layout_ref.get_type() {
                WindowUtils::fire_user_event(
                      self.get_event_loop(),
                      UserEvent::UpdateScroller(
                            self.get_main_win_uuid(),
                            layout_ref.get_uuid()
                      )
                );

                layout_updated = true;
              }
            },
          }
        },

        None => {},
      }

      if !layout_updated {

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
      }
    }
  }

  /// Sets the modified flag
  pub fn set_modified(&mut self, modified: bool) {
    self.modified = modified;
  }
}

impl Debug for MultiLineEdit {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "MultiLineEdit; UUID: {}, # of lines of text: {}",
        self.get_uuid(), self.lines.len())
  }
}

impl ChildWindow for MultiLineEdit {

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
    self.window_base.get_max_size()
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

    if self.top_line > y_scroll as usize {
      self.top_line = y_scroll as usize;
    } else {
      let diff = y_scroll as usize - self.top_line;
      self.top_line += diff;
      if self.top_line >= self.lines.len() {
        self.top_line = self.lines.len() - 1;
      }
    }
    self.window_base.set_y_scroll(y_scroll);

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

    let mut text = "".to_string();
    for line in &self.lines {

      if 0 < text.len() {
        text += "\n";
      }

      text += &line;
    }

    Some(text)
  }
  fn set_text(&mut self, text: String) {

    self.modified = false;
    
    let mut longest_line = 0;

    // Remove existing lines
    self.lines.clear();

    // Save the lines
    for line in text.split('\n') {
      let line_str = line.to_string();
      if line_str.len() > longest_line {
        longest_line = line_str.len();
      }
      self.lines.push(line_str);
    }

    // Get the size of the window
    let (pixel_width, pixel_height) = self.window_base.get_drawing_size();

    // Get the size of the window in characters
    let char_width = (pixel_width / self.char_width as f64) as usize;
    let char_height = (pixel_height / self.char_height as f64) as usize;

    // Use the length of the longest line and the width of the window
    // set the horizontal scroll range.
    self.set_x_scroll_min(0.0);
    if longest_line > char_width {
      self.set_x_scroll_max(((longest_line - char_width) as f64) - 1.0);
    } else {
      self.set_x_scroll_max(0.0);
    }
    self.set_x_scroll(0.0);

    // Use the number of lines and the height of the window
    // set the vertical scroll range.
    self.set_y_scroll_min(0.0);
    if self.lines.len() > char_height {
      self.set_y_scroll_max(((self.lines.len() - char_height) as f64) - 1.0);
    } else {
      self.set_y_scroll_max(0.0);
    }
    self.set_y_scroll(0.0);

    self.draw();

    let (x, y) = self.window_base.get_location();
    WindowUtils::request_redraw(
          self.window_base.get_event_loop().clone(),
          self.window_base.get_main_win_uuid(),
          x,
          y,
          self.window_base.get_pixmap()
    );

    // If this window has a ScrollLayout parent, tell it to redraw also.
    match self.window_base.get_parent() {

      Some(parent) => {

        match parent {

          ChildType::Window(_) => {},

          ChildType::Layout(layout) => {

            let layout_ref = layout.borrow();
            if LayoutType::ScrollLayout == layout_ref.get_type() {
              WindowUtils::fire_user_event(
                    self.window_base.get_event_loop(),
                    UserEvent::UpdateScroller(
                          self.window_base.get_main_win_uuid(),
                          layout_ref.get_uuid()
                    )
              );
            }
          },
        }
      },

      None => {},
    }
  }

  fn handle_keyboard_pressed_event(&mut self, event: KeyEvent) {

    if self.lines.len() > 0 {

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
            self.move_caret(self.insertion_line, self.insertion_point - 1, true);
          }

          processed = true;
        },
        Key::Named(NamedKey::ArrowRight) => {
          let line = &self.lines[self.insertion_line];
          if self.insertion_point <= line.len() {
            self.move_caret(self.insertion_line, self.insertion_point + 1, true);
          }

          processed = true;
        },
        Key::Named(NamedKey::ArrowUp) => {
          if self.insertion_line > 0 {

            // Move the caret to the previous line
            self.move_caret(self.insertion_line - 1, self.insertion_point, true);
          }

          processed = true;
        },
        Key::Named(NamedKey::ArrowDown) => {

          if self.insertion_line < self.lines.len() - 1 {

            // Move the caret to the next line
            self.move_caret(self.insertion_line + 1, self.insertion_point, true);
          }

          processed = true;
        },
        Key::Named(NamedKey::Home) => {

          // Move the insertion point to the beginnning of the text
          self.move_caret(self.insertion_line, 0, true);

          processed = true;
        },
        Key::Named(NamedKey::End) => {

          // Move the insertion point to the end of the text
          let line = &self.lines[self.insertion_line];
          self.move_caret(self.insertion_line, line.len(), true);

          processed = true;
        },

        // Editing keys
        Key::Named(NamedKey::Backspace) => {

          // If the insertion point is already at the beginning of the file,
          // there is nothing to do.
          if self.insertion_line != 0 || self.insertion_point != 0 {

            // Move the insertion point back one character
            if self.insertion_point == 0 {  // Caret is at the beginning of the line

              // Move to the end of the previous line
              self.move_caret(
                    self.insertion_line - 1,
                    self.lines[self.insertion_line - 1].len(),
                    false
              );
            } else {
              self.move_caret(self.insertion_line, self.insertion_point - 1, false);
            }

            // Delete this character
            self.delete_char_at_caret();

            // Text was modified after the caret was moved, so set the flag
            // so that the text will be redrawn.
            text_modified = true;
          }

          processed = true;
        },
        Key::Named(NamedKey::Delete) => {

          self.delete_char_at_caret();

          // Text was modified but the caret was not moved, so set the flag
          // so that the text will be redrawn.
          text_modified = true;

          processed = true;
        },
        Key::Named(NamedKey::Enter) => {

          // Move the text from the caret position on the current line to a new line
          let mut current_line = self.lines[self.insertion_line].clone();
          let newline = current_line[self.insertion_point..].to_string();
          current_line = current_line[0..self.insertion_point].to_string();
          self.lines[self.insertion_line] = current_line;

          // Insert the new line after the current one
          self.lines.insert(self.insertion_line + 1, newline);

          // Move the caret to the beginning of the new line
          self.move_caret(self.insertion_line + 1, 0, true);

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

            if 0 < event_text.len() && '\t' == event_text.chars().nth(0).unwrap() {

              self.insert_tab(self.insertion_line, self.insertion_point);
            } else {

              self.insert_text(
                    self.insertion_line,
                    self.insertion_point,
                    &event_text,
                    true,
                    true
              );
            }
            
            text_modified = true;
          },
          None => {},
        }
      }

      if text_modified {

        self.modified = true;
        
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
      }
    }
  }
  fn handle_keyboard_released_event(&mut self, _event: KeyEvent) {
  }

  fn handle_mouse_pressed(&mut self, button: MouseButton,
      mouse_x: f64, mouse_y: f64) {

    if button == MouseButton::Left {

      // Adjust the Y coordinate so that it is relative to the top
      // of the window.
      let adjusted_y = (mouse_y - self.window_base.get_y()) as u64;

      // Find the line index that is closest to the Y coordinate
      let mut prev_index: usize = 0;
      for (y_coord, line_index) in &self.line_locations {

        // If this line's index is after the adjusted mouse location,
        // stop looking.
        if *y_coord > adjusted_y {
          break;
        }

        // Save the index
        prev_index = *line_index;
      }

      // Calculate the character that was clicked
      let mut char_index = (mouse_x / self.char_width as f64) + self.window_base.get_x_scroll();
      if char_index > self.lines[prev_index].len() as f64 {
        char_index = self.lines[prev_index].len() as f64;
      }

      self.move_caret(prev_index, char_index as usize, true);

      self.fire_caret_moved_event();
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

    // Save the new size
    self.window_base.set_size(width, height);

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
