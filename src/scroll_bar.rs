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
  Pixmap,
  PixmapPaint,
  Rect,
  Transform,
};

use crate::button::Button;
use crate::ChildType;
use crate::ChildWindow;
use crate::child_window::{ContextMenuItem, LayoutType, Orientation};
use crate::context_menu::ContextMenu;
use crate::UserEvent;
use crate::window_base::WindowBase;
use crate::window_utils::WindowUtils;

use uuid::Uuid;

use std:: {
  cell::RefCell,
  fmt::Debug,
  rc::Rc,
};

pub static BAR_SIZE: f64 = 30.0;     // Width or height of the scroll bar, depending on orientation

#[derive(Debug)]
pub enum ScrollEvent {
  Down(u32),
  Up(u32),
}

pub struct ScrollBar {
  parent: Option<Rc<RefCell<dyn ChildWindow>>>,
  window_base: WindowBase,
  orientation: Orientation,
  decrease: Rc<RefCell<Button>>,
  increase: Rc<RefCell<Button>>,
  min_value: f64,
  max_value: f64,
  value: f64,
  color_background: Color,
  color_slide: Color,
  scrolling_callback: Option<Box<dyn Fn(Orientation, f64)>>,
  dragging_slide: bool,
}

impl ScrollBar {

  pub fn new(
        event_loop: Rc<EventLoopProxy<UserEvent>>,
        main_win_uuid: Uuid,
        orientation: Orientation,
  ) -> Self {

    let dec_image_data: Vec<u8>;
    let inc_image_data: Vec<u8>;
    match orientation {
      Orientation::Horizontal => {

        // Read in the contents of the resource files
        dec_image_data = include_bytes!("../resources/arrow_left.png").try_into().unwrap();
        inc_image_data = include_bytes!("../resources/arrow_right.png").try_into().unwrap();
      },
      Orientation::Vertical => {

        dec_image_data = include_bytes!("../resources/arrow_up.png").try_into().unwrap();
        inc_image_data = include_bytes!("../resources/arrow_down.png").try_into().unwrap();
      },
    }

    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_window_type("ScrollBar".to_string());

    let decrease = Button::new(
      event_loop.clone(),
      main_win_uuid,
      None,
      None,
      Some(dec_image_data),
      Color::from_rgba8(191, 191, 191, 255),
      move || {
      }
    );
    let increase = Button::new(
      event_loop.clone(),
      main_win_uuid,
      None,
      None,
      Some(inc_image_data),
      Color::from_rgba8(191, 191, 191, 255),
      move || {
      }
    );

    Self {
      parent: None,
      window_base: window_base,
      orientation: orientation,
      decrease: Rc::new(RefCell::new(decrease)),
      increase: Rc::new(RefCell::new(increase)),
      min_value: 0.0,
      max_value: 0.0,
      value: 0.0,
      color_background: Color::from_rgba8(191, 191, 191, 255),
      color_slide: Color::from_rgba8(64, 64, 255, 255),
      scrolling_callback: None,
      dragging_slide: false,
    }
  }

  // Draws the windows contents
  fn draw(&mut self) {

    // If the scroll bar is not visible, do nothing.
    if !self.is_visible() {
      return;
    }

    // Get the size of the window
    let (width, height) = self.window_base.get_drawing_size();
    if 0.0 == width && 0.0 == height {
      return;
    }

    // Create the new Pixmap
    let mut pixmap = match Pixmap::new(
          width as u32,
          height as u32
    ) {
      Some(pixmap) => pixmap,
      None => {
        println!("In ScrollBar::draw(), cannot create a pixmap of size {width} x {height}");
        Pixmap::new(1, 1).unwrap()
      },
    };

    // Fill the pixmap with the background color
    pixmap.fill(self.color_background);

    // Get the size and location of the slide
    let (slide_x, slide_y) = self.get_slide_location();

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

    let paint = PixmapPaint::default();

    let (win_x, win_y) = self.window_base.get_location();

    // Draw the decrease button
    let decrease_clone = self.decrease.clone();
    let mut decrease_ref = decrease_clone.borrow_mut();
    let child_pixmap = decrease_ref.redraw(
          win_x,
          win_y,
          BAR_SIZE,
          BAR_SIZE,
          false
    );
    pixmap.draw_pixmap(
        0,
        0,
        child_pixmap.as_ref(),
        &paint,
        Transform::identity(),
        None,
    );

    // Draw the increase button
    match self.orientation {
      Orientation::Horizontal => {

        // Draw the increase button
        let increase_clone = self.increase.clone();
        let mut increase_ref = increase_clone.borrow_mut();
        let child_pixmap = increase_ref.redraw(
              win_x + width - BAR_SIZE,
              win_y,
              BAR_SIZE,
              BAR_SIZE,
              false
        );
        pixmap.draw_pixmap(
            (width - BAR_SIZE) as i32,
            0,
            child_pixmap.as_ref(),
            &paint,
            Transform::identity(),
            None,
        );
      },

      Orientation::Vertical => {

        // Draw the increase button
        let increase_clone = self.increase.clone();
        let mut increase_ref = increase_clone.borrow_mut();
        let child_pixmap = increase_ref.redraw(
              win_x,
              win_y + height - BAR_SIZE,
              BAR_SIZE,
              BAR_SIZE,
              false
        );
        pixmap.draw_pixmap(
            0,
            (height - BAR_SIZE) as i32,
            child_pixmap.as_ref(),
            &paint,
            Transform::identity(),
            None,
        );
      },
    }

    self.window_base.set_pixmap(pixmap);
  }

  pub fn get_range(&self) -> (f64, f64) {
    (self.min_value, self.max_value)
  }

  fn get_slide_location(&self) -> (f64, f64) {

    let (width, height) = self.window_base.get_drawing_size();

    // Set the size of the bar between the buttons. The slide itself
    // also has a size of BAR_SIZE. Since we want the center of the slide
    // to indicate the value.
    let bar_size = match self.orientation {

      Orientation::Horizontal => width - (3.0 * BAR_SIZE),
      Orientation::Vertical => height - (3.0 * BAR_SIZE),
    };

    // Calculate the number of pixels used by each value on the bar
    let pixel_spacing: f64 = match self.orientation {
      Orientation::Horizontal => {
        if self.max_value <= self.min_value {
          0.0
        } else {
          bar_size / (self.max_value - self.min_value)
        }
      },
      Orientation::Vertical => {
        if self.max_value <= self.min_value {
          0.0
        } else {
          bar_size / (self.max_value - self.min_value)
        }
      },
    };

    // Calculate the location of the slide
    let slide_x: f64;
    let slide_y: f64;
    match self.orientation {
      Orientation::Horizontal => {
        slide_x = ((self.value - self.min_value) * pixel_spacing) + BAR_SIZE;
        slide_y = 0.0;
      },
      Orientation::Vertical => {
        slide_x = 0.0;
        slide_y = ((self.value - self.min_value) * pixel_spacing) + BAR_SIZE;
      },
    }

    (slide_x, slide_y)
  }

  pub fn get_value(&self) -> f64 {
    self.value
  }

  /// If the max value is less than or equal to the min value, the
  /// scroll bar will not be visible.
  pub fn is_visible(&self) -> bool {
    self.max_value > self.min_value
  }

  pub fn set_parent(&mut self, parent: Option<Rc<RefCell<dyn ChildWindow>>>) {
    self.parent = parent;
  }

  pub fn set_range(&mut self, minimum: f64, maximum: f64, value: f64) {
    self.min_value = minimum;
    self.max_value = maximum;

    self.set_value(value);
  }

  pub fn set_scrolling_callback(&mut self, callback: Box<dyn Fn(Orientation, f64)>) {
    self.scrolling_callback = Some(callback);
  }

  pub fn set_value(&mut self, value: f64) {
    self.value = value;

    self.draw();

    // Request a redraw
    if self.is_visible() {

      let (x, y) = self.window_base.get_location();
      WindowUtils::request_redraw(
            self.window_base.get_event_loop().clone(),
            self.window_base.get_main_win_uuid(),
            x,
            y,
            self.window_base.get_pixmap()
      );

      // Update the parent's scroll value
      match &self.parent {

        Some(parent_rc) => {

          let mut parent_ref = parent_rc.borrow_mut();

          match self.orientation {

            Orientation::Horizontal => parent_ref.set_x_scroll(value),

            Orientation::Vertical => parent_ref.set_y_scroll(value),
          }
        },

        None => {},
      }
    }

    // Fire a ScrollValueChanged event
    WindowUtils::fire_user_event(
          self.get_event_loop(),
          UserEvent::ScrollValueChanged(
            self.get_main_win_uuid(),
            self.get_uuid(),
            self.orientation,
            value
          )
    );
  }

  fn set_value_from_mouse_location(&mut self, mouse_x: f64, mouse_y: f64) {

    let (x, y) = self.get_location();

    // If the mouse is on the decrease button, do nothing
    if mouse_x <= x + BAR_SIZE && mouse_y <= y + BAR_SIZE {
      return;
    }

    let (width, height) = self.window_base.get_drawing_size();

    let new_value;
    match self.orientation {

      Orientation::Horizontal => {

        // If the mouse is on the increase button, do nothing.
        if mouse_x > x + width - BAR_SIZE && mouse_x < x + width &&
              mouse_y >= y && mouse_y < y + BAR_SIZE {
          return;
        } else {    // Clicked on the bar

          // Scroll to the value that corresponds to the location on the scroll bar.
          let percentage = (mouse_x - x) / height;
          new_value = (percentage * (self.max_value - self.min_value)) + self.min_value;

          // Call the callback
          match &self.scrolling_callback {

            Some(callback) => callback(self.orientation, new_value),
            None => {},
          }
        }
      },

      Orientation::Vertical => {

        // If the mouse is on the increase button, do nothing.
        if mouse_x >= x && mouse_x < x + width &&
              mouse_y > y + height - BAR_SIZE && mouse_y < y + height {
          return;
        } else {    // Clicked on the bar

          // Scroll to the value that corresponds to the location on the scroll bar.
          let percentage = (mouse_y - y) / height;
          new_value = (percentage * (self.max_value - self.min_value)) + self.min_value;

          // Call the callback
          match &self.scrolling_callback {

            Some(callback) => callback(self.orientation, new_value),
            None => {},
          }
        }
      },
    }

    // Save the new value
    if new_value != self.value {
      self.set_value(new_value);
    }
  }
}

impl Debug for ScrollBar {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "ScrollBar; UUID: {}, orientation: {:?}", self.get_uuid(), self.orientation)
   }
}

impl ChildWindow for ScrollBar {

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
      Some((width, _height)) => {
        match self.orientation {
          Orientation::Horizontal => Some((width, self.window_base.get_height())),
          Orientation::Vertical => None,
        }
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
    self.window_base.set_text(text);
  }

  fn handle_keyboard_pressed_event(&mut self, _event: KeyEvent) {
  }
  fn handle_keyboard_released_event(&mut self, _event: KeyEvent) {
  }

  fn handle_mouse_pressed(&mut self, button: MouseButton,
        mouse_x: f64, mouse_y: f64) {

    // Ignore right clicks
    if MouseButton::Right == button {
      return;
    }

    let (x, y) = self.window_base.get_location();
    let (width, height) = self.window_base.get_drawing_size();

    // Is it on the decrease button?
    if mouse_x > x && mouse_x < x + BAR_SIZE &&
          mouse_y >= y && mouse_y < y + BAR_SIZE
    {
      if self.value > self.min_value {
        let new_value = self.value - 1.0;

        // Call the callback
        match &self.scrolling_callback {

          Some(callback) => callback(self.orientation, new_value),
          None => {},
        }

        self.set_value(new_value);
      }

      return;
    }

    // Was the slide clicked?
    let (slide_x, slide_y) = self.get_slide_location();
    match self.orientation {

      Orientation::Horizontal => {

        if mouse_x >= x + slide_x && mouse_x <= x + slide_x + BAR_SIZE {
          self.dragging_slide = true;
          return;
        }
      },

      Orientation::Vertical => {

        if mouse_y >= y + slide_y && mouse_y <= y + slide_y + BAR_SIZE {
          self.dragging_slide = true;
          return;
        }
      },
    }

    // Was the mouse clicked on the increase button?
    let new_value;
    let mut on_increase_btn = false;
    let mut percentage = 0.0;
    match self.orientation {

      Orientation::Horizontal => {

        // Is it on the increase button?
        if mouse_x > x + width - BAR_SIZE && mouse_x < x + width &&
              mouse_y >= y && mouse_y < y + BAR_SIZE
        {
          on_increase_btn  = true;
        } else {    // Clicked on the bar

          // Calculate the percentage of the bar that is above the mouse
          percentage = (mouse_x - x) / (width - (2.0 * BAR_SIZE));
        }
      },

      Orientation::Vertical => {

        // Clicked on the increase button
        if mouse_x >= x && mouse_x < x + BAR_SIZE &&
              mouse_y > y + height - BAR_SIZE && mouse_y < y + height {
          on_increase_btn = true;
        } else {    // Clicked on the bar

          // Calculate the percentage of the bar that is above the mouse
          percentage = (mouse_y - y) / (height - (2.0 * BAR_SIZE));
        }
      },
    }

    if on_increase_btn {

      if self.value < self.max_value {
        new_value = self.value + 1.0;

        // Call the callback
        match &self.scrolling_callback {

          Some(callback) => callback(self.orientation, new_value),
          None => {},
        }
      } else {
        new_value = self.value;
      }
    } else {    // The user clicked on the bar

      // Scroll to the value that corresponds to the location on the scroll bar.
      if percentage > 1.0 {
        percentage = 1.0;
      }
      new_value = (percentage * (self.max_value - self.min_value)) + self.min_value;

      // Call the callback
      match &self.scrolling_callback {

        Some(callback) => callback(self.orientation, new_value),
        None => {},
      }
    }

    if new_value != self.value {
      self.set_value(new_value);
    }
  }
  fn handle_mouse_released(&mut self, _button: MouseButton,
        _mouse_x: f64, _mouse_y: f64) {
    self.dragging_slide = false;
  }

  fn handle_mouse_drag(&mut self, main_win_x: f64, main_win_y: f64) {

    if self.dragging_slide {
      let (x, y) = self.window_base.get_location();

      // Convert the mouse location within the main window to a location
      // relative to the scroll bar
      let mouse_x = main_win_x - x;
      let mouse_y = main_win_y - y;

      // Set the value based on the mouse location
      self.set_value_from_mouse_location(mouse_x, mouse_y);
    }
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

        let new_value = self.value + (amount * -1.0) as f64;

        // Call the callback
        match &self.scrolling_callback {

          Some(callback) => callback(self.orientation, new_value),
          None => {},
        }

        // Save the new scroll value
        self.set_value(new_value);
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

    // If the scroll bar is not visible, return a single pixel pixmap
    if !self.is_visible() {
      return Pixmap::new(1, 1).unwrap();
    }

    // Has the size changed?
    if force || width != self.window_base.get_width() ||
        height != self.window_base.get_height()
    {

      // Save the size, keeping the fixed dimension unchanged
      match self.orientation {

        Orientation::Horizontal => {
          self.window_base.set_width(width);
          self.window_base.set_height(BAR_SIZE);
        },

        Orientation::Vertical => {
          self.window_base.set_width(BAR_SIZE);
          self.window_base.set_height(height);
        },
      }

      // Update the Pixmap
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
