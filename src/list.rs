use winit::{
  event::{KeyEvent, MouseButton, MouseScrollDelta, TouchPhase},
  event_loop::EventLoopProxy,
  keyboard::{KeyCode, PhysicalKey},
  window::Window,
};

use tiny_skia::{
  Color,
  Pixmap,
};

use uuid::Uuid;

use crate::ChildType;
use crate::ChildWindow;
use crate::child_window::{ContextMenuItem, LayoutType};
use crate::context_menu::ContextMenu;
use crate::text_font::TextFont;
use crate::UserEvent;
use crate::window_base::WindowBase;
use crate::window_utils::WindowUtils;

use std::{
  cell::RefCell,
  collections::HashMap,
  fmt::Debug,
  rc::Rc,
};

const LINE_PADDING: u32 = 4;

pub enum SelectionMode {
  Single,
  Multiple,
}

pub struct List {
  window_base: WindowBase,
  font: Option<TextFont>,
  char_width: u32,        // Width of a wide character
  char_height: u32,
  items: Vec<String>,
  color_text: Color,
  selection_mode: SelectionMode,
  selected: Vec<usize>,
  selected_color: Color,
  ctrl_down: bool,
  shift_down: bool,
  row_locations: HashMap<usize, f64>, // key is the row's zero-based index, value is the Y coordinate
}

impl List {

  pub fn new(
        event_loop: Rc<EventLoopProxy<UserEvent>>,
        main_win_uuid: Uuid,
  ) -> Self {

    // Load the font
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

    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_window_type("Template".to_string());

    // Create the instance
    let mut inst = Self {
      window_base: window_base,
      font: font,
      char_width: char_width,
      char_height: char_height,
      items: Vec::new(),
      color_text: Color::BLACK,
      selection_mode: SelectionMode::Single,
      selected: Vec::new(),
      selected_color: Color::from_rgba8(32, 32, 150, 255),
      ctrl_down: false,
      shift_down: false,
      row_locations: HashMap::new(),
    };

    // Build the context menu
    let event_loop_clone = event_loop.clone();
    let win_uuid = inst.get_uuid();
    let item = ContextMenuItem {
      label: "Open".to_string(),
      callback: Box::new(move || {

        // Fire the SelectionChanged event
        WindowUtils::fire_user_event(
          event_loop_clone.clone(),
          UserEvent::ProcessSelectedItems(
                main_win_uuid,
                win_uuid
          )
        );
      }),
    };
    inst.add_context_menu_item(Box::new(item));

    inst
  }

  /// Appends an item to the list
  pub fn append(&mut self, item: String) {
    self.items.push(item);

    // Redraw the pixmap
    self.draw();

    // Request a redraw of this window
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

    // Create the new pixmap
    let (width, height) = self.window_base.get_drawing_size();
    let mut pixmap = match Pixmap::new(width as u32, height as u32) {
      Some(pixmap) => pixmap,
      None => {
        return;
      },
    };

    pixmap.fill(Color::WHITE);

    // Draw the items into the pixmap
    match &self.font {

      Some(font) => {

        let mut y = 0;  // Y coordinate within the pixmap

        // Clear the row locations
        self.row_locations.clear();

        let mut item_index = self.window_base.get_y_scroll() as usize;
        let item_count = self.items.len();
        while item_index < item_count {

          let item = self.items.get(item_index).unwrap();

          // Set the drawing color based on whether this item is selected
          let mut drawing_color = self.color_text;
          if self.selected.iter().any(|&i| i == item_index) {
            drawing_color = self.selected_color;
          }

          // Draw the item's text
          let (_drawn_width, drawn_height) = font.draw_text(
            item.as_str(),
            &mut pixmap,
            0,
            y as i32,
            drawing_color,
            Color::WHITE,
            -1,
            drawing_color,
            None
          );

          // Save this row's location (just the Y coordinate)
          self.row_locations.insert(item_index, y as f64);

          // Prepare for the next item
          y += drawn_height;
          y += LINE_PADDING;

          item_index += 1;
        }
      },

      None => {},
    }

    // Save the pixmap
    self.window_base.set_pixmap(pixmap);
  }

  /// Gets the item with the specified zero-based index
  pub fn get_item_by_index(&self, index: usize) -> Option<&String> {
    self.items.get(index)
  }

  /// Gets the list of indices of selected items
  pub fn get_selected_items(&self) -> Vec<usize> {
    self.selected.clone()
  }

  /// Inserts an item into the list
  pub fn insert(&mut self, index: usize, item: String) {
    self.items.insert(index, item);

    // Redraw the pixmap
    self.draw();

    // Request a redraw of this window
    let (x, y) = self.window_base.get_location();
    WindowUtils::request_redraw(
          self.window_base.get_event_loop().clone(),
          self.window_base.get_main_win_uuid(),
          x,
          y,
          self.window_base.get_pixmap()
    );
  }

  /// Removes an item from the list
  ///
  /// Returns an Option containing the removed item or None if the index is invalid.
  pub fn remove(&mut self, index: usize) -> Option<String> {

    if index >= self.items.len() {
      return None;
    }

    let item = self.items.remove(index);

    // Request a redraw
    let (x, y) = self.window_base.get_location();
    WindowUtils::request_redraw(
          self.window_base.get_event_loop().clone(),
          self.window_base.get_main_win_uuid(),
          x,
          y,
          self.window_base.get_pixmap()
    );

    Some(item)
  }

  /// Replacing any exiting items with the ones provided
  pub fn set_items(&mut self, items: Vec<String>) {

    self.items = items;

    self.selected.clear();

    // Find the longest item, in characters
    let mut longest = 0;
    for item in &self.items {
      let len = item.len();
      if len > longest {
        longest = len;
      }
    }

    // Set the scroll ranges
    let (win_width, win_height) = self.window_base.get_drawing_size();
    self.set_x_scroll_min(0.0);
    if longest as f64 > win_width / self.char_width as f64 {
      self.set_x_scroll_max(longest as f64 - (win_width / self.char_width as f64));
    } else {
      self.set_x_scroll_max(0.0);
    }
    self.set_x_scroll(0.0);
    self.set_y_scroll_min(0.0);
    if self.items.len() as f64 > win_height / self.char_height as f64 {
      self.set_y_scroll_max(self.items.len() as f64 - (win_height / self.char_height as f64));
    } else {
      self.set_y_scroll_max(0.0);
    }
    self.set_y_scroll(0.0);

    // Redraw the pixmap
    self.draw();

    // Request a redraw of this window
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

            let mut layout_ref = layout.borrow_mut();
            layout_ref.update();
          },
        }
      },

      None => {},
    }
  }

  /// Sets the selection mode
  pub fn set_selection_mode(&mut self, mode: SelectionMode) {
    self.selection_mode = mode;
  }
}

impl Debug for List {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "List; UUID: {}, # of items: {}", self.get_uuid(), self.items.len())
   }
}

impl ChildWindow for List {

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

    if y_scroll >= 0.0 && (y_scroll as usize) < self.items.len() {

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

    let (width, _height) = self.window_base.get_drawing_size();

    width / self.char_width as f64
  }
  fn get_max_vertical_visible_items(&self) -> f64 {

    let (_width, height) = self.window_base.get_drawing_size();

    height / self.char_height as f64
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
            if row_index < self.items.len() {

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

  fn handle_mouse_movement(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }

  fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta, _phase: TouchPhase) {
    match delta {
      MouseScrollDelta::LineDelta(_, amount) => {
        self.set_y_scroll(self.window_base.get_y_scroll() + (amount * -1.0) as f64);

        // If this window is inside a ScrollLayout, tell the layout to update;
        // otherwise, redraw this window's pixmap.
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
                }
              },
            }
          },

          None => {},
        }
      },
      MouseScrollDelta::PixelDelta(_) => {
      },
    }
  }

  fn populate_context_menu(&self, context_menu_rc: Rc<RefCell<ContextMenu>>) {
    self.window_base.populate_context_menu(context_menu_rc);
  }

  fn redraw(&mut self, x: f64, y: f64, width: f64, height: f64, force: bool) -> Pixmap {

    // Save the location
    self.window_base.set_location(x, y);

    // Has the size changed?
    let (win_width, win_height) = self.window_base.get_drawing_size();
    if force || width != win_width || height != win_height {

      // Save the new size
      self.window_base.set_size(width, height);

      // Redraw the contents
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
