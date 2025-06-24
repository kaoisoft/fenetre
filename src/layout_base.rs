use crate::UserEvent;
use crate::WindowUtils;

use winit::event_loop::EventLoopProxy;

use tiny_skia::Pixmap;

use uuid::Uuid;

use std:: {
  rc::Rc,
};

/// Base child window object that is contained within each concrete window
#[derive(Clone, Debug)]
pub struct LayoutBase {
    uuid: Uuid,                                 // layout's unique ID
    event_loop: Rc<EventLoopProxy<UserEvent>>,  // main window's event loop
    main_win_uuid: Uuid,                        // ID of the outermost parent window
    pixmap: Pixmap,                             // layout's contents
    main_win_x: f64,                            // layout's location within the main window
    main_win_y: f64,
    width: f64,                                 // size of the layout
    height: f64,
    name: String,                               // only used in Debug
}

impl LayoutBase {

  pub fn new(event_loop: Rc<EventLoopProxy<UserEvent>>, main_win_uuid: Uuid) -> Self {

    // Set all of the default values
    Self {
      uuid: Uuid::new_v4(),
      event_loop: event_loop,
      main_win_uuid: main_win_uuid,
      pixmap: Pixmap::new(1, 1).unwrap(),
      main_win_x: 0.0,
      main_win_y: 0.0,
      width: 1.0,
      height: 1.0,
      name: "<unspecified>".to_string(),
    }
  }

  pub fn get_uuid(&self) -> Uuid {
    self.uuid
  }
  pub fn set_uuid(&mut self, uuid: Uuid) {
    self.uuid = uuid;
  }

  pub fn get_event_loop(&self) -> Rc<EventLoopProxy<UserEvent>> {
    self.event_loop.clone()
  }
  pub fn set_event_loop(&mut self, event_loop: Rc<EventLoopProxy<UserEvent>>) {
    self.event_loop = event_loop;
  }

  pub fn get_main_win_uuid(&self) -> Uuid {
    self.main_win_uuid
  }
  pub fn set_main_win_uuid(&mut self, main_win_uuid: Uuid) {
    self.main_win_uuid = main_win_uuid;
  }

  pub fn get_pixmap(&self) -> Pixmap {
    self.pixmap.clone()
  }
  pub fn set_pixmap(&mut self, pixmap: Pixmap) {
    self.pixmap = pixmap;
  }

  pub fn get_main_win_x(&self) -> f64 {
    self.main_win_x
  }
  pub fn set_main_win_x(&mut self, main_win_x: f64) {
    self.main_win_x = main_win_x;
  }

  pub fn get_main_win_y(&self) -> f64 {
    self.main_win_y
  }
  pub fn set_main_win_y(&mut self, main_win_y: f64) {
    self.main_win_y = main_win_y;
  }

  pub fn get_location(&self) -> (f64, f64) {
    (self.main_win_x, self.main_win_y)
  }

  pub fn get_width(&self) -> f64 {
    self.width
  }
  pub fn set_width(&mut self, width: f64) {
    self.width = width;
  }

  pub fn get_height(&self) -> f64 {
    self.height
  }
  pub fn set_height(&mut self, height: f64) {
    self.height = height;
  }

  pub fn get_size(&self) -> (f64, f64) {
    (self.width, self.height)
  }

  pub fn get_name(&self) -> String {
    self.name.clone()
  }
  pub fn set_name(&mut self, name: String) {
    self.name = name;
  }

  pub fn layout(&mut self, _main_win_x: f64, _main_win_y: f64, width: f64, height: f64) -> Pixmap {

    let pixmap = match Pixmap::new(width as u32, height as u32) {

      Some(pixmap) => pixmap,

      None => Pixmap::new(1, 1).unwrap(),
    };

    self.pixmap = pixmap.clone();

    pixmap
  }

  pub fn update(&mut self) {

    // Redraw the pixmap
    let pixmap = self.layout(self.main_win_x, self.main_win_y, self.width, self.height);

    // Tell the main window to redraw the pixmap
    WindowUtils::request_redraw(
          self.event_loop.clone(),
          self.main_win_uuid,
          self.main_win_x,
          self.main_win_y,
          pixmap
    );
  }
}
