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

// Used for internal margins
const INTERNAL_MARGIN: f64 = 4.0;
const DEFAULT_FONT_SIZE: f32 = 14.0;

/// A child window which contains a non-editable, single line of text
pub struct Label {
  window_base: WindowBase,
  font: Option<TextFont>,
  text_color: Color,
  bg_color: Color,
  internal_padding: f64,
}

impl Label {

  pub fn new(
    event_loop: Rc<EventLoopProxy<UserEvent>>,
    main_win_uuid: Uuid,
    text: String,
    text_color: Color,
    bg_color: Color
  ) -> Self {

    // Load the font
    let (text_width, text_height);
    let font = match TextFont::new("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", DEFAULT_FONT_SIZE) {
      Ok(font) => {
        let (bound_width, bound_height) = font.get_bounds(text.as_str(), None);
        text_width = bound_width;
        text_height = bound_height;
        Some(font)
      },
      Err(_err) => {
        text_width = 100;
        text_height = 18;
        None
      },
    };

    // Calculate the size of the window
    let width: f64 = text_width as f64 + 2.0;
    let height: f64 = text_height as f64 + 4.0;

    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_window_type("Label".to_string());
    window_base.set_text(text);
    window_base.set_width(width);
    window_base.set_height(height);
    window_base.set_max_size(width, height);
    window_base.set_min_size(width, height);

    // Create the instance
    let mut inst = Self {
      window_base: window_base,
      font: font,
      text_color: text_color,
      bg_color: bg_color,
      internal_padding: INTERNAL_MARGIN,
    };

    inst.draw();

    inst
  }

  fn draw(&mut self) {

    let (width, height) = self.window_base.get_drawing_size();

    // Create the pixmap into which we will draw
    let mut pixmap = Pixmap::new(width as u32, height as u32).unwrap();

    // Fill the pixmap with the background color
    pixmap.fill(self.bg_color);

    // Display the text
    match &self.font {
      Some(font) => {

        match self.window_base.get_text() {

          Some(text) => {

            let (bounds_width, bounds_height) = font.get_bounds(&text, None);

            // Center the text
            font.draw_text(
              text.as_str(),
              &mut pixmap,
              ((width - bounds_width as f64) / 2.0) as i32,
              ((height - bounds_height as f64) / 2.0) as i32,
              self.text_color,
              self.bg_color,
              -1,
              Color::BLACK,
              None
            );
          },

          None => {},
        }
      },
      None => {},
    };

    self.window_base.set_pixmap(pixmap)
  }
}

impl Debug for Label {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "Label; UUID: {}, text: {:?}", self.get_uuid(), self.window_base.get_text())
   }
}

impl ChildWindow for Label {

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

    match self.window_base.get_max_size() {

      Some((width, mut height)) => {

        height = match &self.font {

          Some(font) => {

            match self.window_base.get_text() {

              Some(text) => {

                // Find the size of the text
                let (_, text_height) = font.get_bounds(text.as_str(), None);

                // Add internal padding
                text_height as f64 + self.internal_padding
              },

              None => 1.0,
            }
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

    self.window_base.set_text(text.clone());

    // Adjust the size to accomodate the new text
    match &self.font {

      Some(font) => {

        let (bound_width, bound_height) = font.get_bounds(text.as_str(), None);

        let width: f64 = bound_width as f64 + 2.0;
        let height: f64 = bound_height as f64 + 4.0;

        self.set_width(width);
        self.set_height(height);
        self.set_max_size(width, height);
        self.set_min_size(width, height);
      },

      None => {},
    }

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

  fn handle_mouse_movement(&mut self, _x: f64, _y: f64) {
  }

  fn handle_mouse_wheel(&mut self, _delta: MouseScrollDelta, _phase: TouchPhase) {
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

      // Save the size
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
