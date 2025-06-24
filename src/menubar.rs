use winit::{
  event::{KeyEvent, MouseButton, MouseScrollDelta, TouchPhase},
  event_loop::EventLoopProxy,
  window::Window,
};

use tiny_skia::{
  Color,
  Pixmap,
  PixmapPaint,
  Transform,
};

use uuid::Uuid;

use crate::ChildType;
use crate::ChildWindow;
use crate::child_window::{ContextMenuItem, Layout, Orientation};
use crate::context_menu::ContextMenu;
use crate::label::Label;
use crate::LayoutArgs;
use crate::row_layout::RowLayout;
use crate::text_font::TextFont;
use crate::UserEvent;
use crate::window_base::WindowBase;

use std::{
  cell::RefCell,
  collections::HashMap,
  fmt::Debug,
  rc::Rc,
};

pub struct MenuBar {
  window_base: WindowBase,
  font: Option<TextFont>,
  button_height: f64,
  layout: RowLayout,
  bg_color: Color,
  action_map: HashMap<String, Box<dyn Fn()>>,  // Map of menu item actions, key is the label of the menu item
}

impl MenuBar {

  /// Creates a new instance of the MenuBar.
  ///
  /// event_loop: the application's event loop
  pub fn new(
    event_loop: Rc<EventLoopProxy<UserEvent>>,
    main_win_uuid: Uuid,
  ) -> Self {

    // Load the font
    let font: Option<TextFont>;
    let win_height;
    match TextFont::new("../resources/FreeMonoBold.ttf", 14.0) {
      Ok(text_font) => {
        let (_text_width, text_height) = text_font.get_bounds(&"Wy".to_string(), None);
        win_height = text_height;
        font = Some(text_font);
      },
      Err(err) => {
        println!("Cannot load font for MenuBar: {err}");
        win_height = 18;
        font = None;
      },
    };

    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_window_type("Menu".to_string());
    window_base.set_height((win_height * 2) as f64);    // This should be enough room for the buttons and some margins

    let mut row_layout = RowLayout::new(
            event_loop.clone(),
            main_win_uuid.clone(),
            Orientation::Horizontal,
            5.0
    );
    row_layout.set_name("MenuBar layout".to_string());

    Self {
      window_base: window_base,
      font: font,
      button_height: (win_height + 4) as f64,
      layout: row_layout,
      bg_color: Color::from_rgba8(64, 64, 255, 255),
      action_map: HashMap::new(),
    }
  }

  /// Adds an item to the menu
  ///
  /// label: menu item's display label
  /// callback: closure that will be called when a menu item is clicked
  pub fn add_item<F: Fn() + 'static>(&mut self, label: String, callback: F) {

    // Add the item to the action map
    self.action_map.insert(label.clone(), Box::new(callback));

    // Create this item
    let event_loop_clone = self.window_base.get_event_loop().clone();
    let mut item = Label::new(
          event_loop_clone.clone(),
          self.window_base.get_main_win_uuid(),
          label.clone(),
          Color::BLACK,
          self.bg_color
    );

    // Adjust the menu's height, if necessary
    match &self.font {

      Some(font) => {
        let (bounds_width, _bounds_height) = font.get_bounds(&label.clone(), None);
        item.set_max_size((bounds_width + 4) as f64, self.button_height);
      },

      None => {},
    }
    let (_item_width, item_height) = item.get_drawing_size();
    if item_height > self.window_base.get_height() {
      self.window_base.set_height(item_height * 2.0);
    }

    // Add the item to the layout
    let item_ref = Rc::new(RefCell::new(item));
    match self.layout.add_child(
          item_ref.clone(),
          LayoutArgs::None,
    ) {
      Ok(_) => {},
      Err(err) => println!("Failed to add menu item: {err}"),
    };
  }

  /// Adds a drop-down sub-menu to the menu
  pub fn add_submenu(&mut self) {
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
    pixmap.fill(self.bg_color);

    // Tell the layout to draw its contents
    let layout_pixmap = self.layout.layout(
          0.0,
          0.0,
          width,
          self.button_height
    );

    // Copy the layout's pixmap image onto the full pixmap
    pixmap.draw_pixmap(
        0,
        5,
        layout_pixmap.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    // Save the new pixmap
    self.window_base.set_pixmap(pixmap);
  }
}

impl Debug for MenuBar {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "MenuBar; UUID: {}, # of items: {}", self.get_uuid(), self.layout.get_child_count())
   }
}

impl ChildWindow for MenuBar {

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
    self.window_base.set_text(text);
  }

  fn handle_keyboard_pressed_event(&mut self, _event: KeyEvent) {
  }
  fn handle_keyboard_released_event(&mut self, _event: KeyEvent) {
  }

  fn handle_mouse_pressed(&mut self, _button: MouseButton,
        mouse_x: f64, mouse_y: f64) {

    // Get the item that was clicked
    match self.layout.get_child_at(mouse_x, mouse_y) {
      Some(item_rc) => {

        // Get the item's text
        let item_ref = item_rc.borrow();
        match item_ref.get_text() {
          Some(text) => {

            // Execute the item's associated closure
            match self.action_map.get(&text) {
              Some(func) => func(),
              None => {},
            }
          },
          None => {},
        }
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

  fn handle_mouse_wheel(&mut self, _delta: MouseScrollDelta, _phase: TouchPhase) {
  }

  fn populate_context_menu(&self, _context_menu_rc: Rc<RefCell<ContextMenu>>) {
  }

  fn redraw(&mut self, x: f64, y: f64, width: f64, height: f64, force: bool) -> Pixmap {

    // Save the location
    self.window_base.set_location(x, y);

    // Has the size changed?
    let (win_width, win_height) = self.window_base.get_drawing_size();
    if force || width != win_width || height != win_height {

      // Save the new width. The height is set in new() and should never change.
      self.window_base.set_size(width, win_height);

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
