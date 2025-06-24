use crate::ChildWindow;
use crate::child_window::{
  BorderLocation,
  Layout,
  LayoutArgs,
  LayoutFill,
  LayoutType,
  Orientation,
  UserEvent,
};
use crate::layout_base::LayoutBase;
use crate::row_layout::RowLayout;

use winit::event_loop::EventLoopProxy;

use tiny_skia::{Pixmap};

use uuid::Uuid;

use std::{
  cell::RefCell,
  fmt::Debug,
  rc::Rc,
};

/// A BorderLayout has five areas in which a child can be placed:
///      top, bottom, center, left, and right
pub struct BorderLayout {
  layout_base: LayoutBase,
  rows: Rc<RefCell<RowLayout>>,     // Contains the top, center row, and bottom
  center: Rc<RefCell<RowLayout>>,   // Contains the left, center, and right
}

impl BorderLayout {

  pub fn new(event_loop: Rc<EventLoopProxy<UserEvent>>, main_win_uuid: Uuid) -> BorderLayout {

    // Create the internal layouts
    let mut rows = RowLayout::new(
          event_loop.clone(),
          main_win_uuid.clone(),
          Orientation::Vertical,
          0.0
    );
    rows.set_name("BorderLayout's column".to_string());
    let mut center = RowLayout::new(
          event_loop.clone(),
          main_win_uuid.clone(),
          Orientation::Horizontal,
          0.0
    );
    center.set_name("BorderLayout's center row".to_string());

    // Tell the vertical row to give all extra space to the center row
    rows.set_fill(Box::new(LayoutFill::Single(center.get_uuid())));

    // Add the center layout to the rows
    let center_rc = Rc::new(RefCell::new(center));
    match rows.add_layout(center_rc.clone(), LayoutArgs::None) {
      Ok(_) => {},
      Err(err) => println!("Could not add center row to layout: {err}"),
    }

    let layout_base = LayoutBase::new(event_loop, main_win_uuid);

    Self {
      layout_base: layout_base,
      rows: Rc::new(RefCell::new(rows)),
      center: center_rc.clone(),
    }
  }
}

impl Debug for BorderLayout {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "BorderLayout; UUID: {}, name: {}",
        self.layout_base.get_uuid(), self.layout_base.get_name())
   }
}

impl Layout for BorderLayout {

  /// args must be LayoutArgs::BORDER
  fn add_child(&mut self, child: Rc<RefCell<dyn ChildWindow>>,
      args: LayoutArgs) -> Result<(), String> {

    let location = match args {
      LayoutArgs::BORDER(location) => location,
      _ => {
        return Err("Required LayoutArgs::BORDER argument is missing".to_string());
      }
    };

    let mut rows_ref = self.rows.borrow_mut();
    let mut center_ref = self.center.borrow_mut();

    // Insert this child into the appropriate inner layout
    match location {
      BorderLocation::TOP => rows_ref.insert_child(child, 0),
      BorderLocation::BOTTOM => {
        let count = rows_ref.get_child_count();
        let mut index = 2;
        if index > count {
          index = count;
        }
        rows_ref.insert_child(child, index);
      },
      BorderLocation::LEFT => center_ref.insert_child(child, 0),
      BorderLocation::CENTER => {
        let count = center_ref.get_child_count();
        let mut index = 1;
        if index > count {
          index = count;
        }
        let child_clone = child.clone();
        let child_ref = child_clone.borrow();

        // Tell the center row to give all extra space to this child
        let uuid = child_ref.get_uuid();
        center_ref.set_fill(Box::new(LayoutFill::Single(uuid)));
        center_ref.insert_child(child, index);
      },
      BorderLocation::RIGHT => {
        let count = center_ref.get_child_count();
        let mut index = 1;
        if index > count {
          index = count;
        }
        center_ref.insert_child(child, index);
      },
    }

    Ok(())
  }

  fn add_layout(&mut self, layout: Rc<RefCell<dyn Layout>>,
    args: LayoutArgs) -> Result<(), String> {

    let location = match args {
      LayoutArgs::BORDER(location) => location,
      _ => {
        return Err("Required LayoutArgs::BORDER argument is missing".to_string());
      }
    };

    let mut rows_ref = self.rows.borrow_mut();
    let mut center_ref = self.center.borrow_mut();

    // Insert this child into the appropriate inner layout
    match location {
      BorderLocation::TOP => rows_ref.insert_layout(layout, 0),
      BorderLocation::BOTTOM => {
        let count = rows_ref.get_child_count();
        let mut index = 2;
        if index > count {
          index = count;
        }
        rows_ref.insert_layout(layout, index);
      },
      BorderLocation::LEFT => center_ref.insert_layout(layout, 0),
      BorderLocation::CENTER => {
        let count = center_ref.get_child_count();
        let mut index = 1;
        if index > count {
          index = count;
        }

        // Tell the center row to give all extra space to this child
        let layout_clone = layout.clone();
        let layout_ref = layout_clone.borrow();
        let uuid = layout_ref.get_uuid();
        center_ref.set_fill(Box::new(LayoutFill::Single(uuid)));
        center_ref.insert_layout(layout, index);
      },
      BorderLocation::RIGHT => {
        let count = center_ref.get_child_count();
        let mut index = 1;
        if index > count {
          index = count;
        }
        center_ref.insert_layout(layout, index);
      },
    }

    Ok(())
  }

  fn clear(&mut self) {
    let mut rows_ref = self.rows.borrow_mut();
    rows_ref.clear();
    let mut center_ref = self.center.borrow_mut();
    center_ref.clear();
  }

  /// The only layout that performs any actions in this method is the TabLayout
  fn close_tab(&mut self, _tab_uuid: Uuid) {
  }

  /// Gets the child which contains the pixel at the location specified by x and y
  fn get_child_at(&self, x: f64, y: f64) -> Option<Rc<RefCell<dyn ChildWindow>>> {

    let rows_ref = self.rows.borrow();

    match rows_ref.get_child_at(x, y) {
      Some(child) => Some(child),

      None => None,
    }
  }

  /// Gets the child with the specified ID
  fn get_child_with_id(&mut self, uuid: Uuid) -> Option<Rc<RefCell<dyn ChildWindow>>> {

    let mut rows_ref = self.rows.borrow_mut();

    match rows_ref.get_child_with_id(uuid) {
      Some(child) => Some(child),

      None => None,
    }
  }

  fn get_layout_with_id(&self, uuid: Uuid) -> Option<Rc<RefCell<dyn Layout>>> {

    let rows_ref = self.rows.borrow();
    let center_ref = self.center.borrow();

    match rows_ref.get_layout_with_id(uuid) {
      Some(child) => Some(child),

      None => center_ref.get_layout_with_id(uuid),
    }
  }

  /// Gets the list of all descendent layouts of the specified type
  fn get_layouts_of_type(&self, layout_type: LayoutType) -> Vec<Rc<RefCell<dyn Layout>>> {

    let rows_ref = self.rows.borrow();
    rows_ref.get_layouts_of_type(layout_type)
  }

  fn get_type(&self) -> LayoutType {
    LayoutType::BorderLayout
  }

  /// Draws the children
  fn layout(&mut self, main_win_x: f64, main_win_y: f64, width: f64, height: f64) -> Pixmap {

    // Save the location
    self.layout_base.set_main_win_x(main_win_x);
    self.layout_base.set_main_win_y(main_win_y);

    let mut rows_ref = self.rows.borrow_mut();

    let pixmap = rows_ref.layout(main_win_x, main_win_y, width, height);
    pixmap
  }

  /// The only layout that performs any actions in this method is the TabLayout
  fn set_active_tab_by_uuid(&mut self, _tab_uuid: Uuid) {
  }

  fn set_fill(&mut self, _algorithm: Box<LayoutFill>) {
    // BorderLayout implements its own fill algorithm
  }

  fn set_max_size(&mut self, width: f64, height: f64) {

    let mut rows_ref = self.rows.borrow_mut();

    rows_ref.set_max_size(width, height);
  }

  /// LayoutBase pass-through functions

  fn get_uuid(&self) -> Uuid {
    self.layout_base.get_uuid()
  }
  fn set_uuid(&mut self, uuid: Uuid) {
    self.layout_base.set_uuid(uuid);
  }

  fn get_event_loop(&self) -> Rc<EventLoopProxy<UserEvent>> {
    self.layout_base.get_event_loop()
  }
  fn set_event_loop(&mut self, event_loop: Rc<EventLoopProxy<UserEvent>>) {
    self.layout_base.set_event_loop(event_loop);
  }

  fn get_main_win_uuid(&self) -> Uuid {
    self.layout_base.get_main_win_uuid()
  }
  fn set_main_win_uuid(&mut self, main_win_uuid: Uuid) {
    self.layout_base.set_main_win_uuid(main_win_uuid);
  }

  fn get_pixmap(&self) -> Pixmap {
    self.layout_base.get_pixmap()
  }
  fn set_pixmap(&mut self, pixmap: Pixmap) {
    self.layout_base.set_pixmap(pixmap);
  }

  fn get_main_win_x(&self) -> f64 {
    self.layout_base.get_main_win_x()
  }
  fn set_main_win_x(&mut self, main_win_x: f64) {
    self.layout_base.set_main_win_x(main_win_x);
  }

  fn get_main_win_y(&self) -> f64 {
    self.layout_base.get_main_win_y()
  }
  fn set_main_win_y(&mut self, main_win_y: f64) {
    self.layout_base.set_main_win_y(main_win_y);
  }

  fn get_location(&self) -> (f64, f64) {
    self.layout_base.get_location()
  }

  fn get_width(&self) -> f64 {
    self.layout_base.get_width()
  }
  fn set_width(&mut self, width: f64) {
    self.layout_base.set_width(width);
  }

  fn get_height(&self) -> f64 {
    self.layout_base.get_height()
  }
  fn set_height(&mut self, height: f64) {
    self.layout_base.set_height(height);
  }

  fn get_size(&self) -> (f64, f64) {

    // The size of the layout is the sizze of the inner rows layout
    let rows_ref = self.rows.borrow();

    rows_ref.get_size()
  }

  fn get_name(&self) -> String {
    self.layout_base.get_name()
  }
  fn set_name(&mut self, name: String) {
    self.layout_base.set_name(name);
  }

  fn update(&mut self) {
    self.layout_base.update();
  }
}
