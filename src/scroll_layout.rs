use winit::{
  event_loop::EventLoopProxy,
};

use tiny_skia::{Pixmap, PixmapPaint, Transform};

use crate::ChildWindow;
use crate::child_window::{
  Layout,
  LayoutArgs,
  LayoutFill,
  LayoutType,
  Orientation,
};
use crate::layout_base::LayoutBase;
use crate::scroll_bar::{
  BAR_SIZE,
  ScrollBar,
};
use crate::UserEvent;

use uuid::Uuid;

use std::{
  cell::RefCell,
  fmt::Debug,
  rc::Rc,
};

pub struct ScrollLayout {
  layout_base: LayoutBase,
  max_width: Option<f64>,
  max_height: Option<f64>,
  child: Option<Rc<RefCell<dyn ChildWindow>>>,
  h_scroll: Rc<RefCell<ScrollBar>>,
  v_scroll: Rc<RefCell<ScrollBar>>,
}

/// Layout with a single child that can be scrolled
impl ScrollLayout {

  pub fn new(
        event_loop: Rc<EventLoopProxy<UserEvent>>,
        main_win_uuid: Uuid,
  ) -> ScrollLayout {

    let h_scroll = ScrollBar::new(
          event_loop.clone(),
          main_win_uuid,
          Orientation::Horizontal
    );

    let v_scroll = ScrollBar::new(
          event_loop.clone(),
          main_win_uuid,
          Orientation::Vertical
    );

    let layout_base = LayoutBase::new(event_loop, main_win_uuid);

    Self {
      layout_base: layout_base,
      max_width: None,
      max_height: None,
      child: None,
      h_scroll: Rc::new(RefCell::new(h_scroll)),
      v_scroll: Rc::new(RefCell::new(v_scroll)),
    }
  }

  /// Returns the Uuids of the ScrollBars
  ///
  /// The first Uuid is the horizontal ScrollBar
  /// The second Uuid is the horizontal ScrollBar
  pub fn get_scroll_bar_uuids(&self) -> (Uuid, Uuid) {

    let h_scroll_ref = self.h_scroll.borrow();
    let v_scroll_ref = self.v_scroll.borrow();

    (h_scroll_ref.get_uuid(), v_scroll_ref.get_uuid())
  }
}

impl Debug for ScrollLayout {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "ScrollLayout; UUID: {}, name: {}",
          self.layout_base.get_uuid(), self.layout_base.get_name())
   }
}

impl Layout for ScrollLayout {

  /// Adds a child window
  ///
  /// If an error occurred, the returned Result contains a string explaining why it failed.
  fn add_child(&mut self, child: Rc<RefCell<dyn ChildWindow>>,
      _args: LayoutArgs) -> Result<(), String> {

    // If this ScrollLayout already has a child, remove it as that child's parent.
    match &self.child {

      Some(child) => {
        let mut child_ref = child.borrow_mut();
        child_ref.set_parent(None);
      },

      None => {},
    }

    self.child = Some(child.clone());

    // Set the scroll handler for the horizontal scroll bar
    let h_scroll_rc = self.h_scroll.clone();
    let mut h_scroll_ref = h_scroll_rc.borrow_mut();
    let h_child_clone = self.child.clone();
    h_scroll_ref.set_scrolling_callback(Box::new(move |_orientation, value| {
      match &h_child_clone {

        Some(child_rc) => {
          let mut child_ref = child_rc.borrow_mut();
          child_ref.set_x_scroll(value);
        },

        None => {},
      }
    }));

    // Set the scroll handler for the vertical scroll bar
    let v_scroll_rc = self.v_scroll.clone();
    let mut v_scroll_ref = v_scroll_rc.borrow_mut();
    let v_child_clone = self.child.clone();
    v_scroll_ref.set_scrolling_callback(Box::new(move |_orientation, value| {
      match &v_child_clone {

        Some(child_rc) => {
          let mut child_ref = child_rc.borrow_mut();
          child_ref.set_y_scroll(value);
        },

        None => {},
      }
    }));

    Ok(())
  }

  /// Adds an inner layout
  ///
  /// If an error occurred, the returned Result contains a string explaining why it failed.
  fn add_layout(&mut self, _layout: Rc<RefCell<dyn Layout>>,
    _args: LayoutArgs) -> Result<(), String> {

    Err("Nested layout is not supported by ScrollLayout".to_string())
  }

  fn clear(&mut self) {
    self.child = None;
  }

  /// The only layout that performs any actions in this method is the TabLayout
  fn close_tab(&mut self, _tab_uuid: Uuid) {
  }

  /// Gets the child that contains the specified screen pixel location
  fn get_child_at(&self, x: f64, y: f64) -> Option<Rc<RefCell<dyn ChildWindow>>> {

    // If the location is not within this layout, stop checking
    if x < self.layout_base.get_main_win_x() ||
          x > self.layout_base.get_main_win_x() + self.layout_base.get_width() ||
          y < self.layout_base.get_main_win_y() ||
          y > self.layout_base.get_main_win_y() + self.layout_base.get_height()
    {
      return None;
    }

    // Set the size of the scroll bars based on their visibility
    let h_size;
    let scroll_bar = self.h_scroll.borrow();
    if scroll_bar.is_visible() {
      h_size = BAR_SIZE;
    } else {
      h_size = 0.0;
    }
    let v_size;
    let scroll_bar = self.v_scroll.borrow();
    if scroll_bar.is_visible() {
      v_size = BAR_SIZE;
    } else {
      v_size = 0.0;
    }

    if x >= self.layout_base.get_main_win_x() + self.layout_base.get_width() - v_size {
      Some(self.v_scroll.clone())
    } else if y >= self.layout_base.get_main_win_y() + self.layout_base.get_height() - h_size {
      Some(self.h_scroll.clone())
    } else {

      // If the coordinates don't match either scroll bar, then they have to match the child,
      // if there is one.
      match &self.child {
        Some(child) => {
          Some(child.clone())
        },
        None => None,
      }
    }
 }

  /// Gets the child window with the specified ID
  fn get_child_with_id(&mut self, uuid: Uuid) -> Option<Rc<RefCell<dyn ChildWindow>>> {

    // If this ID matches the scroll bar's ID, return the child instead.
    if self.layout_base.get_uuid() == uuid {
      match &self.child {
        Some(child) => {
          return Some(child.clone());
        },
        None => {},
      }
    }

    // If this ID matches either of the scroll bars, return the child window instead.
    let v_scroll_ref = self.v_scroll.borrow();
    let h_scroll_ref = self.h_scroll.borrow();
    if v_scroll_ref.get_uuid() == uuid || h_scroll_ref.get_uuid() == uuid {
      return self.child.clone();
    } else {
      match &self.child {
        Some(child) => {
          let child_ref = child.borrow();
          if child_ref.get_uuid() == uuid {
            return self.child.clone();
          }
        },
        None => {},
      }
    }

    None
  }

  /// Gets the child layout with the specified ID
  fn get_layout_with_id(&self, _uuid: Uuid) -> Option<Rc<RefCell<dyn Layout>>> {
    // ScrollLayout nevers contains a nested layout
    None
  }

  /// Gets the list of all descendent layouts of the specified type
  fn get_layouts_of_type(&self, _layout_type: LayoutType) -> Vec<Rc<RefCell<dyn Layout>>> {
    Vec::new()
  }

  fn get_type(&self) -> LayoutType {
    LayoutType::ScrollLayout
  }

  /// Lays out the child windows within the parent.
  ///
  /// main_win_x and main_win_y are the relative the main window
  /// width and height are the size of the layout's region
  ///
  /// Returns a pixmap onto which the window has been drawn
  fn layout(&mut self, main_win_x: f64, main_win_y: f64, width: f64, height: f64) -> Pixmap {

    // Save the location
    self.layout_base.set_main_win_x(main_win_x);
    self.layout_base.set_main_win_y(main_win_y);

    // Save the size, taking into account the max size
    let new_width = match self.max_width {

      Some(max) => {
        if width > max {
          max
        } else {
          width
        }
      },

      None => width,
    };
    self.layout_base.set_width(new_width);
    let new_height = match self.max_height {

      Some(max) => {
        if height > max {
          max
        } else {
          height
        }
      },

      None => height,
    };
    self.layout_base.set_height(new_height);

    // Create the pixmap
    let mut pixmap = match Pixmap::new(
          self.layout_base.get_width() as u32,
          self.layout_base.get_height() as u32
    ) {

      Some(pixmap) =>  pixmap,

      None => {
        println!("Cannot create a layout pixmap of size {} x {}",
              self.layout_base.get_width(), self.layout_base.get_height());
        Pixmap::new(1, 1).unwrap()
      },
    };

    // If there is a window being scrolled draw the window's contents and,
    // if necessary, the scroll bars.
    match &self.child {

      Some(child) => {

        // Get the scroll ranges and values
        let mut child_ref = child.borrow_mut();
        let h_min = child_ref.get_x_scroll_min();
        let h_max = child_ref.get_x_scroll_max();
        let h_value = child_ref.get_x_scroll();
        let v_min = child_ref.get_y_scroll_min();
        let v_max = child_ref.get_y_scroll_max();
        let v_value = child_ref.get_y_scroll();

        // Set the scroll bars
        let mut h_scroll_ref = self.h_scroll.borrow_mut();
        h_scroll_ref.set_range(h_min, h_max, h_value);
        let h_visible = h_scroll_ref.is_visible();
        let mut v_scroll_ref = self.v_scroll.borrow_mut();
        v_scroll_ref.set_range(v_min, v_max, v_value);
        let v_visible = v_scroll_ref.is_visible();

        let paint = PixmapPaint::default();

        // Draw the vertical scroll bar
        let v_scroll_size;
        if v_visible {

          let v_height;
          if h_visible {
            v_height = height - BAR_SIZE;
          } else {
            v_height = height;
          }

          v_scroll_size = BAR_SIZE;
          let child_pixmap = v_scroll_ref.redraw(
                self.layout_base.get_main_win_x() + self.layout_base.get_width() - BAR_SIZE,
                self.layout_base.get_main_win_y(),
                BAR_SIZE,
                v_height,
                true
          );
          pixmap.draw_pixmap(
              (width - BAR_SIZE) as i32,
              0,
              child_pixmap.as_ref(),
              &paint,
              Transform::identity(),
              None,
          );
        } else {
          v_scroll_size = 0.0;
        }

        // Draw the horizontal scroll bar
        let h_scroll_size;
        if h_scroll_ref.is_visible() {

          let h_width;
          if v_visible {
            h_width = width - BAR_SIZE;
          } else {
            h_width = width;
          }

          h_scroll_size = BAR_SIZE;
          let child_pixmap = h_scroll_ref.redraw(
                self.layout_base.get_main_win_x(),
                self.layout_base.get_main_win_y() + self.layout_base.get_height() - BAR_SIZE,
                h_width,
                BAR_SIZE,
                true
          );
          pixmap.draw_pixmap(
              0,
              (height - BAR_SIZE) as i32,
              child_pixmap.as_ref(),
              &paint,
              Transform::identity(),
              None,
          );
        } else {
          h_scroll_size = 0.0;
        }

        // Set the size of the child so that it fills the layout
        let child_width;
        if v_visible {
          child_width = self.layout_base.get_width() - BAR_SIZE;
        } else {
          child_width = self.layout_base.get_width();
        }
        child_ref.set_width(child_width);
        let child_height;
        if h_visible {
          child_height = self.layout_base.get_height() - BAR_SIZE;
        } else {
          child_height = self.layout_base.get_height();
        }
        child_ref.set_height(child_height);

        // Draw the child
        let child_pixmap = child_ref.redraw(
              self.layout_base.get_main_win_x(),
              self.layout_base.get_main_win_y(),
              width - v_scroll_size,
              height - h_scroll_size,
              true
        );
        pixmap.draw_pixmap(
            0,
            0,
            child_pixmap.as_ref(),
            &paint,
            Transform::identity(),
            None,
        );
      },

      None => {},
    }

    self.layout_base.set_pixmap(pixmap.clone());

    pixmap
  }

  /// The only layout that performs any actions in this method is the TabLayout
  fn set_active_tab_by_uuid(&mut self, _tab_uuid: Uuid) {
  }

  /// Sets the fill algorithm
  fn set_fill(&mut self, _algorithm: Box<LayoutFill>) {
  }

  /// Sets the layout's maximum size
  fn set_max_size(&mut self, width: f64, height: f64) {
    self.max_width = Some(width);
    self.max_height = Some(height);
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
    self.layout_base.get_size()
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
