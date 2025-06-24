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
  IntRect,
  Pixmap,
  PixmapPaint,
  PixmapRef,
  Transform,
};

use uuid::Uuid;

use image::{
  ColorType,
  DynamicImage,
};

use std::{
  cell::RefCell,
  fmt::Debug,
  rc::Rc,
};

use crate::ChildType;
use crate::ChildWindow;
use crate::child_window::{ContextMenuItem, LayoutType};
use crate::context_menu::ContextMenu;
use crate::UserEvent;
use crate::window_base::WindowBase;
use crate::window_utils::WindowUtils;

#[derive(Debug, PartialEq)]
pub enum DisplayType {
  Actual,         // display the image in its actual size
  ScaleLarger,    // scale the image larger is necessary, but not smaller
  ScaleSmaller,   // scale the image smaller is necessary, but not smaller
  ScaleAlways,    // scale the image larger or smaller as necessary
}

/// A child window which displays an image
pub struct ImageView {
  window_base: WindowBase,
  image_width: f64,
  image_height: f64,
  image: DynamicImage,
  full_pixmap: Pixmap,
  image_path: Option<String>,
  display_type: DisplayType,
}

impl ImageView {

  pub fn new(
    event_loop: Rc<EventLoopProxy<UserEvent>>,
    main_win_uuid: Uuid,
    display_type: DisplayType,
  ) -> Self {

    let mut window_base = WindowBase::new(event_loop.clone(), main_win_uuid);
    window_base.set_window_type("Imageview".to_string());

    // Create the instance
    Self {
      window_base: window_base,
      image_width: 1.0,
      image_height: 1.0,
      image: DynamicImage::new(1, 1, ColorType::Rgba8),
      full_pixmap: Pixmap::new(1, 1).unwrap(),
      image_path: None,
      display_type: display_type,
    }
  }

  // Copies a portion of an image
  fn copy_image_section(&mut self, x: f64, y: f64, width: f64, height: f64) -> Pixmap {

    // Copy the requested portion of the image into the Pixmap
    if
          0.0 <= self.image_width - x &&
          0.0 <= self.image_height - y
    {

      // Calculate the size of the visible portion
      let mut subimage_width = self.image_width - x;
      if subimage_width > width {
        subimage_width = width;
      }
      let mut subimage_height = self.image_height - y;
      if subimage_height > height {
        subimage_height = height;
      }

      let rect = match IntRect::from_xywh(
              x as i32,
              y as i32,
              subimage_width as u32,
              subimage_height as u32
      ) {
        Some(rect) => rect,
        None => {
          println!("In ImageView::copy_image_section(), cannot create an IntRect from ({},{}) with size {} x {}",
                x, y, subimage_width, subimage_height);
          return self.window_base.get_pixmap().clone()
        },
      };

      // Get the requested portion of the image
      match self.full_pixmap.clone_rect(rect) {
        Some(pixmap) => pixmap,
        None => {
          println!("Cannot create sub-image pixmap from rectangle at ({x}, {y}) with size {subimage_width} x {subimage_height} from image of size {} x {}",
                self.image_width, self.image_height);
          Pixmap::new(1, 1).unwrap()
        },
      }
    } else {
      self.window_base.get_pixmap().clone()
    }
  }

  // Draw the image into the window
  fn draw(&mut self) {

    let mut pixmap = self.window_base.get_pixmap();

    // If the window's size has changed, create a new pixmap
    let (drawing_width, drawing_height) = self.window_base.get_drawing_size();
    if drawing_width != pixmap.width() as f64 || drawing_height != pixmap.height() as f64 {
      pixmap = match Pixmap::new(drawing_width as u32, drawing_height as u32) {
        Some(pixmap) => pixmap,
        None => {
          println!("Cannot create a pixmap of size {drawing_width} x {drawing_height}");
          Pixmap::new(1, 1).unwrap()
        },
      };
    }

    // Erase the pixmap
    pixmap.fill(Color::BLACK);

    // Get the upper left corner of the visible portion of the image
    let x = self.window_base.get_x_scroll();
    let y = self.window_base.get_y_scroll();

    // Draw based on the display type
    let sub_pixmap = match self.display_type {

      DisplayType::Actual => {

        // Copy the visible portion of the image into the Pixmap
        self.copy_image_section(x, y, drawing_width, drawing_height)
      },

      DisplayType::ScaleLarger => {

        // If the image is smaller than the display size, scale it up;
        // Otherwise, display it unchanged.
        if self.image_width - x < drawing_width ||
              self.image_height - y < drawing_height {

          // Create a scaled version of the image that fills the window
          match self.scale_by_size(
                x as f32,
                y as f32,
                drawing_width as u32,
                drawing_height as u32
          ) {

            Some(pixmap) => pixmap,

            None => {
              println!("Cannot scale pixmap");
              Pixmap::new(1, 1).unwrap()
            },
          }
        } else {
          self.copy_image_section(x, y, drawing_width, drawing_height)
        }
      },

      DisplayType::ScaleSmaller => {

        // If the image is larger than the display size, scale it down;
        // Otherwise, display it unchanged.
        if self.image_width - x > drawing_width ||
              self.image_height - y > drawing_height {

          // Create a scaled version of the image that fills the window
          match self.scale_by_size(
                x as f32,
                y as f32,
                drawing_width as u32,
                drawing_height as u32
          ) {

            Some(pixmap) => pixmap,

            None => {
              println!("Cannot scale pixmap");
              Pixmap::new(1, 1).unwrap()
            },
          }
        } else {
          self.copy_image_section(x, y, drawing_width, drawing_height)
        }
      },

      DisplayType::ScaleAlways => {

        // Create a scaled version of the image that fills the window
        match self.scale_by_size(
              x as f32,
              y as f32,
              drawing_width as u32,
              drawing_height as u32
        ) {

          Some(pixmap) => pixmap,

          None => {
            println!("Cannot scale pixmap");
            Pixmap::new(1, 1).unwrap()
          },
        }
      },
    };

    // Copy the visible portion of the image onto the pixmap
    let sub_pixmap_width = sub_pixmap.width();
    let sub_pixmap_height = sub_pixmap.height();
    WindowUtils::copy_pixmap(
          sub_pixmap,
          sub_pixmap_width as u32,
          sub_pixmap_height as u32,
          &mut pixmap,
          0,
          0
    );

    // Save the modified pixmap
    self.window_base.set_pixmap(pixmap);
  }

  /// Gets the path of the image being displayed
  pub fn get_image_path(&self) -> Option<String> {
    self.image_path.clone()
  }

  /// Gets the size of the image
  pub fn get_image_size(&self) -> (u32, u32) {
    (self.full_pixmap.width(), self.full_pixmap.height())
  }

  /// Load an image from a file
  pub fn load(&mut self, filename: &String) {

    self.image = match WindowUtils::load_image(filename) {
      Ok(img) => {
        self.image_path = Some(filename.clone());
        img
      },
      Err(err) => {
        println!("Could not load image from file {filename}: {err}");
        return;
      }
    };

    // Save the image's size
    self.image_width = self.image.width() as f64;
    self.image_height = self.image.height() as f64;

    // Create the Pixmap that contains the full image
    self.full_pixmap = WindowUtils::create_image_pixmap(&self.image);

    // Update the Pixmap
    self.draw();

    // If this image is being display in its actual size, set the scroll ranges
    match self.display_type {

      DisplayType::Actual => {

        // Set the scroll ranges
        let (win_width, win_height) = self.window_base.get_drawing_size();
        self.set_x_scroll_min(0.0);
        if self.image_width > win_width {
          self.set_x_scroll_max(self.image_width - win_width);
        } else {
          self.set_x_scroll_max(0.0);
        }
        self.set_x_scroll(0.0);
        self.set_y_scroll_min(0.0);
        if self.image_height > win_height {
          self.set_y_scroll_max(self.image_height - win_height);
        } else {
          self.set_y_scroll_max(0.0);
        }
        self.set_y_scroll(0.0);

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
      },

      _ => {},
    }

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

  /// Rotates the image clockwise 90 degrees
  pub fn rotate_clockwise(&mut self) {

    // Get the current pixmap's data
    let old_width = self.image_width as u32;
    let old_height = self.image_height as u32;

    // Create a pixmap for the rotated data
    let mut rotated_pixmap = match Pixmap::new(old_height, old_width) {
      Some(pixmap) => pixmap,
      None => {
        println!("Cannot create a rotated pixmap of size {old_height} x {old_width}");
        return;
      },
    };

    // Get the rotated pixmap's pixels
    let rotated_pixels = rotated_pixmap.pixels_mut();

    // Copy the pixel data into the new pixmap, rotating it to it's new position
    for col in 0..old_width {
      for row in 0..old_height {

        let pixel = match self.full_pixmap.pixel(col, row) {
          Some(pixel) => pixel,
          None => {
            println!("Cannot retrieve the pixel at ({col}, {row})");
            return;
          },
        };

        // Place the pixel at the correct location in the rotated pixmap
        let new_col = old_height - row - 1;
        let new_row = col;
        let pixel_index = (new_row * old_height) + new_col;

        // Copy this pixel's data into its new location in the rotated pixmap
        rotated_pixels[pixel_index as usize] = pixel;
      }
    }

    // Replace the full image with the rotated one
    self.image_width = rotated_pixmap.width() as f64;
    self.image_height = rotated_pixmap.height() as f64;
    self.full_pixmap = rotated_pixmap;
    self.window_base.set_x_scroll(0.0);
    self.window_base.set_y_scroll(0.0);

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

  /// Rotates the image counter clockwise 90 degrees
  pub fn rotate_counter_clockwise(&mut self) {

    // Get the current pixmap's size
    let old_width = self.image_width as u32;
    let old_height = self.image_height as u32;

    // Create a pixmap for the rotated data
    let mut rotated_pixmap = match Pixmap::new(old_height, old_width) {
      Some(pixmap) => pixmap,
      None => {
        println!("Cannot create a rotated pixmap of size {old_height} x {old_width}");
        return;
      },
    };

    // Get the rotated pixmap's pixels
    let rotated_pixels = rotated_pixmap.pixels_mut();

    // Copy the pixel data into the new pixmap, rotating it to it's new position
    for col in 0..old_width {
      for row in 0..old_height {

        let pixel = match self.full_pixmap.pixel(col, row) {
          Some(pixel) => pixel,
          None => {
            println!("Cannot retrieve the pixel at ({col}, {row})");
            return;
          },
        };

        // Place the pixel at the correct location in the rotated pixmap
        let new_col = row;
        let new_row = old_width - col - 1;
        let pixel_index = (new_row * old_height) + new_col;

        // Copy this pixel's data into its new location in the rotated pixmap
        rotated_pixels[pixel_index as usize] = pixel;
      }
    }

    // Replace the full image with the rotated one
    self.image_width = rotated_pixmap.width() as f64;
    self.image_height = rotated_pixmap.height() as f64;
    self.full_pixmap = rotated_pixmap;
    self.window_base.set_x_scroll(0.0);
    self.window_base.set_y_scroll(0.0);

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

  fn scale(
        &self,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
        scaling_factor_width: f32,
        scaling_factor_height: f32
  ) -> Option<Pixmap> {

    // Create the scaled pixmap
    let scaled_pixmap = match Pixmap::new(width, height) {

      Some(mut scaled_pixmap) => {

        // Create the drawing canvas
        let mut canvas = scaled_pixmap.as_mut();

        // Get the pixel data for the original pixmap
        let pixel_data = WindowUtils::get_pixel_data(
              self.full_pixmap.clone(),
              x as i32,
              y as i32,
              self.image_width as u32,
              self.image_height as u32
        );

        // Create a PixmapRef from the data
        let pixmap_ref = match PixmapRef::from_bytes(
              &pixel_data,
              self.image_width as u32,
              self.image_height as u32
        ) {
          Some(pixmap_ref) => pixmap_ref,
          None => {
            println!("In TextFont::draw_image(), cannot create a PixmapRef from the source pixmap");
            return None;
          },
        };

        // Scale the pixmap and draw it onto the canvas
        canvas.draw_pixmap(
            0,
            0,
            pixmap_ref,
            &PixmapPaint::default(),
            Transform::from_scale(scaling_factor_width, scaling_factor_height),
            None,
        );

        scaled_pixmap
      },

      None => {
        println!("Failed to create a scaled pixmap");
        Pixmap::new(1, 1).unwrap()
      },
    };

    Some(scaled_pixmap)
  }

  // Returns a scaled version of the image
  fn scale_by_size(
        &self,
        x: f32,
        y: f32,
        width: u32,
        height: u32
  ) -> Option<Pixmap> {

    // Calculate the scaling factors
    let scaling_factor_width = width as f64/ self.image_width;
    let scaling_factor_height = height as f64 / self.image_height;

    // Use the smaller scaling factor
    let scaling_factor;
    if scaling_factor_width < scaling_factor_height {
      scaling_factor = scaling_factor_width;
    } else {
      scaling_factor = scaling_factor_height;
    }

    self.scale(x, y, width, height, scaling_factor as f32, scaling_factor as f32)
  }
}

impl Debug for ImageView {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    let image_path = match &self.image_path {
      Some(path) => path,
      None => &"".to_string(),
    };

    write!(fmt, "ImageView; UUID: {}, display_type: {:?}, image: {}",
          self.get_uuid(), self.display_type, image_path)
   }
}

impl ChildWindow for ImageView {

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

  fn get_max_horizontal_visible_items(&self) -> f64 {

    let (width, _height) = self.window_base.get_drawing_size();

    // Return the number of pixels tha can be visible
    if width > self.window_base.get_width() {
      self.window_base.get_width()
    } else {
      width
    }
  }
  fn get_max_vertical_visible_items(&self) -> f64 {

    let (_width, height) = self.window_base.get_drawing_size();

    // Return the number of pixels tha can be visible
    if height > self.window_base.get_height() {
      self.window_base.get_height()
    } else {
      height
    }
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
  fn set_x_scroll(&mut self, amount: f64) {

    if amount < self.image_width {

      // Save the new scroll amount
      self.window_base.set_x_scroll(amount);

      // Update the Pixmap
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
  fn set_y_scroll(&mut self, amount: f64) {

    if amount < self.image_height {

      // Save the new scroll amount
      self.window_base.set_y_scroll(amount);

      // Update the Pixmap
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

    // If the user clicked the left button, call the callback. If the user clicked
    // the right button, display the tooltip, if there is one.
    match button {

      MouseButton::Left => {
      },

      MouseButton::Right => {
        self.window_base.handle_mouse_pressed(button, mouse_x, mouse_y);
      },

      _ => {},
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

  fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta, _phase: TouchPhase) {

    // Scroll the image in whichever direction has the most unseen pixels
    let (width, height) = self.window_base.get_size();
    let horz_diff = self.image_width - width;
    let vert_diff = self.image_height - height;
    if horz_diff < 0.0 && vert_diff < 0.0 {   // Entire image is already visible
      return;
    }
    if horz_diff > vert_diff {
      match delta {
        LineDelta(_horz_amount, vert_amount) => {

          let current_value = self.window_base.get_x_scroll();

          // Reverse the sign on the amount, so scrolling up is negative.
          let scroll_amount = vert_amount * -1.0;

          // If the user is scrolling up and the image is already scrolled to the top, do nothing.
          if current_value != 0.0 || scroll_amount > 0.0 {

            // Scroll 10 pixels for each wheel position
            let mut new_value = current_value + (scroll_amount * 10.0) as f64;
            if new_value < 0.0 {
              new_value = 0.0;
            }
            self.set_x_scroll(new_value);
          }
        },
        PixelDelta(_logical_position) => {},
      }
    } else {
      match delta {
        LineDelta(_horz_amount, vert_amount) => {

          let current_value = self.window_base.get_y_scroll();

          // Reverse the sign on the amount, so scrolling up is negative.
          let scroll_amount = vert_amount * -1.0;

          // If the user is scrolling up and the image is already scrolled to the top, do nothing.
          if current_value != 0.0 || scroll_amount > 0.0 {

            // Scroll 10 pixels for each wheel position
            let mut new_value = current_value + (scroll_amount * 10.0) as f64;
            if new_value < 0.0 {
              new_value = 0.0;
            }
            self.set_y_scroll(new_value);
          }
        },
        PixelDelta(_logical_position) => {},
      }
    }
  }

  fn populate_context_menu(&self, context_menu_rc: Rc<RefCell<ContextMenu>>) {
    self.window_base.populate_context_menu(context_menu_rc);
  }

  fn redraw(&mut self, x: f64, y: f64, width: f64, height: f64, force: bool) -> Pixmap {

    // Save the location
    self.window_base.set_location(x, y);

    // Has the size changed?
    if force || width != self.window_base.get_width() || height != self.window_base.get_height() {

      // Save the new size
      self.window_base.set_size(width, height);

      // Update the pixmap
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
