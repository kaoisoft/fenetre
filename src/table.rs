use winit::{
  event::{
    KeyEvent,
    MouseButton,
    MouseScrollDelta,
    TouchPhase,
  },
  event_loop::EventLoopProxy,
  window::Window,
};

use tiny_skia::{
  Color,
  Paint,
  PathBuilder,
  Pixmap,
  Rect,
  Stroke,
  Transform,
};

use uuid::Uuid;

use std::{
  cell::RefCell,
  fmt::Debug,
  rc::Rc,
};
use std::collections::HashMap;
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::ChildType;
use crate::ChildWindow;
use crate::child_window::ContextMenuItem;
use crate::context_menu::ContextMenu;
use crate::list::SelectionMode;
use crate::text_font::TextFont;
use crate::UserEvent;
use crate::window_base::WindowBase;
use crate::window_utils::WindowUtils;

struct ColumnData {
  name: String,     // Heading
  x: f32,           // X coordinate where the column begins, including starting line separator
  width: f32,       // Width of the column, in pixels
}

/// A child window which contains a table
pub struct Table {
  window_base: WindowBase,
  font: Option<TextFont>,
  columns: Vec<Box<ColumnData>>,
  row_data: Vec<Vec<String>>,
  line_color: Color,
  header_bg_color: Color,
  cell_bg_color: Color,
  selection_mode: SelectionMode,
  selected: Vec<usize>,
  selected_color: Color,
  ctrl_down: bool,
  shift_down: bool,
  row_locations: HashMap<usize, f64>, // key is the row's zero-based index, value is the Y coordinate
  char_width: u32,        // Width of a wide character
  char_height: u32,
}

impl Table {

  pub fn new(
    event_loop: Rc<EventLoopProxy<UserEvent>>,
    main_win_uuid: Uuid,
  ) -> Self {

    let mut char_width = 0;
    let mut char_height = 0;
    let font = match TextFont::new("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", 14.0) {
      Ok(mut font) => {
        char_width = font.get_max_char_width();
        char_height = font.get_max_char_height();
        Some(font)
      },
      Err(_err) => None,
    };

    // Set the default header background color
    let header_bg_color = Color::from_rgba8(50, 127, 150, 200);

    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_window_type("Table".to_string());
    window_base.set_size(200.0, 200.0);

    // Create the instance
    Self {
      window_base: window_base,
      font: font,
      columns: Vec::new(),
      row_data: Vec::new(),
      line_color: Color::BLACK,
      header_bg_color: header_bg_color,
      cell_bg_color: Color::WHITE,
      selection_mode: SelectionMode::Single,
      selected: Vec::new(),
      selected_color: Color::from_rgba8(32, 32, 150, 255),
      ctrl_down: false,
      shift_down: false,
      row_locations: HashMap::new(),
      char_width: char_width,
      char_height: char_height,
    }
  }

  pub fn add_column(&mut self, name: String) {
    let data = ColumnData {
      name: name,
      x: 0.0,
      width: 0.0,
    };
    self.columns.push(Box::new(data));
  }

  pub fn add_row(&mut self, data: Vec<String>) {
    self.row_data.push(data.clone());
  }

  pub fn clear_rows(&mut self) {

    self.selected.clear();
    self.row_data.clear();

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

  /// Clears the selected item(s)
  pub fn clear_selection(&mut self) {

    if self.selected.len() > 0 {

      self.selected.clear();

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
  }

  fn draw(&mut self) {

    let (width, height) = self.window_base.get_drawing_size();

    // Create the pixmap into which we will draw
    let mut pixmap = Pixmap::new(width as u32, height as u32).unwrap();

    // Fill the pixmap with the cell background color
    pixmap.fill(self.cell_bg_color);

    // Set the border and cell divider colors
    let mut paint = Paint::default();
    paint.set_color(self.line_color);
    paint.anti_alias = true;

    // Set the header cell background color
    let mut header_bg_paint = Paint::default();
    header_bg_paint.set_color(self.header_bg_color);
    header_bg_paint.anti_alias = true;

    // Draw the border
    WindowUtils::draw_border(&mut pixmap, width, height, &paint);

    // For each column, find the widest value (header or data)
    match &self.font {
      Some(font) => {

        for col_data in &mut self.columns {
          let (bounds_width, _bounds_height) =
              font.get_bounds(col_data.name.as_str(), None);
          col_data.width = bounds_width as f32;     // set the initial width to the width of the heading

          let mut row_index = self.window_base.get_y_scroll() as usize;
          let row_count = self.row_data.len();
          while row_index < row_count {

            let row = self.row_data.get(row_index).unwrap();

            for cell_data in row {
              let (bounds_width, _bounds_height) =
                  font.get_bounds(cell_data.as_str(), None);
              if bounds_width as f32 > col_data.width {
                col_data.width = bounds_width as f32;
              }
            }

            row_index += 1;
          }
        }
      },
      None => {},
    }

    // Calculate the padding that each column will receive
    let mut total_width: f64 = 0.0;
    for col_data in &self.columns {
      total_width += col_data.width as f64;
    }
    let padding: usize;
    if 0 == self.columns.len() {
      padding = 0;
    } else {
      padding = (width - total_width) as usize / self.columns.len();
    }

    // Draw the column vertical dividers
    let stroke = Stroke::default();   // One pixel wide
    let mut x = 0.0;
    let path = PathBuilder::from_rect(Rect::from_ltrb(x, 0.0, x, height as f32).unwrap());
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    for col_data in &mut self.columns {
      col_data.x = x;
      x += col_data.width;
      x += padding as f32;
      let path = PathBuilder::from_rect(Rect::from_ltrb(x, 0.0, x, height as f32).unwrap());
      pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    // Set the padding for the rows
    let row_padding: f64 = 10.0;
    let row_height = match &self.font {
      Some(font) => {
        let (_bounds_width, bounds_height) = font.get_bounds("Wy", None);
        bounds_height as f64 + 4.0   // add margins
      },
      None => 18.0,
    };

    // Draw the row horizontal dividers
    let mut y: f64 = 0.0;
    let mut path = PathBuilder::from_rect(Rect::from_ltrb(0.0, y as f32, width as f32, y as f32).unwrap());
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    y += row_height;
    y += row_padding;
    path = PathBuilder::from_rect(Rect::from_ltrb(0.0, y as f32, width as f32, y as f32).unwrap());
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    for _row in &mut self.row_data {
      y += row_height;
      y += row_padding;
      let path = PathBuilder::from_rect(Rect::from_ltrb(0.0, y as f32, width as f32, y as f32).unwrap());
      pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    // Display the column headings and cell data
    match &self.font {
      Some(font) => {

        self.row_locations.clear();

        let mut col_index = 0;
        for col_data in &self.columns {

          // Set the background of the heading cell
          pixmap.fill_rect(
            Rect::from_xywh(
              col_data.x,
              0.0,
              col_data.width + padding as f32,
              (row_height + row_padding) as f32
            ).unwrap(),
            &header_bg_paint,
            Transform::identity(),
            None
          );

          // Draw the heading
          let mut y = 2;
          font.draw_text(
            &col_data.name,
            &mut pixmap,
            (col_data.x as i32) + 2,
            y,
            Color::BLACK,
            self.header_bg_color,
            -1,
            Color::BLACK,
            None
          );

          // Draw the cell data
          let mut row_index = self.window_base.get_y_scroll() as usize;
          let row_count = self.row_data.len();
          while row_index < row_count {

            let row = self.row_data.get(row_index).unwrap();

            // Set the background color based on whether this item is selected
//            let mut bg_color = Color::WHITE;
            let mut bg_color = Color::BLACK;
            if self.selected.iter().any(|&i| i == row_index) {
              bg_color = self.selected_color;
            }

            // Draw this row's data for this column
            y += row_height as i32;
            y += row_padding as i32;
            font.draw_text(
              &row[col_index],
              &mut pixmap,
              (col_data.x as i32) + 2,
              y,
//              Color::BLACK,
//              bg_color,
              bg_color,
              Color::WHITE,
              -1,
              Color::BLACK,
              None
            );

            // Save this row's location (just the Y coordinate)
            if 0 == col_index {
              self.row_locations.insert(row_index, y as f64);
            }

            row_index += 1;
          }

          col_index += 1;
        }
      },
      None => {},
    }

    self.window_base.set_pixmap(pixmap);
  }

  /// Gets the data from the row with the specified zero-based index
  pub fn get_row_by_index(&self, index: usize) -> Option<&Vec<String>> {

    self.row_data.get(index)
  }

  /// Gets the list of indices of selected ros
  pub fn get_selected_rows(&self) -> Vec<usize> {
    self.selected.clone()
  }

  pub fn set_cell_bg_color(&mut self, color: Color) {
    self.cell_bg_color = color;
  }

  pub fn set_header_bg_color(&mut self, color: Color) {
    self.header_bg_color = color;
  }

  pub fn set_line_color(&mut self, color: Color) {
    self.line_color = color;
  }

  /// Sets the selection mode
  pub fn set_selection_mode(&mut self, mode: SelectionMode) {
    self.selection_mode = mode;
  }
}

impl Debug for Table {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "Table; UUID: {}, # of rows: {}", self.get_uuid(), self.row_data.len())
   }
}

impl ChildWindow for Table {

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
  }

  fn get_focused(&self) -> bool {
    self.window_base.get_focused()
  }
  fn set_focused(&mut self, focused: bool) {
    self.window_base.set_focused(focused);
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
    self.window_base.get_drawing_size()
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

    if y_scroll >= 0.0 && (y_scroll as usize) < self.row_data.len() {

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
    self.window_base.set_text(text);
  }

  fn handle_keyboard_pressed_event(&mut self, event: KeyEvent) {

    match event.physical_key {

      PhysicalKey::Code(key_code) => {
        if KeyCode::ControlLeft == key_code || KeyCode::ControlRight == key_code {
          self.ctrl_down = true;
        } else if KeyCode::ShiftLeft == key_code || KeyCode::ShiftRight == key_code {
          self.shift_down = true;
        }
      },

      PhysicalKey::Unidentified(_native_key_code) => {},
    }
  }
  fn handle_keyboard_released_event(&mut self, event: KeyEvent) {

    match event.physical_key {

      PhysicalKey::Code(key_code) => {
        if KeyCode::ControlLeft == key_code || KeyCode::ControlRight == key_code {
          self.ctrl_down = false;
        } else if KeyCode::ShiftLeft == key_code || KeyCode::ShiftRight == key_code {
          self.shift_down = false;
        }
      },

      PhysicalKey::Unidentified(_native_key_code) => {},
    }
  }

  fn handle_mouse_pressed(&mut self, button: MouseButton,
        mouse_x: f64, mouse_y: f64) {

    // Right mouse click shows the context menu
    if MouseButton::Right == button {
      self.window_base.handle_mouse_pressed(button, mouse_x, mouse_y);
      return;
    }

    match &self.font {

      Some(_font) => {

        // Convert the mouse coordinates the coordinates relative to the top of the window
        let (x, y) = self.window_base.get_location();
        let win_y = mouse_y - y;

        // Get the index of the item's row that was clicked
        let mut row_index = self.window_base.get_y_scroll() as usize;
        for _i in 0..self.row_locations.len() {
          // Start search if this row's Y coordinate is greater than the mouse location
          if self.row_locations[&row_index] > win_y {
            break;
          }
          row_index += 1;
        }
        if row_index > 0 {
          row_index -= 1;   // The mouse is on the previous row
        }

        // If this item is already selected, deselect it; otherwise,
        // save the selection.
        match self.selected.iter().position(|value| *value == row_index) {

          Some(position) => {
            self.selected.remove(position);
          },
          None => {
            if row_index < self.row_data.len() {

              match self.selection_mode {

                SelectionMode::Single => {

                  // Clear any previous selections
                  self.selected.clear();

                  // Save this index
                  self.selected.push(row_index);
                },

                SelectionMode::Multiple => {

                  if !self.ctrl_down && !self.shift_down {
                    self.selected.clear();
                  }

                  if self.shift_down {

                    // Sort the vector of indices of selected items
                    self.selected.sort();

                    // Find the the index of the selected item that is closest to
                    // but not after this item.
                    let mut done = false;
                    for i in 0..self.selected.len() {

                      if done {
                        break;
                      }

                      match self.selected.get(i) {
                        Some(index) => {
                          if *index > row_index {
                            if i > 0 {

                              let start = self.selected.get(i - 1).unwrap();

                              // Select all items after the selected item and up to and
                              // including this item.
                              let mut j = start + 1;
                              while j <= row_index {
                                self.selected.push(j);
                                j += 1;
                              }
                            }

                            done = true;
                          }
                        },
                        None => {},
                      }
                    }

                    // If no selected item was found after this one, selected all
                    // from the last selected item to this one.
                    if !done && self.selected.len() > 0 {

                      let start = self.selected.get(self.selected.len() - 1).unwrap();
                      let mut j = start + 1;
                      while j <= row_index {
                        self.selected.push(j);
                        j += 1;
                      }
                    }
                  } else if self.ctrl_down {

                    // If this item is already selected, unselect it; otherwise,
                    // select it.
                    if self.selected.contains(&row_index) {
                      // In-place filter of the Vec
                      self.selected.retain(|&i| i != row_index);
                    } else {
                      self.selected.push(row_index);
                    }
                  } else {
                    // Save this index
                    self.selected.push(row_index);
                  }
                },
              }
            }
          },
        }

        // Redraw the list so that the highlighted items is updated
        self.draw();
        WindowUtils::request_redraw(
          self.window_base.get_event_loop().clone(),
          self.window_base.get_main_win_uuid(),
          x,
          y,
          self.window_base.get_pixmap()
        );

        // Fire the SelectionChanged event
        WindowUtils::fire_user_event(
          self.window_base.get_event_loop().clone(),
          UserEvent::SelectionChanged(
            self.window_base.get_main_win_uuid(),
            self.window_base.get_uuid()
          )
        );
      },

      None => {},
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

    // Save the location of the window
    self.window_base.set_location(x, y);

    // Has the size changed?
    let (win_width, win_height) = self.window_base.get_drawing_size();
    if force || width != win_width || height != win_height {

      // Save the new size
      self.window_base.set_size(width, height);

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

    // Set the scroll ranges
    let (_win_width, win_height) = self.window_base.get_drawing_size();
    self.set_x_scroll_min(0.0);
    self.set_x_scroll_max(0.0);
    self.set_x_scroll(0.0);
    self.set_y_scroll_min(0.0);
    if self.row_data.len() as f64 > win_height / self.char_height as f64 {
      self.set_y_scroll_max(self.row_data.len() as f64 - (win_height / self.char_height as f64));
    } else {
      self.set_y_scroll_max(0.0);
    }
    self.set_y_scroll(0.0);

    self.draw();

    self.window_base.update();
  }
}
