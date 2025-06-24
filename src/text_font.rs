use fontdue::{Font};

use tiny_skia::{
  Color,
  Paint,
  PathBuilder,
  Pixmap,
  PixmapPaint,
  Stroke,
  Transform,
};

pub struct TextFont {
  font: Font,
  font_path: String,
  font_size: f32,
  max_char_height: u32,
  max_char_width: u32,
}

use std::{
  fmt::Debug,
};

impl TextFont {

  pub fn new(font_path: &str, font_size: f32) -> Result<TextFont, String> {

    // Load the font
    let font = include_bytes!("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf") as &[u8];

    // Parse it into the font type.
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

    Ok(Self {
          font: font,
          font_path: font_path.to_string(),
          font_size: font_size,
          max_char_width: 0,
          max_char_height: 0,
    })
  }

  // Returns a Pixmap the contains the specified character as an image.
  //
  // returns:
  //  Pixmap - the character's Pixmap
  //  u32 - number of pixels above the baseline
  //  u32 - number of pixels below the baseline
  fn draw_character(
    &self,
    ch: char,
    red: u8,
    green: u8,
    blue: u8,
    bg_color: Color,
    fixed_size: Option<(u32, u32)>,
  ) -> (Pixmap, u32, u32) {

    // Rasterize and get the layout metrics for the letter at the current
    // font size.
    //
    // metrics is a Metrics and contains the sizing and positioning metadata
    // for the rasterized glyph.
    //
    // coverage_data is Vec<u8> and contains the coverage vector for the glyph.
    // Coverage is a linear scale where 0 represents 0% coverage of that pixel
    // by the glyph and 255 represents 100% coverage. The vec starts at
    // the top left corner of the glyph.
    let (metrics, coverage_data) = self.font.rasterize(ch, self.font_size);

    // Determine how many pixels are below the baseline. Some characters, like
    // '~' and '-', have a positive value, which indicates how far down the
    // character's pixmap needs to be shifted. Otherwise, the value is the
    // depth below the baseline the pixmap extends.
    let mut shift_down = 0;
    let mut pixels_below = 0;
    let mut pixels_above = metrics.height as u32;
    if metrics.ymin < 0 {
      pixels_below = -metrics.ymin;
      // If any part of the character is below the baseline, shift the character
      // down by filling in the top of the Pixmap.
      shift_down = pixels_below;
    } else if 0 < metrics.ymin {
      pixels_above = (metrics.height as i32 + metrics.ymin) as u32;
    }
    let mut full_height = metrics.height as i32 + pixels_below;
    if 0 == full_height {   // Spaces have a height of 0
      full_height = 1;
    }

    // Create a Pixmap for the image
    let mut pixmap;
    match fixed_size {

      Some((width, height)) => {
        if width < metrics.width as u32 {
          pixmap = Pixmap::new(metrics.width as u32, height as u32).unwrap();
        } else {
          pixmap = match Pixmap::new(width, height as u32) {

            Some(pixmap) => pixmap,

            None => {
              println!("Cannot create a Pixmap of size {width} x {height}");
              Pixmap::new(metrics.width as u32, height as u32).unwrap()
            },
          }
        }
      },

      None => {

        pixmap = match Pixmap::new(
              metrics.width as u32,
              full_height as u32
        ) {
          Some(pixmap) => pixmap,

          None => {
            if ' ' == ch {
              return (Pixmap::new(8, 16).unwrap(), 16, 0);
            } else {
              return (Pixmap::new(1, 1).unwrap(), 1, 0);
            }
          },
        }
      },
    };
    let pixmap_width = pixmap.width();

    // Fill in the background color
    pixmap.fill(bg_color);

    let pixels = pixmap.data_mut();

    // If any part of the character is below the baseline, shift the character
    // down by filling in the top of the Pixmap.
    let mut pixel_index = 0;
    let mut i = 0;
    while i < shift_down {

      let mut col = 0;
      while col < pixmap_width && pixel_index < pixels.len() - 4 {

        pixels[pixel_index] = 0;
        pixels[pixel_index + 1] = 0;
        pixels[pixel_index + 2] = 0;
        pixels[pixel_index + 3] = 0;

        pixel_index += 4;
        col += 1;
      }

      i += 1;
    }

    // Fill in the pixels according to the coverage data
    let mut index = 0;
    let mut row_index = 0;
    while index < coverage_data.len() && pixel_index < pixels.len() - 4 {

      pixels[pixel_index] = red;
      pixels[pixel_index + 1] = green;
      pixels[pixel_index + 2] = blue;
      pixels[pixel_index + 3] = coverage_data[index];

      index += 1;
      pixel_index += 4;

      // row_index is used to determine when padding is need on the right of a scanline
      row_index += 1;
      if row_index == metrics.width {
        if pixmap_width > metrics.width as u32 {

          // Skip to the beginning of the next scanline
          pixel_index += ((pixmap_width - metrics.width as u32) * 4) as usize;
        }
        row_index = 0;
      }
    }

    (pixmap, pixels_above, pixels_below as u32)
  }

  // Returns a Pixmap that contains the image of a single line of text.
  fn draw_image(
        &self,
        text: &str,
        color: Color,
        bg_color: Color,
        draw_caret: bool,
        caret_index: i64,
        caret_color: Color,
        fixed_width: Option<u32>
  ) -> Pixmap{

    // Set the fixed height if a fixed width was specified
    let fixed_size: Option<(u32, u32)> = match fixed_width {

      Some(width) => Some((width, 13)),

      None => None,
    };

    // Get the text color's components
    let color_u8 = color.to_color_u8();
    let red = color_u8.red();
    let green = color_u8.green();
    let blue = color_u8.blue();

    // Get the Pixmap for each character and use it to calculate
    // the size of the full Pixmap
    let mut pixmap_width = 0;
    let mut max_height = 0;
    let mut max_below = 0;
    for ch in text.chars() {

      // Get this character's Pixmap
      let (ch_pixmap, _pixels_above, pixels_below) =
          self.draw_character(ch, red, green, blue, bg_color, fixed_size);
      if pixels_below > max_below {
        max_below = pixels_below;
      }

      // Adjust the size of the full Pixmap
      pixmap_width += ch_pixmap.width();
      if ch_pixmap.height() > max_height {
        max_height = ch_pixmap.height();
      }
    }
    let baseline = max_height;

    // Create the full Pixmap
    let pixmap_height = baseline + max_below;
    let mut full_pixmap = match Pixmap::new(pixmap_width, pixmap_height) {
      Some(pixmap) => pixmap,

      None => {
        return Pixmap::new(1, 1).unwrap();
      },
    };

    // Draw each character Pixmap into the full Pixmap
    let mut x = 0;
    let mut caret_x = 0.0;
    let mut char_index = 0;
    for ch in text.chars() {

      // If this character is where the caret is location, save its
      // X coordinate of its left edge within the full pixmap so that
      // we can later draw the caret at this location.
      if draw_caret && char_index == caret_index {
        caret_x = x as f32;
      }

      // Get this character's Pixmap
      let (ch_pixmap, pixels_above, _pixels_below) =
            self.draw_character(ch, red, green, blue, bg_color, fixed_size);

      // Determine where the top of this character's Pixmap goes. Some
      // characters, lower case letters for example, have shorter Pixmaps
      // than other chracters, so they need to be placed appropriately
      // within the larger Pixmap.
      let top;
      if baseline > pixels_above {
        top = baseline - pixels_above;
      } else {
        top = 0;
      }

      full_pixmap.draw_pixmap(
        x,
        top as i32,
        ch_pixmap.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None
      );

      x += ch_pixmap.width() as i32;
      char_index += 1;
    }

    // Draw the caret
    if draw_caret {

      // Is the caret at the end of the text?
      if caret_index == text.len() as i64 {
        caret_x = x as f32;
      }

      let mut pb = PathBuilder::new();
      pb.move_to(caret_x, 0.0);
      pb.line_to(caret_x, pixmap_height as f32);
      let path = pb.finish().unwrap();

      let mut paint = Paint::default();
      paint.set_color(caret_color);

      let stroke = Stroke::default();
      full_pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    // Return the pixmap
    full_pixmap
  }

  /// Draws the text into the specified pixmap at the given location.
  ///
  /// If fixed_width has a value, that value will be the width of each character.
  ///
  /// Returns the size of the drawn text
  pub fn draw_text(
    &self,
    text: &str,
    pixmap: &mut Pixmap,
    x: i32,
    y: i32,
    color: Color,
    bg_color: Color,
    caret_index: i64,
    caret_color: Color,
    fixed_width: Option<u32>,
  ) -> (u32, u32) {

    // Draw the text into a pixmap
    let draw_caret: bool;
    if caret_index >= 0 {
      draw_caret = true;
    } else {
      draw_caret = false;
    }
    let text_pixmap = self.draw_image(
          text,
          color,
          bg_color,
          draw_caret,
          caret_index,
          caret_color,
          fixed_width,
    );

    // Draw the text's pixmap onto the big pixmap
    pixmap.draw_pixmap(
      x,
      y,
      text_pixmap.as_ref(),
      &PixmapPaint::default(),
      Transform::identity(),
      None
    );

    (text_pixmap.width(), text_pixmap.height())
  }

  /// Gets the size of a string, in pixels, when it is rendered using the font.
  ///
  /// If fixed_width has a value, that value will be the width of each character.
  pub fn get_bounds(&self, text: &str, fixed_width: Option<u32>) -> (u32, u32) {

    let pixmap = self.draw_image(
          text,
          Color::BLACK,
          Color::WHITE,
          false,
          0,
          Color::BLACK,
          fixed_width,
    );

    (pixmap.width(), pixmap.height())
  }

  /// Gets the maximum height of a character in the font.
  pub fn get_max_char_height(&mut self) -> u32 {

    if self.max_char_height == 0 {

      // Get the width of the widest letter
      let (width, _height) = self.get_bounds(&"W".to_string(), None);
      self.max_char_width = width;

      // Get the height of a captial letter and one that is partially below the
      // baseline.
      let (_width, height) = self.get_bounds(&"Wy".to_string(), None);
      self.max_char_height = height;
    }

    self.max_char_height
  }

  /// Gets the maximum width of a character in the font.
  pub fn get_max_char_width(&mut self) -> u32 {

    if self.max_char_width == 0 {

      let (width, height) = self.get_bounds(&"W".to_string(), None);
      self.max_char_width = width;
      self.max_char_height = height;
    }

    self.max_char_width
  }
}

impl Debug for TextFont {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "TextFont; font path: {}, font size: {}", self.font_path, self.font_size)
   }
}
