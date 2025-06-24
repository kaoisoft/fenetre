use crate::ChildWindow;
use crate::child_window::{
  ChildType,
  Layout,
  LayoutArgs,
  LayoutFill,
  LayoutType,
  Orientation,
  UserEvent,
};
use crate::layout_base::LayoutBase;

use winit::event_loop::EventLoopProxy;

use tiny_skia::{Pixmap, PixmapPaint, Transform};

use uuid::Uuid;

use std::{
  cell::RefCell,
  collections::HashMap,
  fmt::Debug,
  rc::Rc,
};

pub struct LayoutData {
  pub index: usize,             // First child has an index of 0
  pub child: ChildType,
  pub x: f64,                   // Child's location within the layout
  pub y: f64,
}

pub struct RowLayout {
  layout_base: LayoutBase,
  orientation: Orientation,               // Vertical or Horizontal
  inner_padding: f64,                     // Padding between children, in pixels
  children: Vec<Box<LayoutData>>,         // List of children
  fill_algorithm: Box<LayoutFill>,
  max_width: Option<f64>,
  max_height: Option<f64>,
}

/// Layout in which all of the children are drawn in a single row or column
impl RowLayout {

  pub fn new(
        event_loop: Rc<EventLoopProxy<UserEvent>>,
        main_win_uuid: Uuid,
        orientation: Orientation,
        inner_padding: f64
  ) -> Self {

    let layout_base = LayoutBase::new(event_loop, main_win_uuid);

    Self {
      layout_base: layout_base,
      orientation: orientation,
      inner_padding: inner_padding,
      children: Vec::new(),
      fill_algorithm: Box::new(LayoutFill::Unused),
      max_width: None,
      max_height: None,
    }
  }

  /// Returns the number of children
  pub fn get_child_count(&self) -> usize {
    self.children.len()
  }

  /// Returns the child with the specified zero-based index
  pub fn get_child_with_index(&self, index: usize) -> Option<ChildType> {

    if self.children.len() >= index {
      return None;
    }

    Some(self.children.get(index).unwrap().child.clone())
  }

  /// Inserts a window at the specified zero-based index
  pub fn insert_child(&mut self, child: Rc<RefCell<dyn ChildWindow>>, index: usize) {

    let layout_data = LayoutData {
      index: self.children.len(),
      child: ChildType::Window(child),
      x: 0.0,
      y: 0.0,
    };
    self.children.insert(index, Box::new(layout_data));
  }

  /// Inserts a layout at the specified zero-based index
  pub fn insert_layout(&mut self, layout: Rc<RefCell<dyn Layout>>, index: usize) {

    let layout_data = LayoutData {
      index: self.children.len(),
      child: ChildType::Layout(layout),
      x: 0.0,
      y: 0.0,
    };
    self.children.insert(index, Box::new(layout_data));
  }
}

impl Debug for RowLayout {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "RowLayout; UUID: {}, orientation: {:?}, name: {}",
          self.layout_base.get_uuid(),
          self.orientation,
          self.layout_base.get_name()
    )
   }
}

impl Layout for RowLayout {

  /// args is ignored, so it can be any of the enum values.
  fn add_child(&mut self, child: Rc<RefCell<dyn ChildWindow>>,
      _args: LayoutArgs) -> Result<(), String> {

    let layout_data = LayoutData {
      index: self.children.len(),
      child: ChildType::Window(child),
      x: 0.0,
      y: 0.0,
    };
    self.children.push(Box::new(layout_data));

    Ok(())
  }

  fn add_layout(&mut self, layout: Rc<RefCell<dyn Layout>>,
      _args: LayoutArgs) -> Result<(), String> {

    let layout_data = LayoutData {
      index: self.children.len(),
      child: ChildType::Layout(layout),
      x: 0.0,
      y: 0.0,
    };
    self.children.push(Box::new(layout_data));

    Ok(())
  }

  fn clear(&mut self) {
    self.children.clear();
  }

  /// The only layout that performs any actions in this method is the TabLayout
  fn close_tab(&mut self, _tab_uuid: Uuid) {
  }

  fn get_child_at(&self, x: f64, y: f64) -> Option<Rc<RefCell<dyn ChildWindow>>> {

    for child_index in 0..self.children.len() {
      let child_data = self.children.get(child_index).unwrap();

      // If this child is a window, see if the point is within the child. Otherwise,
      // see if the point is within any of the layout's children.
      match &child_data.child {

        ChildType::Window(window) => {

          // Get a reference to the child
          let child_ref = window.borrow();

          // Is the mouse over top of this child?
          let (child_x, child_y) = child_ref.get_location();  // relative to the main window
          let (child_width, child_height) = child_ref.get_drawing_size();
          if x >= child_x && x <= child_x + child_width &&
              y >= child_y && y <= child_y + child_height {
            return Some(window.clone());
          }
        },

        ChildType::Layout(layout) => {

          let layout_ref = layout.borrow();

          // Check this layout for a matching child
          match layout_ref.get_child_at(x - child_data.x, y - child_data.y) {
            Some(window) => {
              return Some(window);
            },
            None => {},
          }
        },
      };
    }

    None
  }

  /// Gets the child with the specified ID
  fn get_child_with_id(&mut self, uuid: Uuid) -> Option<Rc<RefCell<dyn ChildWindow>>> {

    // Search all of the children
    for child_data in &self.children {

      match &child_data.child {

        ChildType::Window(window) => {

          // Get the mutable reference to the child
          let child_ref = window.borrow_mut();

          if child_ref.get_uuid() == uuid {
            return Some(window.clone())
          }
        },

        ChildType::Layout(layout) => {

          let mut layout_ref = layout.borrow_mut();

          match layout_ref.get_child_with_id(uuid) {
            Some(child) => return Some(child),
            None => {},
          }
        },
      }
    }

    None
  }

  fn get_layout_with_id(&self, uuid: Uuid) -> Option<Rc<RefCell<dyn Layout>>> {

    // Search all of the children
    for child_data in &self.children {

      match &child_data.child {

        ChildType::Window(_window) => {},    // we are only looking for layouts

        ChildType::Layout(layout) => {

          let layout_ref = layout.borrow();

          if layout_ref.get_uuid() == uuid {
            return Some(layout.clone());
          }

          match layout_ref.get_layout_with_id(uuid) {
            Some(layout) => return Some(layout),
            None => {},
          }
        },
      }
    }

    None
  }

  /// Gets the list of all descendent layouts of the specified type
  fn get_layouts_of_type(&self, layout_type: LayoutType) -> Vec<Rc<RefCell<dyn Layout>>> {

    let mut layouts:Vec<Rc<RefCell<dyn Layout>>> = Vec::new();

    // Search all of the children
    for child_data in &self.children {

      match &child_data.child {

        ChildType::Window(_window) => {},    // we are only looking for layouts

        ChildType::Layout(layout) => {

          let layout_ref = layout.borrow();

          if layout_ref.get_type() == layout_type.clone() {
            layouts.push(layout.clone());
          }

          let inner_layouts = layout_ref.get_layouts_of_type(layout_type.clone());
          if 0 < inner_layouts.len() {
            layouts.extend(inner_layouts);
          }
        }
      }
    }

    layouts
  }

  fn get_type(&self) -> LayoutType {
    LayoutType::RowLayout
  }

  fn layout(&mut self, main_win_x: f64, main_win_y: f64, width: f64, height: f64) -> Pixmap {

    // Save the layout's location within the main window
    self.layout_base.set_main_win_x(main_win_x);
    self.layout_base.set_main_win_y(main_win_y);

    // Save the size
    let layout_width = width;
    let layout_height = height;

    // Create the pixmap into which we will draw
    let mut pixmap = match Pixmap::new(width as u32, height as u32) {
      Some(pixmap) => pixmap,
      None => Pixmap::new(1, 1).unwrap(),
    };

    let paint = PixmapPaint::default();

    // Create a vector to hold the locations and sizes of the children.
    // This information will be calculates during several passes over the children.
    // Each element's key is the child's UUID, and the data is x, y, width, height.
    let mut drawing_info: HashMap<Uuid, (f64, f64, f64, f64)> = HashMap::new();

    // Calculate the amount of space used by all of the children, and set the
    // initial size for each child. The location will be set later and passed
    // to child via call to redraw() or layout().
    let mut used = 0;
    if 1 == self.children.len() {     // There is only one child, so give it all of the space

      // Add the size of the child
      let child_data = &self.children[0];
      used = match &child_data.child {

        ChildType::Window(window) => {

          let child_ref = window.borrow();

          // Initially, set the size to the child's unaltered drawing size
          drawing_info.insert(
                child_ref.get_uuid(),
                (
                      0.0,
                      0.0,
                      layout_width - (2.0 * self.inner_padding),
                      layout_height - (2.0 * self.inner_padding)
                )
          );

          match self.orientation {
            Orientation::Horizontal => layout_width as i32,
            Orientation::Vertical => layout_height as i32,
          }
        },

        ChildType::Layout(layout) => {

          let layout_ref = layout.borrow_mut();

          // Initially, set the size to the child's unaltered drawing size
          drawing_info.insert(
                layout_ref.get_uuid(),
                (
                      0.0,
                      0.0,
                      layout_width - (2.0 * self.inner_padding),
                      layout_height - (2.0 * self.inner_padding)
                )
          );

          match self.orientation {
            Orientation::Horizontal => layout_width as i32,
            Orientation::Vertical => layout_height as i32,
          }
        }
      };
    } else {

      for child_data in &mut self.children {

        // Add the padding in front of the child
        used += self.inner_padding as i32;

        // Add the size of the child
        used += match &child_data.child {

          ChildType::Window(window) => {

            let child_ref = window.borrow();
            let (child_width, child_height) = child_ref.get_drawing_size();

            // Initially, set the size to the child's unaltered drawing size
            drawing_info.insert(
                  child_ref.get_uuid(),
                  (0.0, 0.0, child_width, child_height)
            );

            match self.orientation {
              Orientation::Horizontal => child_width as i32,
              Orientation::Vertical => child_height as i32,
            }
          },

          ChildType::Layout(layout) => {
            let layout_ref = layout.borrow();
            let (layout_width, layout_height) = layout_ref.get_size();

            // Initially, set the size to the child's unaltered drawing size
            drawing_info.insert(
                  layout_ref.get_uuid(),
                  (0.0, 0.0, layout_width, layout_height)
            );

            match self.orientation {
              Orientation::Horizontal => layout_width as i32,
              Orientation::Vertical => layout_height as i32,
            }
          }
        };
      }
    }

    // Add the padding after the last child
    used += self.inner_padding as i32;

    // Check to see if the total used space is bigger than the layout
    let mut too_big = 0.0;
    match self.orientation {
      Orientation::Horizontal => {
        if used > width as i32 {
          too_big = used as f64 - width;
          used = width as i32;
        }
      },
      Orientation::Vertical => {
        if used > height as i32 {
          too_big = used as f64 - height;
          used = height as i32;
        }
      },
    }

    let mut remaining = self.children.len();  // # of children left to adjust
    let total_children = self.children.len(); // total # of children

      // If the total size of the children is too big, reduce them appropriately
    if too_big > 0.0 {

      // Adjust the window's with minimum sizes first
      used = 0;
      for child_data in &mut self.children {

        match &child_data.child {

          ChildType::Window(window) => {

            let window_ref = window.borrow();
            match window_ref.get_min_size() {

              Some((min_width, min_height)) => {
                let window_uuid = window_ref.get_uuid();
                let (x, y, mut width, mut height) = drawing_info.get(&window_uuid).unwrap();

                match self.orientation {

                  Orientation::Horizontal => {
                    let amount = width - min_width;
                    too_big -= amount;
                    width = min_width;
                    used += min_width as i32;
                    drawing_info.insert(window_uuid, (*x, *y, width, height));
                  },

                  Orientation::Vertical => {
                    let amount = height - min_height;
                    too_big -= amount;
                    height = min_height;
                    used += min_height as i32;
                    drawing_info.insert(window_uuid, (*x, *y, width, height));
                  },
                }

                remaining -= 1;
              },

              None => {},
            }
          },

          ChildType::Layout(_layout) => {},  // layout's don't have minimum sizes
        }
      }

      if too_big > 0.0 {

        // Adjust the size of the children without minimum sizes
        let equal_reduction = too_big / remaining as f64;
        for child_data in &mut self.children {

          match &child_data.child {

            ChildType::Window(window) => {

              let window_ref = window.borrow();
              let window_uuid = window_ref.get_uuid();
              let (x, y, mut child_width, mut child_height) = drawing_info.get(&window_uuid).unwrap();

              match window_ref.get_min_size() {

                Some((_min_width, _min_height)) => {},

                None => {

                  match self.orientation {

                    Orientation::Horizontal => {
                      child_width -= equal_reduction;
                      used += child_width as i32;
                    },

                    Orientation::Vertical => {
                      child_height -= equal_reduction;
                      used += child_height as i32;
                    },
                  }

                  // Update the size
                  drawing_info.insert(window_uuid, (*x, *y, child_width, child_height));
                },
              }
            },

            ChildType::Layout(layout) => {

              let layout_ref = layout.borrow();
              let layout_uuid = layout_ref.get_uuid();
              let (x, y, mut layout_width, mut layout_height) = drawing_info.get(&layout_uuid).unwrap();

              match self.orientation {

                Orientation::Horizontal => {
                  layout_width -= equal_reduction;
                  used += layout_width as i32;
                },

                Orientation::Vertical => {
                  layout_height -= equal_reduction;
                  used += layout_height as i32;
                },
              }

              // Update the size
              drawing_info.insert(layout_uuid, (*x, *y, layout_width, layout_height));
            },
          }
        }
      }
    }

    // Calculate the amount of extra space
    let extra = match *self.fill_algorithm {
      LayoutFill::Single(_child_type) => {
        match self.orientation {
          Orientation::Horizontal => width as i32 - used,
          Orientation::Vertical => height as i32 - used,
        }
      },
      LayoutFill::Evenly => {
        let padding = self.inner_padding * (self.children.len() + 2) as f64;
        used += padding as i32;

        match self.orientation {
          Orientation::Horizontal => (width as i32 - used) / self.children.len() as i32,
          Orientation::Vertical => (height as i32 - used) / self.children.len() as i32,
        }
      },
      LayoutFill::Unused => 0,
    };

    if extra > 0 {

      // Add the extra to the appropriate children
      for child_data in &mut self.children {

        match &child_data.child {

          ChildType::Window(window) => {

            let window_ref = window.borrow();
            let window_uuid = window_ref.get_uuid();
            let (x, y, mut child_width, mut child_height) = drawing_info.get(&window_uuid).unwrap();

            match *self.fill_algorithm {

              LayoutFill::Single(uuid) => {
                if uuid == window_uuid {

                  // Give all the extra to this child
                  match self.orientation {

                    Orientation::Horizontal => child_width += extra as f64,
                    Orientation::Vertical => child_height += extra as f64,
                  }
                  drawing_info.insert(window_uuid, (*x, *y, child_width, child_height));
                }
              },

              LayoutFill::Evenly => {

                  // Give an equal share of the extra to this child
                  match self.orientation {

                    Orientation::Horizontal => child_width += (extra / total_children as i32) as f64,
                    Orientation::Vertical => child_height += (extra / total_children as i32) as f64,
                  }
                  drawing_info.insert(window_uuid, (*x, *y, child_width, child_height));
              },

              LayoutFill::Unused => {},
            }
          },

          ChildType::Layout(layout) => {

            let layout_ref = layout.borrow();
            let layout_uuid = layout_ref.get_uuid();
            let (x, y, mut layout_width, mut layout_height) = drawing_info.get(&layout_uuid).unwrap();

            match *self.fill_algorithm {

              LayoutFill::Single(uuid) => {
                if uuid == layout_uuid {

                  // Give all the extra to this child
                  match self.orientation {

                    Orientation::Horizontal => layout_width += extra as f64,
                    Orientation::Vertical => layout_height += extra as f64,
                  }
                  drawing_info.insert(layout_uuid, (*x, *y, layout_width, layout_height));
                }
              },

              LayoutFill::Evenly => {

                  // Give an equal share of the extra to this child
                  match self.orientation {

                    Orientation::Horizontal => layout_width += (extra / total_children as i32) as f64,
                    Orientation::Vertical => layout_height += (extra / total_children as i32) as f64,
                  }
                  drawing_info.insert(layout_uuid, (*x, *y, layout_width, layout_height));
              },

              LayoutFill::Unused => {},
            }
          },
        }
      }
    }

    // Set the initial coordinates for the children
    let mut x = 0.0;
    let mut y = 0.0;

    // Set the coordinates for the children
    for child_data in &self.children {

      // Add the padding before this child
      match self.orientation {
        Orientation::Horizontal => x += self.inner_padding,
        Orientation::Vertical => y += self.inner_padding,
      }

      let uuid = match &child_data.child {
        ChildType::Window(window) => {
          let window_ref = window.borrow();
          window_ref.get_uuid()
        },
        ChildType::Layout(layout) => {
          let layout_ref = layout.borrow();
          layout_ref.get_uuid()
        },
      };

      // Set the child's location
      let (_child_x, _child_y, child_width, child_height) = *(drawing_info.get(&uuid).unwrap());
      drawing_info.insert(uuid, (x, y, child_width, child_height));

      // Update the location for the next child
      match self.orientation {
        Orientation::Horizontal => x += child_width,
        Orientation::Vertical => y += child_height,
      }
    }

    // Draw each child
    for child_data in &mut self.children {

      let uuid = match &child_data.child {
        ChildType::Window(window) => {
          let window_ref = window.borrow();
          window_ref.get_uuid()
        },
        ChildType::Layout(layout) => {
          let layout_ref = layout.borrow();
          layout_ref.get_uuid()
        },
      };

      let (x, y, mut new_width, mut new_height) = drawing_info.get(&uuid).unwrap();

      // Set the unaffected dimension to the layout's
      match self.orientation {
        Orientation::Horizontal => new_height = layout_height,
        Orientation::Vertical => new_width = layout_width,
      }

      let child_pixmap = match &child_data.child {

        ChildType::Window(window) => {

          let mut window_ref = window.borrow_mut();
          window_ref.redraw(
                self.layout_base.get_main_win_x() + *x as f64,
                self.layout_base.get_main_win_y() + *y as f64,
                new_width,
                new_height,
                false
          )
        },

        ChildType::Layout(layout) => {

          let mut layout_ref = layout.borrow_mut();
          layout_ref.layout(
                  self.layout_base.get_main_win_x() + *x as f64,
                  self.layout_base.get_main_win_y() + *y as f64,
                  new_width,
                  new_height
          )
        },
      };

      // Copy the child's pixmap image onto the full pixmap
      pixmap.draw_pixmap(
          *x as i32,
          *y as i32,
          child_pixmap.as_ref(),
          &paint,
          Transform::identity(),
          None,
      );
    }

    self.layout_base.set_pixmap(pixmap.clone());

    pixmap
  }

  /// The only layout that performs any actions in this method is the TabLayout
  fn set_active_tab_by_uuid(&mut self, _tab_uuid: Uuid) {
  }

  fn set_fill(&mut self, algorithm: Box<LayoutFill>) {
    self.fill_algorithm = algorithm;
  }

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

  // Calculates the minimum size needed to display children with padding
  fn get_size(&self) -> (f64, f64) {

    let mut width;
    let mut height;
    match self.orientation {
      Orientation::Horizontal => {
        width = self.inner_padding;
        height = 0.0;
      },
      Orientation::Vertical => {
        width = 0.0;
        height = self.inner_padding;
      },
    }

    for child_data in &self.children {

      match &child_data.child {

        ChildType::Window(window) => {

          // Get a reference to the child
          let child_ref = window.borrow();

          let (child_width, child_height) = child_ref.get_drawing_size();
          match self.orientation {
            Orientation::Horizontal => {
              width += child_width;
              width += self.inner_padding;
              if child_height > height {
                height = child_height;
              }
            },
            Orientation::Vertical => {
              if child_width > width {
                width = child_width;
              }
              height += child_height;
              height += self.inner_padding;
            },
          }
        },

        ChildType::Layout(layout) => {

          // Get a reference to the child
          let child_ref = layout.borrow();

          let (layout_width, layout_height) = child_ref.get_size();
          match self.orientation {
            Orientation::Horizontal => {
              width += layout_width;
              width += self.inner_padding;
              if layout_height > height {
                height = layout_height;
              }
            },
            Orientation::Vertical => {
              if layout_width > width {
                width = layout_width;
              }
              height += layout_height;
              height += self.inner_padding;
            },
          }
        },
      };
    }

    // Add extra for the edge padding
    width += 2.0 * self.inner_padding;
    height += 2.0 * self.inner_padding;

    (width, height)
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
