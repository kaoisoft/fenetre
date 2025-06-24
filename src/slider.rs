use winit::{
  event::{
    KeyEvent,
    MouseButton,
    MouseScrollDelta,
    MouseScrollDelta::{LineDelta, PixelDelta},
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

use crate::ChildType;
use crate::ChildWindow;
use crate::child_window::{ContextMenuItem, Orientation};
use crate::context_menu::ContextMenu;
use crate::text_font::TextFont;
use crate::UserEvent;
use crate::window_base::WindowBase;
use crate::window_utils::WindowUtils;

const BAR_SIZE: f64 = 20.0;     // Width or height of the slider bar, depending on orientation
const EXTRA_SPACE: f64 = 5.0;   // Extra space at each end of the slider so that
                                // half of the slide can extend beyond the first/last value

/// Child window that is a slider control with a range of values
pub struct Slider {
  window_base: WindowBase,
  orientation: Orientation,
  font: Option<TextFont>,
  color_text: Color,
  color_background: Color,
  color_slide: Color,
  min_value: f64,
  max_value: f64,
  value: f64,
  show_ticks: bool,
  tick_steps: f64,
  show_labels: bool,
}

impl Slider {

  pub fn new(
    event_loop: Rc<EventLoopProxy<UserEvent>>,
    main_win_uuid: Uuid,
    orientation: Orientation,
  ) -> Self {

    // Load the font
    let font = match TextFont::new("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", 14.0) {
      Ok(font) => Some(font),
      Err(_err) => None,
    };

    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_window_type("Slider".to_string());
    window_base.set_min_size(BAR_SIZE, BAR_SIZE);

    let mut inst = Self {
      window_base: window_base,
      orientation: orientation,
      font: font,
      color_text: Color::BLACK,
      color_background: Color::from_rgba8(191, 191, 191, 255),
      color_slide: Color::from_rgba8(64, 64, 255, 255),
      min_value: 1.0,
      max_value: 10.0,
      value: 1.0,
      show_ticks: false,
      tick_steps: 1.0,
      show_labels: false,
    };

    inst.draw();

    inst
  }

  // Converts pixel value within the scroll bar to a value within the range
  //
  // The pixel value is either the X coordinate (for horizontal sliders)
  // or the Y coordinate (for vertical sliders)
  fn convert_pixel_to_value(&self, pixel: f64) -> f64 {

    let range: f64 = self.max_value - self.min_value;
    let max_clickable: f64 = match self.orientation {
      Orientation::Horizontal => self.window_base.get_width(),
      Orientation::Vertical => self.window_base.get_height(),
    };

    (pixel * range) / (max_clickable as f64)
  }

  fn draw(&mut self) {

    let (width, height) = self.window_base.get_drawing_size();

    // Create the pixmap
    let mut pixmap = match Pixmap::new(
          width as u32,
          height as u32
    ) {
      Some(pixmap) => pixmap,
      None => {
        println!("In Slider::draw(), cannot create a pixmap of size {width} x {height}");
        Pixmap::new(1, 1).unwrap()
      },
    };

    // Fill the pixmap with the background color
    pixmap.fill(self.color_background);

    // If we are drawing ticks, set the background for the ticks to WHITE
    if self.show_ticks {

      let mut x_ticks: f32 = 0.0;
      let mut y_ticks: f32 = 0.0;
      let (ticks_width, ticks_height) = match self.orientation {

        Orientation::Horizontal => {
          x_ticks = 1.0;
          (width, BAR_SIZE)
        },

        Orientation::Vertical => {
          y_ticks = 1.0;
          (BAR_SIZE, height)
        },
      };

      let rect = Rect::from_xywh(
            x_ticks,
            y_ticks,
            ticks_width as f32,
            ticks_height as f32
      ).unwrap();
      let mut tick_bg_paint = Paint::default();
      tick_bg_paint.set_color(Color::WHITE);

      pixmap.fill_rect(rect, &tick_bg_paint, Transform::identity(), None);
    }

    // Get the size and location of the slide
    let (mut slide_x, mut slide_y) = self.get_slide_location();

    // If we are drawing ticks, move the slider' bar and slide to make room.
    if self.show_ticks {

      match self.orientation {

        Orientation::Horizontal => {
          slide_y += BAR_SIZE;
        },

        Orientation::Vertical => {
          slide_x += BAR_SIZE;
        },
      }
    }

    // Draw the slide
    let rect = match Rect::from_xywh(slide_x as f32, slide_y as f32, BAR_SIZE as f32, BAR_SIZE as f32) {
      Some(rect) => rect,
      None => {
        println!("In Slider::draw(), cannot create a Rect from ({slide_x},{slide_y}) with size {BAR_SIZE} x {BAR_SIZE}");
        return;
      },
    };
    let mut paint = Paint::default();
    paint.set_color(self.color_slide);
    pixmap.fill_rect(
          rect,
          &paint,
          Transform::identity(),
          None
    );

    // Draw the ticks, if requested.
    if self.show_ticks {

      let paint = Paint::default();
      let stroke = Stroke::default();

      match self.orientation {

        Orientation::Horizontal => {

          // Calculate the distance between each tick
          let tick_distance = width / (self.max_value - self.min_value);

          // Draw each tick
          let mut tick_value = self.min_value;
          while tick_value <= self.max_value {

            // Calculate the tick's location
            let x = (tick_value * tick_distance) as f32;
            let y = 0.0 as f32;

            // Create the Path the defines the tick's line
            let mut pb = PathBuilder::new();
            pb.move_to(x, y);
            pb.line_to(x, y + BAR_SIZE as f32);
            let path = pb.finish().unwrap();

            // Draw the tick
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

            // Move to the next tick's value
            tick_value += self.tick_steps;
          }
        },

        Orientation::Vertical => {

          // Calculate the distance between each tick
          let tick_distance = height / (self.max_value - self.min_value);

          // Draw each tick
          let mut tick_value = self.min_value;
          while tick_value <= self.max_value {

            // Calculate the tick's location
            let x = 0.0 as f32;
            let y = (tick_value * tick_distance) as f32;

            // Create the Path the defines the tick's line
            let mut pb = PathBuilder::new();
            pb.move_to(x, y);
            pb.line_to(x + BAR_SIZE as f32, y);
            let path = pb.finish().unwrap();

            // Draw the tick
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

            // Move to the next tick's value
            tick_value += self.tick_steps;
          }
        },
      }
    }

    // Save the new pixmap
    self.window_base.set_pixmap(pixmap);
  }

  pub fn get_range(&self) -> (f64, f64) {
    (self.min_value, self.max_value)
  }

  fn get_slide_location(&self) -> (f64, f64) {

    let (width, height) = self.window_base.get_size();

    // Calculate the number of pixels between each value
    let pixel_spacing: f64 = match self.orientation {
      Orientation::Horizontal => {
        if self.max_value <= self.min_value {
          0.0
        } else {
          (width - (EXTRA_SPACE * 2.0)) / (self.max_value - self.min_value)
        }
      },
      Orientation::Vertical => {
        if self.max_value <= self.min_value {
          0.0
        } else {
          (height - (EXTRA_SPACE * 2.0)) / (self.max_value - self.min_value)
        }
      },
    };

    // Calculate the location of the slide
    let slide_x: f64;
    let slide_y: f64;
    match self.orientation {
      Orientation::Horizontal => {
        slide_x = (self.value - self.min_value) * pixel_spacing;
        slide_y = 0.0;
      },
      Orientation::Vertical => {
        slide_x = 0.0;
        slide_y = (self.value - self.min_value) * pixel_spacing;
      },
    }

    (slide_x, slide_y)
  }

  fn get_slide_size(&self) -> (f64, f64) {

    let (width, height) = self.window_base.get_size();

    // Calculate the size of the slide
    let slide_width: f64;
    let slide_height: f64;
    match self.orientation {
      Orientation::Horizontal => {
        slide_width = 11.0;
        slide_height = height;
      },
      Orientation::Vertical => {
        slide_width = width;
        slide_height = 11.0;
      },
    }

    (slide_width, slide_height)
  }

  pub fn get_value(&self) -> f64 {
    self.value
  }

  pub fn set_background_color(&mut self, color: Color) {
    self.color_background = color;
  }

  pub fn set_range(&mut self, min: f64, max: f64, value: f64) {

    if min != self.min_value || max != self.max_value || value != self.value {
      self.min_value = min;
      self.max_value = max;
      self.value = value;

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

  pub fn set_slide_color(&mut self, color: Color) {
    self.color_slide = color;
  }

  pub fn set_text_color(&mut self, color: Color) {
    self.color_text = color;
  }

  pub fn set_tick_steps(&mut self, steps: f64) {

    self.tick_steps = steps;

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

  pub fn set_value(&mut self, value: f64) {

    if value != self.value {
      self.value = value;

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

      // Fire an event
      WindowUtils::fire_user_event(
          self.window_base.get_event_loop().clone(),
          UserEvent::SliderValueChange(
                value
          )
      );
    }
  }

  pub fn show_ticks(&mut self, flag: bool) {

    // If the flag isn't changing, do nothing.
    if flag == self.show_ticks {
      return;
    }

    // Save the flag
    self.show_ticks = flag;

    if self.show_ticks {

      // If a font has been loaded, increase the size of the window to
      // provide room for drawing the ticks.
      match &self.font {

        Some(_font) => {

          let (width, height) = self.window_base.get_drawing_size();

          match self.orientation {

            Orientation::Horizontal => self.window_base.set_min_size(width, height + BAR_SIZE as f64),

            Orientation::Vertical => self.window_base.set_min_size(width + BAR_SIZE as f64, height),
          }
        },

        None => {},
      }
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
}

impl Debug for Slider {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "Slider; UUID: {}, value: {}, orientation: {:?}",
        self.get_uuid(),
        self.value,
        self.orientation
    )
   }
}

impl ChildWindow for Slider {

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

  fn handle_mouse_pressed(&mut self, button: MouseButton,
      mouse_x: f64, mouse_y: f64) {

    if button == MouseButton::Left {

      // If the mouse was clicked on the slide, do nothing.
      let adjusted_mouse_x = mouse_x - self.window_base.get_x();
      let adjusted_mouse_y = mouse_y - self.window_base.get_y();
      let (slide_x, slide_y) = self.get_slide_location();
      let (slide_width, slide_height) = self.get_slide_size();
      if adjusted_mouse_x >= slide_x && adjusted_mouse_x <= slide_x + slide_width &&
            adjusted_mouse_y >= slide_y && adjusted_mouse_y <= slide_y + slide_height {
        return;
      }

      // Scroll to the value that corresponds to the spot that was clicked.
      let mut adjusted_mouse;
      match self.orientation {

        Orientation::Horizontal => {

          // Adjust the X coordinate for the scroll bar's location
          adjusted_mouse = adjusted_mouse_x;
          if adjusted_mouse < 0.0 {
            adjusted_mouse = 0.0;
          }
        },

        Orientation::Vertical => {

          // Adjust the Y coordinate for the scroll bar's location
          adjusted_mouse = adjusted_mouse_y;
          if adjusted_mouse < 0.0 {
            adjusted_mouse = 0.0;
          }
        },
      }

      // Calculate the value based on the mouse location within the slider
      let value: f64 = self.convert_pixel_to_value(adjusted_mouse);

      // Set the new value
      self.set_value(value);
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
      LineDelta(_horz_amount, vert_amount) => {

        // Reverse the sign on the amount, so scrolling up/left is negative.
        let scroll_amount = vert_amount * -1.0;

        // If the user is scrolling up and the image is already scrolled to the top/left, do nothing.
        if self.value != 0.0 || scroll_amount > 0.0 {

          // Scroll 10 pixels for each wheel position
          let mut new_value = self.value + (scroll_amount * 10.0) as f64;
          if new_value < 0.0 {
            new_value = 0.0;
          }
          self.set_value(new_value);

          // Fire an event
          WindowUtils::fire_user_event(
              self.window_base.get_event_loop().clone(),
              UserEvent::SliderValueChange(
                    new_value
              )
          );
        }
      },
      PixelDelta(_logical_position) => {},
    }
  }

  fn populate_context_menu(&self, context_menu_rc: Rc<RefCell<ContextMenu>>) {
    self.window_base.populate_context_menu(context_menu_rc);
  }

  fn redraw(&mut self, x: f64, y: f64, mut width: f64, mut height: f64, force: bool) -> Pixmap {

    // Save the location
    self.window_base.set_location(x, y);

    // Calculate the fixed dimension
    let mut fixed_dimension = BAR_SIZE;
    if self.show_labels {
      fixed_dimension += BAR_SIZE;
    }
    if self.show_ticks {
      fixed_dimension += BAR_SIZE;
    }

    // Update the size of the fixed dimension
    match self.orientation {

      Orientation::Horizontal => height = fixed_dimension,

      Orientation::Vertical => width = fixed_dimension,
    }

    // If the size has changed, save the new size and redraw the window
    let (current_width, current_height) = self.window_base.get_drawing_size();
    if force || width != current_width || height != current_height {
      self.window_base.set_size(width, height);
      self.draw();
    }

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
