use winit::{
  event::MouseButton,
  event_loop::EventLoopProxy,
};

use tiny_skia::{
  Color,
  Pixmap,
};

use uuid::Uuid;

use crate::button::Button;
use crate::ChildType;
use crate::child_window::{
  ChildWindow,
  ContextMenuItem,
};
use crate::context_menu::ContextMenu;
use crate::UserEvent;
use crate::window_utils::WindowUtils;

use std::{
  cell::RefCell,
  rc::Rc,
};

/// Base child window object that is contained within each concrete window
#[derive(Clone, Debug)]
pub struct WindowBase {
  window_type: String,                        // struct type of the window
  event_loop: Rc<EventLoopProxy<UserEvent>>,  // main window's event loop
  uuid: Uuid,                                 // window's unique ID
  main_win_uuid: Uuid,                        // ID of the outermost parent window
  enabled: bool,                              // whether the window is enabled
  focused: bool,                              // whether the window has input focus
  main_win_x: f64,                            // window's location within the main window
  main_win_y: f64,
  width: f64,                                 // size of the window
  height: f64,
  layout_x: f64,                              // location within the layout
  layout_y: f64,
  min_size: Option<(f64, f64)>,               // smallest size that the sindow can be
  max_size: Option<(f64, f64)>,               // largest size that the window can be
  x_scroll: f64,                              // horizontal scroll amount
  x_scroll_min: f64,
  x_scroll_max: f64,
  y_scroll: f64,                              // vertical scroll amount
  y_scroll_min: f64,
  y_scroll_max: f64,
  pixmap: Pixmap,                             // window's contents
  text: Option<String>,                       // text associated with the window
  background_color: Color,                    // solid background color
  name: String,                               // only used in Debug
  context_menu_items: Vec<Rc<RefCell<Button>>>, // Items for a window's context menu
  parent: Option<ChildType>,                  // parent of this window
  tooltip_text: Option<String>,               // tooltip text
}

impl WindowBase {

  pub fn new(event_loop: Rc<EventLoopProxy<UserEvent>>, main_win_uuid: Uuid) -> Self {

    // Set all of the default values
    Self {
      window_type: "unknown".to_string(),
      event_loop: event_loop,
      uuid: Uuid::new_v4(),
      main_win_uuid: main_win_uuid,
      enabled: true,
      focused: false,
      main_win_x: 0.0,
      main_win_y: 0.0,
      width: 0.0,
      height: 0.0,
      layout_x: 0.0,
      layout_y: 0.0,
      min_size: None,
      max_size: None,
      x_scroll: 0.0,
      x_scroll_min: 0.0,
      x_scroll_max: 0.0,
      y_scroll: 0.0,
      y_scroll_min: 0.0,
      y_scroll_max: 0.0,
      pixmap: Pixmap::new(1, 1).unwrap(),   // empty pixmap
      text: None,
      background_color: Color::WHITE,
      name: "unspecified".to_string(),
      context_menu_items: Vec::new(),
      parent: None,
      tooltip_text: None,
    }
  }

  pub fn add_context_menu_item(
        &mut self,
        item: Box<ContextMenuItem>
  ) {

    // Create a Button for this item
    let btn = Button::new(
          self.event_loop.clone(),
          self.main_win_uuid,
          Some(item.label),
          None,
          None,
          Color::WHITE,
          item.callback
    );

    // Add this button to the list
    self.context_menu_items.push(Rc::new(RefCell::new(btn)));
  }
  pub fn add_context_menu_separator(&mut self) {

    // Create a Button for the separator
    let btn = Button::new(
      self.event_loop.clone(),
      self.main_win_uuid,
      Some("----------".to_string()),
      None,
      None,
      Color::WHITE,
      ||{}
    );

    // Add this button to the list
    self.context_menu_items.push(Rc::new(RefCell::new(btn)));
  }
  pub fn get_context_menu_items(&self) -> Vec<Rc<RefCell<Button>>> {
    self.context_menu_items.clone()
  }

  pub fn get_uuid(&self) -> Uuid {
    self.uuid
  }
  pub fn set_uuid(&mut self, uuid: Uuid) {
    self.uuid = uuid;
  }
  pub fn get_main_win_uuid(&self) -> Uuid {
    self.main_win_uuid
  }

  pub fn get_name(&self) -> String {
    self.name.clone()
  }
  pub fn set_name(&mut self, name: String) {
    self.name = name;
  }

  pub fn get_window_type(&self) -> String {
    self.window_type.clone()
  }
  pub fn set_window_type(&mut self, window_type: String) {
    self.window_type = window_type;
  }

  pub fn get_event_loop(&self) -> Rc<EventLoopProxy<UserEvent>> {
    self.event_loop.clone()
  }
  pub fn set_event_loop(&mut self, event_loop: Rc<EventLoopProxy<UserEvent>>) {
    self.event_loop = event_loop;
  }

  pub fn get_enabled(&self) -> bool {
    self.enabled
  }
  pub fn set_enabled(&mut self, enabled: bool) {
    self.enabled = enabled;
  }

  pub fn get_focused(&self) -> bool {
    self.focused
  }
  pub fn set_focused(&mut self, focused: bool) {
    self.focused = focused;
  }

  // Location related functions
  pub fn get_location(&self) -> (f64, f64) {
    (self.main_win_x, self.main_win_y)
  }
  pub fn set_location(&mut self, x: f64, y: f64) {
    self.main_win_x = x;
    self.main_win_y = y;
  }
  pub fn get_x(&self) -> f64 {
    self.main_win_x
  }
  pub fn set_x(&mut self, x: f64) {
    self.main_win_x = x;
  }
  pub fn get_y(&self) -> f64 {
    self.main_win_y
  }
  pub fn set_y(&mut self, y: f64) {
    self.main_win_y = y;
  }
  pub fn get_layout_location(&self) -> (f64, f64) {
    (self.layout_x, self.layout_y)
  }
  pub fn set_layout_location(&mut self, x: f64, y: f64) {
    self.layout_x = x;
    self.layout_y = y;
  }

  // Size related functions
  pub fn get_size(&self) -> (f64, f64) {
    (self.width, self.height)
  }
  pub fn set_size(&mut self, width: f64, height: f64) {
    self.width = width;
    self.height = height;
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
  pub fn get_min_size(&self) -> Option<(f64, f64)> {
    self.min_size
  }
  pub fn set_min_size(&mut self, width: f64, height: f64) {
    self.min_size = Some((width, height));
  }
  pub fn get_max_size(&self) -> Option<(f64, f64)> {
    self.max_size
  }
  pub fn set_max_size(&mut self, width: f64, height: f64) {
    self.max_size = Some((width, height));
  }
  /// Gets the size for drawing the window.
  ///
  /// This takes into account the max size, full size, and actual size, in that order.
  pub fn get_drawing_size(&self) -> (f64, f64) {

    let mut width = self.width;
    let mut height = self.height;

    // Ensure that the window isn't smaller than the minimum size
    match self.min_size {
      Some((min_width, min_height)) => {
        if width < min_width {
          width = min_width;
        }
        if height < min_height {
          height = min_height;
        }
      },

      None => {},
    }

    // Ensure that the window isn't larger than the maximum size
    match self.max_size {
      Some((max_width, max_height)) => {
        if width > max_width {
          width = max_width;
        }
        if height > max_height {
          height = max_height;
        }
      },

      None => {},
    }

    (width, height)
  }

  pub fn get_x_scroll(&self) -> f64 {
    self.x_scroll
  }
  pub fn set_x_scroll(&mut self, x_scroll: f64) {
    self.x_scroll = x_scroll;
  }
  pub fn get_x_scroll_min(&self) -> f64 {
    self.x_scroll_min
  }
  pub fn set_x_scroll_min(&mut self, value: f64) {
    self.x_scroll_min = value;
  }
  pub fn get_x_scroll_max(&self) -> f64 {
    self.x_scroll_max
  }
  pub fn set_x_scroll_max(&mut self, value: f64) {
    self.x_scroll_max = value;
  }

  pub fn get_y_scroll(&self) -> f64 {
    self.y_scroll
  }
  pub fn set_y_scroll(&mut self, y_scroll: f64) {
    self.y_scroll = y_scroll;
  }
  pub fn get_y_scroll_min(&self) -> f64 {
    self.y_scroll_min
  }
  pub fn set_y_scroll_min(&mut self, value: f64) {
    self.y_scroll_min = value;
  }
  pub fn get_y_scroll_max(&self) -> f64 {
    self.y_scroll_max
  }
  pub fn set_y_scroll_max(&mut self, value: f64) {
    self.y_scroll_max = value;
  }

  pub fn get_pixmap(&self) -> Pixmap {
    self.pixmap.clone()
  }
  pub fn set_pixmap(&mut self, pixmap: Pixmap) {
    self.pixmap = pixmap;
  }

  pub fn get_text(&self) -> Option<String> {

    match &self.text {
      Some(text) => Some(text.clone()),
      None => None,
    }
  }
  pub fn set_text(&mut self, text: String) {
    self.text = Some(text);
  }

  pub fn get_background_color(&self) -> Color {
    self.background_color
  }
  pub fn set_background_color(&mut self, color: Color) {
    self.background_color = color;
  }

  pub fn get_parent(&self) -> Option<ChildType> {
    self.parent.clone()
  }
  pub fn set_parent(&mut self, parent: Option<ChildType>) {
    self.parent = parent;
  }

  pub fn get_tooltip_text(&self) -> Option<String> {
    self.tooltip_text.clone()
  }
  pub fn set_tooltip_text(&mut self, text: String) {
    self.tooltip_text = Some(text);
  }

  pub fn handle_mouse_pressed(&mut self, button: MouseButton,
    mouse_x: f64, mouse_y: f64) {

    // If the user pressed the right button, display the context menu, if
    // there is one.
    match button {

      MouseButton::Left => {
      },

      MouseButton::Right => {

        if 0 < self.get_context_menu_items().len() {

          // Create a context menu
          WindowUtils::fire_user_event(self.get_event_loop(),
                UserEvent::ShowContextMenu(
                  self.get_main_win_uuid(),
                  self.get_uuid(),
                  mouse_x,
                  mouse_y,
                )
          );
        }
      },

      _ => {},
    }
  }

  pub fn populate_context_menu(&self, context_menu_rc: Rc<RefCell<ContextMenu>>) {

    let mut context_menu = context_menu_rc.borrow_mut();
    context_menu.clear();

    let mut max_width = 0.0;
    let mut total_height = 0.0;
    for item in &self.context_menu_items {

      // Add this item's height to the total
      let btn = item.borrow();
      let (width, height) = btn.get_drawing_size();
      total_height += height;

      if width > max_width {
        max_width = width;
      }

      // Add this item to the menu
      context_menu.add_item(item.clone());
    }

    // Resize the menu
    context_menu.set_size(max_width, total_height);
  }

  pub fn update(&mut self) {

    WindowUtils::request_redraw(
      self.event_loop.clone(),
      self.main_win_uuid,
      self.main_win_x,
      self.main_win_y,
      self.pixmap.clone()
    )
  }
}
