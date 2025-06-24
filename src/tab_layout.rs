use crate::button::Button;
use crate::ChildWindow;
use crate::child_window::{
  ChildType,
  Layout,
  LayoutArgs,
  LayoutFill,
  LayoutType,
};
use crate::layout_base::LayoutBase;
use crate::text_font::TextFont;
use crate::UserEvent;
use crate::window_utils::WindowUtils;

use winit::event_loop::EventLoopProxy;

use tiny_skia::{
  Color,
  Paint,
  PathBuilder,
  Pixmap,
  PixmapPaint,
  Stroke,
  Transform,
};

use uuid::Uuid;

use std::{
  cell::RefCell,
  fmt::Debug,
  rc::Rc,
};

const TOP_RIGHT_PADDING: f32 = 6.0;

// Information for each tab
struct Tab {
  pub uuid: Uuid,
  pub title: String,
  pub close_btn: Rc<RefCell<Button>>,
  pub contents: ChildType,
  pub top_start_x: f32,         // X coordinate where the tab top starts
  pub top_width: f32,           // width of the tab's top
}

/// Layout in which each child is in a separate tab
pub struct TabLayout {
  layout_base: LayoutBase,
  font: Option<TextFont>,
  tabs: Vec<Rc<RefCell<Tab>>>,
  active_tab: Option<Uuid>,
  tab_top_height: f64,          // Height of the top of the tab (title and close button)
  max_width: Option<f64>,
  max_height: Option<f64>,
}

impl TabLayout {

  pub fn new(
        event_loop: Rc<EventLoopProxy<UserEvent>>,
        main_win_uuid: Uuid,
  ) -> TabLayout {

    // Load the font
    let font = match TextFont::new("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", 14.0) {
      Ok(font) => Some(font),
      Err(_err) => None,
    };

    let layout_base = LayoutBase::new(event_loop, main_win_uuid);

    Self {
      layout_base: layout_base,
      font: font,
      tabs: Vec::new(),
      active_tab: None,
      tab_top_height: 30.0,
      max_width: None,
      max_height: None,
    }
  }

  /// Adds a new tab to the layout
  pub fn add_tab(&mut self, title: String, contents: ChildType) {

    let tab_uuid = Uuid::new_v4();
    let tab_uuid_clone = tab_uuid.clone();
    let layout_uuid = self.layout_base.get_uuid().clone();

    // Create the close button
    let event_loop_clone = self.layout_base.get_event_loop().clone();
    let main_win_uuid_clone = self.layout_base.get_main_win_uuid().clone();
    let btn = Button::new(
          self.layout_base.get_event_loop().clone(),
          self.layout_base.get_main_win_uuid().clone(),
          Some("x".to_string()),
          None,
          None,
          Color::from_rgba8(191, 191, 191, 255),
          move || {
            WindowUtils::fire_user_event(
                  event_loop_clone.clone(),
                  UserEvent::CloseTab(
                        main_win_uuid_clone,
                        layout_uuid,
                        tab_uuid_clone
                  )
            );
          }
    );

    // Create the Tab
    let tab = Tab {
      uuid: tab_uuid,
      title: title,
      close_btn: Rc::new(RefCell::new(btn)),
      contents: contents,
      top_start_x: 0.0,
      top_width: 0.0,
    };
    let tab_rc = Rc::new(RefCell::new(tab));

    // Add the new tab
    self.tabs.push(tab_rc.clone());

    // Set this as the active tab
    let tab_ref = tab_rc.borrow();
    self.active_tab = Some(tab_ref.uuid);

    // Request a redraw
    WindowUtils::request_full_redraw(
          self.layout_base.get_event_loop().clone(),
          self.layout_base.get_main_win_uuid()
    );
  }

  fn get_active_tab(&self) -> Option<Rc<RefCell<Tab>>> {

    match self.active_tab {

      Some(active_tab_uuid) => {

        // Find the active tab
        for tab in &self.tabs {

          let tab_ref = tab.borrow();
          if tab_ref.uuid == active_tab_uuid {
            return Some(tab.clone());
          }
        }
      },

      None => {},
    }

    None
  }

  /// Sets the active tab.
  ///
  /// index - the zero-based index of the tab to make active
  pub fn set_active_tab(&mut self, index: usize) {

    if index >= self.tabs.len() {
      return;
    }

    match self.tabs.get(index) {

      Some(tab_rc) => {
        let tab_ref = tab_rc.borrow();
        self.active_tab = Some(tab_ref.uuid);
        WindowUtils::request_full_redraw(
              self.layout_base.get_event_loop().clone(),
              self.layout_base.get_main_win_uuid()
        );
      },

      None => {},
    }
  }
}

impl Debug for TabLayout {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    write!(fmt, "TabLayout; UUID: {}, name: {}",
          self.layout_base.get_uuid(), self.layout_base.get_name())
   }
}

impl Layout for TabLayout {

  /// Use add_tab() to add a tab
  fn add_child(&mut self, _child: Rc<RefCell<dyn ChildWindow>>,
      _args: LayoutArgs) -> Result<(), String> {
    Err("Use TabLayout::add_tab() to add a tab".to_string())
  }

  /// Use add_tab() to add a tab
  fn add_layout(&mut self, _layout: Rc<RefCell<dyn Layout>>,
      _args: LayoutArgs) -> Result<(), String> {
    Err("Use TabLayout::add_tab() to add a tab".to_string())
  }

  fn clear(&mut self) {
    self.tabs.clear();
    self.active_tab = None;
  }

  /// The only layout that performs any actions in this method is the TabLayout
  fn close_tab(&mut self, tab_uuid: Uuid) {

    // Cannot remove the only tab
    if self.tabs.len() == 1 {
      return;
    }

    // Get the tab's index within the Vec
    let mut index: usize = 0;
    let mut i: usize = 0;
    for tab in &self.tabs {

      let tab_ref = tab.borrow();

      if tab_ref.uuid == tab_uuid {
        index = i;
        break;
      }

      i += 1;
    }

    if index < self.tabs.len() {

      // Remove the tab
      self.tabs.remove(index);

      // Make the first tab the active tab
      let tab_ref = self.tabs.get(0).unwrap().borrow();
      self.active_tab = Some(tab_ref.uuid);

      // Request a redraw
      WindowUtils::request_full_redraw(
            self.layout_base.get_event_loop().clone(),
            self.layout_base.get_main_win_uuid()
      );
    }
  }

  fn get_child_at(&self, x: f64, y: f64) -> Option<Rc<RefCell<dyn ChildWindow>>> {

    // If the location is within the title of a tab, make that tab the active one. If
    // the location is within the active tab's contents, return the tab's contents.
    if y - self.layout_base.get_main_win_y() <= self.tab_top_height {

      for tab in &self.tabs {

        let tab_ref = tab.borrow_mut();

        if x >= tab_ref.top_start_x as f64 && x <= (tab_ref.top_start_x + tab_ref.top_width) as f64 {

          // Make this the active tab
          match self.active_tab {

            Some(active_tab) => {     // There is already an active tab

              if active_tab != tab_ref.uuid {

                // If the location is on the close_button, return the button; otherwise,
                // return None, but fire the TabSelected event
                let btn_pixmap = tab_ref.close_btn.borrow_mut().redraw(x as f64, 2.0, 1.0, 1.0, false);   // width & height are ignore by Button
                if x >= (tab_ref.top_start_x + tab_ref.top_width - TOP_RIGHT_PADDING - btn_pixmap.width() as f32) as f64 &&
                      x <= (tab_ref.top_start_x + tab_ref.top_width - TOP_RIGHT_PADDING) as f64 {
                  return Some(tab_ref.close_btn.clone());
                } else {
                  WindowUtils::fire_user_event(
                        self.layout_base.get_event_loop().clone(),
                        UserEvent::TabSelected(
                              self.layout_base.get_main_win_uuid(),
                              self.layout_base.get_uuid(),
                              tab_ref.uuid
                        )
                  );
                  return None;
                }
              } else {

                // If the location is on the close_button, return the button; otherwise,
                // return None.
                let btn_pixmap = tab_ref.close_btn.borrow_mut().redraw(x as f64, 2.0, 1.0, 1.0, false);   // width & height are ignore by Button
                if x >= (tab_ref.top_start_x + tab_ref.top_width - TOP_RIGHT_PADDING - btn_pixmap.width() as f32) as f64 &&
                      x <= (tab_ref.top_start_x + tab_ref.top_width - TOP_RIGHT_PADDING) as f64 {
                  return Some(tab_ref.close_btn.clone());
                } else {
                  return None;
                }
              }
            },
            None => {     // There is no currently active tab

              // If the location is on the close_button, return the button; otherwise,
              // return None, but fire the TabSelected event
              let btn_pixmap = tab_ref.close_btn.borrow_mut().redraw(x as f64, 2.0, 1.0, 1.0, false);   // width & height are ignore by Button
              if x >= (tab_ref.top_start_x + tab_ref.top_width - TOP_RIGHT_PADDING - btn_pixmap.width() as f32) as f64 &&
                    x <= (tab_ref.top_start_x + tab_ref.top_width - TOP_RIGHT_PADDING) as f64 {
                return Some(tab_ref.close_btn.clone());
              } else {
                WindowUtils::fire_user_event(
                      self.layout_base.get_event_loop().clone(),
                      UserEvent::TabSelected(
                            self.layout_base.get_main_win_uuid(),
                            self.layout_base.get_uuid(),
                            tab_ref.uuid
                      )
                );
                return None;
              }
            },
          }
        }
      }
    } else {      // Location is not within the tab tops

      match self.get_active_tab() {

        Some(active_tab) => {

          let active_tab_ref = active_tab.borrow();

          match &active_tab_ref.contents {

            ChildType::Window(window) => {
              return Some(window.clone());
            },

            ChildType::Layout(layout) => {

              let layout_ref = layout.borrow();

              return layout_ref.get_child_at(x, y);
            }
          }
        },

        None => {},
      }
    }

    None
  }

  /// Gets the child with the specified ID
  fn get_child_with_id(&mut self, uuid: Uuid) -> Option<Rc<RefCell<dyn ChildWindow>>> {

    // Search all of the children
    for tab in &mut self.tabs {

      let tab_ref = tab.borrow();
      match &tab_ref.contents {

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
    for tab in &self.tabs {

      let tab_ref = tab.borrow();
      match &tab_ref.contents {

        ChildType::Window(_window) => {},

        ChildType::Layout(layout) => {

          let layout_ref = layout.borrow();

          if layout_ref.get_uuid() == uuid {
            return Some(layout.clone());
          }
        },
      }
    }

    None
  }

  /// Gets the list of all descendent layouts of the specified type
  fn get_layouts_of_type(&self, layout_type: LayoutType) -> Vec<Rc<RefCell<dyn Layout>>> {

    let mut layouts:Vec<Rc<RefCell<dyn Layout>>> = Vec::new();

    for tab in &self.tabs {

      let tab_ref = tab.borrow();
      match &tab_ref.contents {

        ChildType::Window(_window) => {},

        ChildType::Layout(layout) => {

          let layout_ref = layout.borrow();

          if layout_ref.get_type() == layout_type.clone() {
            layouts.push(layout.clone());
          }

          let inner_layouts = layout_ref.get_layouts_of_type(layout_type.clone());
          if 0 < inner_layouts.len() {
            layouts.extend(inner_layouts);
          }
        },
      }
    }

    layouts
  }

  fn get_type(&self) -> LayoutType {
    LayoutType::TabLayout
  }

  fn layout(&mut self, main_win_x: f64, main_win_y: f64, width: f64, height: f64) -> Pixmap {

    // Save the location
    self.layout_base.set_main_win_x(main_win_x);
    self.layout_base.set_main_win_y(main_win_y);

    // Save the new size
    match self.max_width {
      Some(max_width) => {
        if width < max_width {
          self.layout_base.set_width(width);
        } else {
          self.layout_base.set_width(max_width);
        }
      },
      None => self.layout_base.set_width(width),
    }
    match self.max_height {
      Some(max_height) => {
        if height < max_height {
          self.layout_base.set_height(height);
        } else {
          self.layout_base.set_height(max_height);
        }
      },
      None => self.layout_base.set_height(height),
    }

    // Create the pixmap into which we will draw
    let mut pixmap = match Pixmap::new(width as u32, height as u32) {
      Some(pixmap) => pixmap,
      None => {
        println!("In TabLayout::layout(), cannot create pixmap of size {width} x {height}");
        Pixmap::new(1, 1).unwrap()
      },
    };

    let paint = PixmapPaint::default();

    // Create the stroke for drawing the tab edges
    let stroke = Stroke::default();   // One pixel wide

    // Draw each tab's top containing the title and close button
    let mut x = 0.0;
    match &self.font {

      Some(font) => {
        let mut start_x;
        for tab in &self.tabs {

          let mut tab_ref = tab.borrow_mut();

          // Draw the left edge of the tab top
          start_x = x;
          let mut path_builder = PathBuilder::new();
          path_builder.move_to(x, 0.0);
          path_builder.line_to(x, self.tab_top_height as f32);
          let path = path_builder.finish().unwrap();
          pixmap.stroke_path(&path, &Paint::default(), &stroke, Transform::identity(), None);
          x += 2.0;

          // Draw the text
          let text = tab_ref.title.clone();
          let (drawn_width, _drawn_height) = font.draw_text(
            text.as_str(),
            &mut pixmap,
            x as i32,
            6,
            Color::BLACK,
            Color::WHITE,
            -1,
            Color::BLACK,
            None
          );
          x += drawn_width as f32;

          // Draw the close button
          x += 15.0;
          let btn_pixmap = tab_ref.close_btn.borrow_mut().redraw(x as f64, 2.0, 1.0, 1.0, false);   // width & height are ignore by Button
          pixmap.draw_pixmap(
              x as i32,
              2,
              btn_pixmap.as_ref(),
              &paint,
              Transform::identity(),
              None,
          );
          x += btn_pixmap.width() as f32;

          // Add some padding on the right
          x += TOP_RIGHT_PADDING;

          // Save the start X coordinate and width of the tab's top
          tab_ref.top_start_x = start_x;
          tab_ref.top_width = x - start_x;

          // Draw the right edge of the tab top
          let mut path_builder = PathBuilder::new();
          path_builder.move_to(x, 0.0);
          path_builder.line_to(x, self.tab_top_height as f32);
          let path = path_builder.finish().unwrap();
          pixmap.stroke_path(&path, &Paint::default(), &stroke, Transform::identity(), None);

          // Draw the top edge of the tab top
          let mut path_builder = PathBuilder::new();
          path_builder.move_to(start_x, 0.0);
          path_builder.line_to(x, 0.0);
          let path = path_builder.finish().unwrap();
          pixmap.stroke_path(&path, &Paint::default(), &stroke, Transform::identity(), None);

          // If this is not the active tab, draw the bottom edge of the tab top
          match self.active_tab {
            Some(active_tab) => {
              if tab_ref.uuid != active_tab {
                let mut path_builder = PathBuilder::new();
                path_builder.move_to(start_x, self.tab_top_height as f32);
                path_builder.line_to(x, self.tab_top_height as f32);
                let path = path_builder.finish().unwrap();
                pixmap.stroke_path(&path, &Paint::default(), &stroke, Transform::identity(), None);
              }
            },
            None => {
              let mut path_builder = PathBuilder::new();
              path_builder.move_to(start_x, self.tab_top_height as f32);
              path_builder.line_to(x, self.tab_top_height as f32);
              let path = path_builder.finish().unwrap();
              pixmap.stroke_path(&path, &Paint::default(), &stroke, Transform::identity(), None);
            }
          }
        }
      },
      None => {},
    }

    // Draw the rest of the top line
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(x, self.tab_top_height as f32);
    path_builder.line_to(width as f32, self.tab_top_height as f32);
    let path = path_builder.finish().unwrap();
    pixmap.stroke_path(&path, &Paint::default(), &stroke, Transform::identity(), None);

    // Calculate the upper left and lower right coordinates for the contents' frame
    let contents_frame_ul_x = 0.0;
    let contents_frame_ul_y = self.tab_top_height as f32;
    let contents_frame_lr_x = width as f32;
    let contents_frame_lr_y = height as f32;

    // Draw the line around the tabs' contents
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(contents_frame_lr_x, contents_frame_ul_y);   // start at upper right corner
    path_builder.line_to(contents_frame_lr_x, contents_frame_lr_y);   // go to lower right corner
    path_builder.line_to(contents_frame_ul_x, contents_frame_lr_y);   // go to lower left corner
    path_builder.line_to(contents_frame_ul_x, contents_frame_ul_y);   // back to upper left corner
    let path = path_builder.finish().unwrap();
    pixmap.stroke_path(&path, &Paint::default(), &stroke, Transform::identity(), None);

    // Draw the contents of the active tab, leaving a one pixel margin all around it
    match self.active_tab {

      Some(active_tab_uuid) => {

        // Find the active tab
        for tab in &self.tabs {

          let tab_ref = tab.borrow();
          if tab_ref.uuid == active_tab_uuid {

            let child_pixmap = match &tab_ref.contents {

              ChildType::Window(window) => {

                // Get the mutable reference to the child
                let mut child_ref = window.borrow_mut();

                // Redraw the child
                child_ref.redraw(
                      main_win_x,
                      main_win_y + self.tab_top_height + 2.0,
                      (contents_frame_lr_x - contents_frame_ul_x - 4.0) as f64,
                      (contents_frame_lr_y - contents_frame_ul_y - 4.0) as f64,
                      false
                    )
              },

              ChildType::Layout(layout) => {

                // Get the mutable reference to the layout
                let mut layout_ref = layout.borrow_mut();

                // Redraw the layout
                layout_ref.layout(
                      main_win_x,
                      main_win_y + self.tab_top_height + 2.0,
                      (contents_frame_lr_x - contents_frame_ul_x - 4.0) as f64,
                      (contents_frame_lr_y - contents_frame_ul_y - 4.0) as f64
                )
              },
            };

            pixmap.draw_pixmap(
              contents_frame_ul_x as i32 + 2, // one for the frame and one for the margin
              contents_frame_ul_y as i32 + 2,
              child_pixmap.as_ref(),
              &PixmapPaint::default(),
              Transform::identity(),
              None,
            );
          }
        }
      },

      None => {},
    }

    // Save the pixmap
    self.layout_base.set_pixmap(pixmap.clone());

    pixmap
  }

  /// The only layout that performs any actions in this method is the TabLayout
  fn set_active_tab_by_uuid(&mut self, tab_uuid: Uuid) {
    self.active_tab = Some(tab_uuid);
    WindowUtils::request_full_redraw(self.layout_base.get_event_loop().clone(),
          self.layout_base.get_main_win_uuid());
  }

  fn set_fill(&mut self, _algorithm: Box<LayoutFill>) {
    // TabLayout doesn't use a fill algorithm
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
