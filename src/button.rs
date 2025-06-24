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

use image::{DynamicImage};

use uuid::Uuid;

use std::{
  cell::RefCell,
  fmt::{Debug},
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

/// Child window that is a clickable button with text and/or an image
pub struct Button {
  window_base: WindowBase,
  font: Option<TextFont>,
  image: Option<DynamicImage>,
  callback: Box<dyn Fn()>,    // function that is called when the button is clicked
  color_text: Color,
  color_background: Color,
  internal_padding: f64,
}

impl Button {

  /// If both a PNG file path and text are specified, the text will be drawn over the image.
  pub fn new<F: Fn() + 'static> (
    event_loop: Rc<EventLoopProxy<UserEvent>>,
    main_win_uuid: Uuid,
    text: Option<String>,
    image_path: Option<String>,
    image_data: Option<Vec<u8>>,
    bg_color: Color,
    callback: F,
  ) -> Self {

    let mut width = 1.0;
    let mut height = 1.0;

    // If a PNG file was specified, use it to determine the size of the button.
    let mut image: Option<DynamicImage> = None;
    match image_path {
      Some(filename) => {

        match WindowUtils::load_image(&filename) {
          Ok(img) => {
            width = img.width() as f64;
            height = img.height() as f64;
            image = Some(img);
          },
          Err(err) => println!("Could not load button image {filename}: {err}"),
        }
      },
      None => {},
    }

    // If image data was specified, use it to determine the size of the button.
    match image_data {
      Some(data) => {

        match WindowUtils::load_image_from_resource(&data) {
          Ok(img) => {
            width = img.width() as f64;
            height = img.height() as f64;
            image = Some(img);
          },
          Err(err) => println!("Could not load button image: {err}"),
        }
      },
      None => {},
    }

    // Load the font
    let font = match TextFont::new("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", DEFAULT_FONT_SIZE) {
      Ok(font) => Some(font),
      Err(_err) => None,
    };

    // If text was specified, use it to determine the size of the button. If a PNG
    // file was also specified, use the biggest to determine the button's size.
    let mut text_width = 0.0;
    let mut text_height = 0.0;
    match text {
      Some(ref text) => {

        // Get the size of the drawn text
        let (bounds_width, bounds_height) = match &font {
          Some(font) => font.get_bounds(text, None),
          None => (1, 1),
        };

        // Add margins
        text_width = bounds_width as f64 + 10.0;
        text_height = bounds_height as f64 + 4.0;
      },
      None => {},
    }

    // Use the largest size
    if width < text_width as f64 {
      width = text_width as f64;
    }
    if height < text_height as f64 {
      height = text_height as f64;
    }

    // Create the WindowBase instance
    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_size(width, height);
    match text {
      Some(text) => window_base.set_text(text),
      None => {},
    }

    // Use the size as the minimum size for the button
    window_base.set_min_size(width, height);
    window_base.set_max_size(width + INTERNAL_MARGIN, height + INTERNAL_MARGIN);
    window_base.set_window_type("Button".to_string());

    let mut inst = Self {
      window_base: window_base,
      font: font,
      image: image,
      callback: Box::new(callback),
      color_text: Color::BLACK,
      color_background: bg_color,
      internal_padding: 8.0,
    };

    inst.draw();

    inst
  }

  fn draw(&mut self) {

    let pixmap = match self.image {
      Some(_) => self.redraw_image(
            self.window_base.get_x(),
            self.window_base.get_y(),
            self.window_base.get_width(),
            self.window_base.get_height()
        ),
      None => self.redraw_text(
            self.window_base.get_x(),
            self.window_base.get_y(),
            self.window_base.get_width(),
            self.window_base.get_height()
        ),
    };
    self.window_base.set_pixmap(pixmap);
  }

  fn redraw_image(&mut self, x: f64, y: f64, _width: f64, _height: f64) -> Pixmap {

    match &self.image {

      Some(image) => {

        // Save the location
        self.window_base.set_location(x, y);

        // Draw the image into a Pixmap
        WindowUtils::create_image_pixmap(&image)
      },
      None => {

        // The image was not loaded, so create an empty pixmap
        Pixmap::new(self.window_base.get_width() as u32, self.window_base.get_height() as u32).unwrap()
      },
    }
  }

  fn redraw_text(&mut self, x: f64, y: f64, width: f64, height: f64) -> Pixmap {

    // Ensure that the size isn't smaller than the minimum size
    let (min_width, min_height) = match self.window_base.get_min_size() {
      Some((width, height)) => (width, height),
      None => (1.0, 1.0),
    };
    let mut new_width = width;
    if new_width < min_width {
      new_width = min_width;
    }
    new_width += 2.0;   // Extra padding on the right
    let mut new_height = height;
    if new_height < min_height {
      new_height = min_height;
    }
    new_height += 2.0;   // Extra padding on the bottom

    // Ensure that the size isn't bigger than the maximum size
    let (max_width, max_height) = match self.window_base.get_max_size() {
      Some((width, height)) => (width, height),
      None => (1.0, 1.0),
    };
    if new_width > max_width {
      new_width = max_width;
    }
    if new_height > max_height {
      new_height = max_height;
    }

    // Save the location and size
    self.window_base.set_location(x, y);
    self.window_base.set_size(new_width, new_height);

    // Create the pixmap into which we will draw
    let mut pixmap = Pixmap::new(new_width as u32, new_height as u32).unwrap();

    // Determine the background color based on the enabled status
    let bg_color;
    if self.window_base.get_enabled() {
      bg_color = self.color_background;
    } else {
      // If the button does not have focus, gray the text color slightly.
      let mut red = self.color_background.red() - 0.25;
      if red < 0.0 {
        red = 0.0;
      }
      let mut green = self.color_background.green() - 0.25;
      if green < 0.0 {
        green = 0.0;
      }
      let mut blue = self.color_background.blue() - 0.25;
      if blue < 0.0 {
        blue = 0.0;
      }
      bg_color = match Color::from_rgba(red, green, blue, self.color_background.alpha()) {
        Some(color) => color,
        None => self.color_background,
      };
    }

    // Fill the pixmap with the background color
    pixmap.fill(bg_color);

    // Draw the text or image
    match &self.window_base.get_text() {

      Some(text) => match &self.font {
        Some(font) => {

          // Center the text
          let (text_width, text_height) = font.get_bounds(text, None);
          let mut x = 0;
          if new_width as u32 > text_width {         // button is wider than the text
            x = (new_width as u32 - text_width) / 2;
          }
          let mut y = 0;
          if new_height as u32 > text_height {       // button is taller that the text
            y = (new_height as u32 - text_height) / 2;
          }
          font.draw_text(
            text.as_str(),
            &mut pixmap,
            x as i32,
            y as i32,
            self.color_text,
            bg_color,
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

  pub fn set_background_color(&mut self, color: Color) {
    self.color_background = color;
  }

  pub fn set_text_color(&mut self, color: Color) {
    self.color_text = color;
  }
}

impl Debug for Button {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    let text = match self.window_base.get_text() {
      Some(text) => text,
      None => "".to_string(),
    };

    write!(fmt, "Button; UUID: {}, text: {}", self.get_uuid(), text)
   }
}

impl ChildWindow for Button {

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

  fn get_drawing_size(&self) -> (f64, f64) {
    self.window_base.get_drawing_size()
  }

  fn get_max_size(&self) -> Option<(f64, f64)> {
    let width = self.window_base.get_width();

    let height: f64 = match &self.window_base.get_text() {

      Some(text) => {

        match &self.font {
          Some(font) => {
            // Find the size of the text
            let (_, text_height) = font.get_bounds(&text, None);

            // Add internal padding
            text_height as f64 + self.internal_padding
          },
          None => 0.0,
        }
      },
      None => {

        // Get the height of the image
        match &self.image {
          Some(image) => image.width() as f64,
          None => 0.0,
        }
      },
    };

    Some((width, height))
  }
  fn set_max_size(&mut self, width: f64, height: f64) {
    self.window_base.set_max_size(width, height);
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

  fn handle_mouse_drag(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }
  fn handle_mouse_drag_start(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }
  fn handle_mouse_drag_end(&mut self, _main_win_x: f64, _main_win_y: f64) {
  }

  fn handle_mouse_pressed(&mut self, button: MouseButton,
      _mouse_x: f64, _mouse_y: f64) {

    // If the user clicked the left button, call the callback. If the user clicked
    // the right button, display the tooltip, if there is one.
    match button {

      MouseButton::Left => {
        if self.window_base.get_enabled() {
          (self.callback)();
        }
      },

      MouseButton::Right => {},

      _ => {},
    }
  }
  fn handle_mouse_released(&mut self, _button: MouseButton,
        _mouse_x: f64, _mouse_y: f64) {
  }

  fn handle_mouse_movement(&mut self, _x: f64, _y: f64) {

    // If there is tooltip text, display the text in a pop-up
    match self.window_base.get_tooltip_text() {

      Some(text) => {
        WindowUtils::fire_user_event(
              self.window_base.get_event_loop(),
              UserEvent::ShowToolTip(
                    self.window_base.get_main_win_uuid(),
                    self.window_base.get_uuid(),
                    text
              )
        );
      },

      None => {},
    }
  }

  fn handle_mouse_wheel(&mut self, _delta: MouseScrollDelta, _phase: TouchPhase) {
  }

  fn populate_context_menu(&self, context_menu_rc: Rc<RefCell<ContextMenu>>) {
    self.window_base.populate_context_menu(context_menu_rc);
  }

  fn redraw(&mut self, x: f64, y: f64, _width: f64, _height: f64, _force: bool) -> Pixmap {

    // Save the location
    self.window_base.set_location(x, y);

    self.window_base.get_pixmap().clone()
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
