use winit::{
  event::{KeyEvent, MouseButton, MouseScrollDelta, TouchPhase},
  event_loop::EventLoopProxy,
  window::Window,
};

use tiny_skia::{Color, Pixmap, PixmapPaint, Transform};

use uuid::Uuid;

use crate::ChildType;
use crate::ChildWindow;
use crate::child_window::ContextMenuItem;
use crate::context_menu::ContextMenu;
use crate::text_font::TextFont;
use crate::UserEvent;
use crate::window_base::WindowBase;
use crate::window_utils::WindowUtils;

use std::{
  cell::RefCell,
  fmt::Debug,
  rc::Rc,
};

pub struct StatusBar {
  window_base: WindowBase,
  font: Option<TextFont>,
  color_background: Color,
  color_text: Color,
}

impl StatusBar {

  pub fn new(
        event_loop: Rc<EventLoopProxy<UserEvent>>,
        main_win_uuid: Uuid,
        bg_color: Color,
  ) -> Self {

    // Load the font
    let mut char_height = 18;
    let font = match TextFont::new("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", 14.0) {
      Ok(font) => {
        let (_bounds_width, bounds_height) = font.get_bounds("Wy", None);
        char_height = bounds_height;
        Some(font)
      },
      Err(_err) => {
        None
      },
    };

    let window_height = char_height as f64 + 4.0;
    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_window_type("Template".to_string());
    window_base.set_size(10.0, window_height);
    window_base.set_max_size(10.0, window_height);

    Self {
      window_base: window_base,
      font: font,
      color_background: bg_color,
      color_text: Color::from_rgba8(0, 0, 0, 255),
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

    // Redraw the message
    let message_pixmap = self.redraw_message(0.0, 0.0, width, height);

    // Copy the message's pixmap image onto the full pixmap
    pixmap.draw_pixmap(
        2,
        2,
        message_pixmap.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    // Save the pixmap
    self.window_base.set_pixmap(pixmap);
  }

  /// Gets the message currently being displayed
  pub fn get_message(&self) -> Option<String> {
    self.window_base.get_text()
  }

  fn redraw_message(&mut self, _x: f64, _y: f64, width: f64, height: f64) -> Pixmap {

    // Create the pixmap into which we will draw
    let mut pixmap = Pixmap::new(width as u32, height as u32).unwrap();

    // Fill the pixmap with the background color
    pixmap.fill(self.color_background);

    // Draw the text
    match &self.window_base.get_text() {

      Some(text) => match &self.font {
        Some(font) => {

          // Place the text at the left edge
          let x = 2;
          let y = 2;
          font.draw_text(
            text.as_str(),
            &mut pixmap,
            x as i32,
            y as i32,
            self.color_text,
            self.color_background,
            -1,
            Color::BLACK,
            None
          );
        },
        None => {},
      },
      None => {},
    }

    pixmap
  }

  pub fn set_message(&mut self, message: String) {

    self.window_base.set_text(message);

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
}

impl Debug for StatusBar {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    let message = match self.window_base.get_text() {
      Some(text) => text,
      None => "".to_string(),
    };

    write!(fmt, "StatusBar; UUID: {}, message: {}", self.get_uuid(), message)
   }
}

impl ChildWindow for StatusBar {

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
        _mouse_x: f64, _mouse_y: f64) {
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

  fn populate_context_menu(&self, context_menu_rc: Rc<RefCell<ContextMenu>>) {
    self.window_base.populate_context_menu(context_menu_rc);
  }

  fn redraw(&mut self, x: f64, y: f64, width: f64, _height: f64, _force: bool) -> Pixmap {
    // Save the location
    self.window_base.set_location(x, y);

    // Has the size changed?
    let (win_width, win_height) = self.window_base.get_drawing_size();

    // Update the maximum width
    let (_max_width, max_height) = self.window_base.get_max_size().unwrap(); // we can safely use unwrap(), because we know that it has been set
    self.window_base.set_max_size(width, max_height);

    if width != win_width {

      // Save the new width, but don't change the height
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
