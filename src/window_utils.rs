use winit::event_loop::EventLoopProxy;

use tiny_skia::{
  IntRect,
  IntSize,
  Paint,
  PathBuilder,
  Pixmap,
  PixmapPaint,
  PixmapRef,
  Rect,
  Stroke,
  Transform,
};

use image::{
  DynamicImage,
  GenericImageView,
  ImageReader,
};

use chrono::Local;

use uuid::Uuid;

use crate::UserEvent;

use std::{
  io::Cursor,
  rc::Rc,
};

static mut IGNORE_EVENTS: bool = true;

/// Utility functions used by child windows
pub struct WindowUtils {
}

impl WindowUtils {

  /// Copies the contents of one pixmap onto another
  pub fn copy_pixmap(
        src_pixmap: Pixmap,
        width: u32,
        height: u32,
        dest_pixmap: &mut Pixmap,
        dest_x: i32,
        dest_y: i32
  ) {

    // Get the source pixmap's data
    let mut buffer = Vec::new();
    for pixel in src_pixmap.pixels() {
      buffer.push(pixel.red());
      buffer.push(pixel.green());
      buffer.push(pixel.blue());
      buffer.push(pixel.alpha());
    }
//    src_pixmap.read_pixels(&mut buffer).unwrap();
    let pixmap_ref = match PixmapRef::from_bytes(&buffer, width, height) {
      Some(pixmap_ref) => pixmap_ref,
      None => {
        println!("In TextFont::draw_image(), cannot create a PixmapRef from the source pixmap");
        return;
      },
    };

    // Draw the character's pixmap data onto the pixmap
    dest_pixmap.draw_pixmap(
      dest_x,
      dest_y,
      pixmap_ref,
      &PixmapPaint::default(),
      Transform::identity(),
      None
    );
  }

  /// Draws an image into a Pixmap
  pub fn create_image_pixmap(image: &DynamicImage) -> Pixmap {

    // Loads the image's pixels into a buffer
    let mut pixels: Vec<u8> = Vec::new();
    for (_x, _y, pixel) in image.pixels() {
      pixels.push(pixel.0[0]);    // red color component
      pixels.push(pixel.0[1]);    // green
      pixels.push(pixel.0[2]);    // blue
      pixels.push(pixel.0[3]);    // alpha
    }

    // Create a new Pixmap into which the image will be copied
    let size = IntSize::from_wh(image.width(), image.height()).unwrap();
    let pixmap = match Pixmap::from_vec(pixels, size) {
      Some(pixmap) => pixmap,
      None => {
        println!("In WindowUtils::create_image_pixmap(), cannot create a Pixmap for the image");
        Pixmap::new(image.width(), image.height()).unwrap()   // Create an empty Pixmap
      },
    };

    pixmap

  }
  
  /// Gets the current date and time as a string
  pub fn date_str() -> String {
    let date = Local::now();
    format!("{}", date.format("%Y-%m-%d %H:%M:%S%.3f"))
  }

  /// Draws a border around the inner perimeter of a window
  pub fn draw_border(pixmap: &mut Pixmap, width: f64, height: f64, paint: &Paint) {

    // Draw the border
    let stroke = Stroke::default();   // One pixel wide
    let path = PathBuilder::from_rect(Rect::from_ltrb(0.0, 0.0, width as f32, height as f32).unwrap());
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
/*
    let rect = SkRRect::from_rect_and_corners(
      SkRect::new(0.0, 0.0, self.width, self.height), // bounds
      SkFloat::new(5.0), // radius for all corners
    );
    let path = PathBuilder::from_rect(rect);
*/
  }

  // Sends a UserEvent
  pub fn fire_user_event(
        event_loop: Rc<EventLoopProxy<UserEvent>>,
        event: UserEvent
  ) {

    unsafe {
      if IGNORE_EVENTS {
        return;
      }
    }

    match event_loop.send_event(event) {
      Ok(_) => {},
      Err(err) => println!("Cannot send UserEvent event: {err}"),
    }
  }

  /// Gets the pixel data of a portion of a Pixmap
  pub fn get_pixel_data(
        src_pixmap: Pixmap,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
  ) -> Vec<u8> {

    // Define a Rect for the desired portion of the Pixmap
    let rect = match IntRect::from_xywh(
            x,
            y,
            width,
            height
    ) {
      Some(rect) => rect,
      None => {
        println!("In WindowUtils::get_pixel_data(), cannot create an IntRect from ({},{}) with size {} x {}",
              x, y, width, height);
        return Vec::new();
      },
    };

    // Create a Pixmap from the desired portion of the source pixmap
    let portion_pixmap = match src_pixmap.clone_rect(rect) {
      Some(pixmap) => pixmap,
      None => {
        println!("Cannot create sub-image pixmap");
        Pixmap::new(1, 1).unwrap()
      },
    };

    // Get the portion pixmap's data
    let mut buffer = Vec::new();
    for pixel in portion_pixmap.pixels() {
      buffer.push(pixel.red());
      buffer.push(pixel.green());
      buffer.push(pixel.blue());
      buffer.push(pixel.alpha());
    }

    buffer
  }

  /// Loads an image from a file into memory.
  ///
  /// Returns the image
  pub fn load_image(filename: &String) -> Result<DynamicImage, String> {

    // Open the image file
    let reader = match ImageReader::open(filename) {
      Ok(reader) => reader,
      Err(err) => return Err(format!("Could not open image: {err}")),
    };

    // Decode the image
    match reader.decode() {
      Ok(img) => Ok(img),
      Err(err) => Err(format!("Could not open image: {err}")),
    }
  }

  /// Loads an image from a resource file into memory.
  ///
  /// Returns the image
  pub fn load_image_from_resource(image_data: &[u8]) -> Result<DynamicImage, String> {

    // Create the image reader from the data
    let mut reader = ImageReader::new(Cursor::new(image_data));
    reader.set_format(image::ImageFormat::Png);

    // Decode the image
    match reader.decode() {
      Ok(img) => Ok(img),
      Err(err) => Err(format!("Could not decode image: {err}")),
    }
  }

  // Sends a redraw request event
  pub fn request_full_redraw(
        event_loop: Rc<EventLoopProxy<UserEvent>>,
        main_win_uuid: Uuid
  ) {
    WindowUtils::fire_user_event(event_loop, UserEvent::RedrawAll(main_win_uuid));
  }

  // Sends a redraw request event
  pub fn request_redraw(
        event_loop: Rc<EventLoopProxy<UserEvent>>,
        main_win_uuid: Uuid,
        x: f64,
        y: f64,
        pixmap: Pixmap
  ) {
    WindowUtils::fire_user_event(event_loop, UserEvent::Redraw(main_win_uuid, x, y, pixmap));
  }

  pub fn set_ignore_events(flag: bool) {
    unsafe {
      IGNORE_EVENTS = flag;
    }
  }
}
