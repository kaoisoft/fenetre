use winit::{
  event::{
    KeyEvent,
    MouseButton,
    MouseScrollDelta,
    TouchPhase,
  },
  event_loop::EventLoopProxy,
  window::{Window, WindowId},
};

use tiny_skia::{
  Color,
  Pixmap,
};

use uuid::Uuid;

use crate::context_menu::ContextMenu;

use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Orientation {
  Horizontal,
  Vertical,
}

/// Application-specific events sent to the event loop
///
/// The following events have default processing if no handler is set:
///   CreateWindow
///   Redraw
///   RedrawAll
#[derive(Debug, Clone)]
pub enum UserEvent {
  // Uuid is the editor window that generated the event
  // The first usize is the zero-based index of the line that the caret moved to
  // The second usize is the zero-based index of the character that the caret moved to
  CaretMoved(Uuid, usize, usize),
  // Uuid is the top-level parent window's ID
  // WindowId is the UUID of the window being closed
  ClosePopUp(Uuid, WindowId),
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ID of window that is the source of the event
  // The third Uuid is the ID of the tab being closed
  CloseTab(Uuid, Uuid, Uuid),
  // Creates a fully decorated window
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ID for the new window
  // The first two f64 values are the X and Y coordinates where the window will appear
  // The last two f64 values are the width and height of the window
  // The bool indicates whether the window is modal
  CreateWindow(Uuid, Uuid, f64, f64, f64, f64, bool),
  // Deletes the selected items within a ChildWindow
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ID for the window containing the items
  DeleteItems(Uuid, Uuid),
  // Sets the context menu to hidden
  // Uuid is the top-level parent window's ID
  HideContextMenu(Uuid),
  // Fired when the selected items within certain windows, such as List,
  //    should be processed.
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ID of the window in which the selected items exist
  ProcessSelectedItems(Uuid, Uuid),
  // Redraws a portion of the main window
  // Uuid is the top-level parent window's ID
  // The two f64 values are the upper left corner of where the Pixmap will be drawn
  // Pixmap is the image that will be drawn over the main window
  Redraw(Uuid, f64, f64, Pixmap),
  // Redraws the entire main window' contents
  // Uuid is the top-level parent window's ID
  RedrawAll(Uuid),
  // Fired when the value in a ScrollBar changes
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ID of the ScrollBar in which the selection changed
  // Orientation is the ScrollBar's orientation
  // f64 is the new value
  ScrollValueChanged(Uuid, Uuid, Orientation, f64),
  // Fired when the selected items within certain windows, such as List,
  //    changes.
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ID of the window in which the selection changed
  SelectionChanged(Uuid, Uuid),
  // Sets the data list in a window
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ID of the window whose data is being set
  // Vec<String> is the new data
  SetList(Uuid, Uuid, Vec<String>),
  // Displays the context menu
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ID of the window that fired the event
  // The two f64 values are the X and Y coordinates where the window will appear
  ShowContextMenu(Uuid, Uuid, f64, f64),
  // Displays a tooltip
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ID of the window that fired the event
  // The String is the new text for the tooltip
  ShowToolTip(Uuid, Uuid, String),
  // Fired when a Slider's value is changed
  // The f64 is the new value
  SliderValueChange(f64),
  // Fired when the user clicks on the title of a tab within a TabLayout
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ID of the TabLayout
  // The third Uuid is the ID of tab that was clicked
  TabSelected(Uuid, Uuid, Uuid),
  // Updates a ScrollLayout
  // The first Uuid is the top-level parent window's ID
  // The second Uuid is the ScrollLayout's ID
  UpdateScroller(Uuid, Uuid),
  // These are messages that are defined by an application's programmer
  // Uuid is the top-level parent window's ID
  // The u64 is a unique number assigned by the programmer
  // The Vec<String> is user-defined data associated with the event
  UserDefined(Uuid, u64, Vec<String>),
}

pub struct ContextMenuItem {
  pub label: String,
  pub callback: Box<dyn Fn()>
}

/// Trait for all child windows
pub trait ChildWindow : Debug {

  /// Called when a requested window is created.
  fn created_window(&self, window: Window);

  /// WindowBase pass-through functions ///

  /// Adds an item to the context menu
  ///
  /// The new item won't appear until the next time the context menu is displayed
  fn add_context_menu_item(&mut self, item: Box<ContextMenuItem>);
  /// Adds a separator to the context menu
  fn add_context_menu_separator(&mut self);

  fn get_uuid(&self) -> Uuid;
  fn set_uuid(&mut self, uuid: Uuid);
  fn get_main_win_uuid(&self) -> Uuid;

  fn get_pixmap(&self) -> Pixmap;

  fn get_name(&self) -> String;
  fn set_name(&mut self, name: String);

  fn get_window_type(&self) -> String;
  fn set_window_type(&mut self, window_type: String);

  fn get_event_loop(&self) -> Rc<EventLoopProxy<UserEvent>>;
  fn set_event_loop(&mut self, event_loop: Rc<EventLoopProxy<UserEvent>>);

  fn get_enabled(&self) -> bool;
  fn set_enabled(&mut self, enabled: bool);

  fn get_focused(&self) -> bool;
  fn set_focused(&mut self, focused: bool);

  fn get_location(&self) -> (f64, f64);
  fn set_location(&mut self, x: f64, y: f64);

  fn get_layout_location(&self) -> (f64, f64);
  fn set_layout_location(&mut self, x: f64, y: f64);

  fn get_width(&self) -> f64;
  fn set_width(&mut self, width: f64);
  fn get_height(&self) -> f64;
  fn set_height(&mut self, height: f64);

  fn get_min_size(&self) -> Option<(f64, f64)>;
  fn set_min_size(&mut self, width: f64, height: f64);

  fn get_max_size(&self) -> Option<(f64, f64)>;
  fn set_max_size(&mut self, width: f64, height: f64);

  fn get_drawing_size(&self) -> (f64, f64);

  fn get_x_scroll(&self) -> f64;
  fn set_x_scroll(&mut self, x_scroll: f64);
  fn get_x_scroll_min(&self) -> f64;
  fn set_x_scroll_min(&mut self, value: f64);
  fn get_x_scroll_max(&self) -> f64;
  fn set_x_scroll_max(&mut self, value: f64);

  fn get_y_scroll(&self) -> f64;
  fn set_y_scroll(&mut self, y_scroll: f64);
  fn get_y_scroll_min(&self) -> f64;
  fn set_y_scroll_min(&mut self, value: f64);
  fn get_y_scroll_max(&self) -> f64;
  fn set_y_scroll_max(&mut self, value: f64);

  /// Returns the maximum number of visible items, used when scrolling
  fn get_max_horizontal_visible_items(&self) -> f64;
  fn get_max_vertical_visible_items(&self) -> f64;

  fn get_text(&self) -> Option<String>;
  fn set_text(&mut self, text: String);

  fn get_background_color(&self) -> Color;
  fn set_background_color(&mut self, color: Color);
  /// End WindowBase pass-through functions ///

  /// Processes keyboard events when this window has focus
  fn handle_keyboard_pressed_event(&mut self, event: KeyEvent);
  fn handle_keyboard_released_event(&mut self, event: KeyEvent);

  /// Processes mouse click events when this window has focus
  ///
  /// mouse_x and mouse_y are relative to the main window
  fn handle_mouse_pressed(&mut self, _button: MouseButton,
      mouse_x: f64, mouse_y: f64);
  fn handle_mouse_released(&mut self, _button: MouseButton,
      mouse_x: f64, mouse_y: f64);

  /// Processes mouse drag event
  fn handle_mouse_drag(&mut self, main_win_x: f64, main_win_y: f64);
  fn handle_mouse_drag_start(&mut self, main_win_x: f64, main_win_y: f64);
  fn handle_mouse_drag_end(&mut self, main_win_x: f64, main_win_y: f64);

  /// Processes mouse movement events when this window has focus
  fn handle_mouse_movement(&mut self, main_win_x: f64, main_win_y: f64);

  /// Processes mouse wheel events when this window has focus
  fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta, phase: TouchPhase);

  /// Populates a ContextMenu
  fn populate_context_menu(&self, context_menu_rc: Rc<RefCell<ContextMenu>>);

  /// Updates the window's display
  ///
  /// Returns a pixmap onto which the window has been drawn
  fn redraw(&mut self, x: f64, y: f64, width: f64, height: f64, force: bool) -> Pixmap;

  fn get_parent(&self) -> Option<ChildType>;
  fn set_parent(&mut self, parent: Option<ChildType>);

  fn get_tooltip_text(&self) -> Option<String>;
  fn set_tooltip_text(&mut self, text: String);

  /// Causes the window to redraw its contents
  fn update(&mut self);
}

/// Type of child object within a layout
#[derive(Clone, Debug)]
pub enum ChildType {
  Window(Rc<RefCell<dyn ChildWindow>>),
  Layout(Rc<RefCell<dyn Layout>>),
}

/// Indicates a child's location within a BorderLayout
#[derive(Eq, Hash, PartialEq, Clone)]
pub enum BorderLocation {
  TOP,
  BOTTOM,
  LEFT,
  RIGHT,
  CENTER,
}

/// Alignment of a window within a Layout
pub enum Alignment {
  BOTTOM,
  CENTER,
  LEFT,
  RIGHT,
  TOP,
}

/// Specifies how child will fill a layout (not used by all layouts)
#[derive(Debug)]
pub enum LayoutFill {
  Single(Uuid),       // All extra space is given to the specified child
  Evenly,             // Extra space is evenly distributed amoungst all of the children
  Unused,             // Extra space is left unused (default)
}

/// Argument passed to a layout when a child is added to that layout
pub enum LayoutArgs {
  BORDER(BorderLocation),
  XY(f64, f64),
  None,
}

/// Type of Layout
#[derive(Clone, PartialEq)]
pub enum LayoutType {
  BorderLayout,
  RowLayout,
  ScrollLayout,
  TabLayout,
  XYLayout,
  Unknown,
}

/// Trait for all layouts
pub trait Layout : Debug {

  /// Adds a child window
  ///
  /// If an error occurred, the returned Result contains a string explaining why it failed.
  fn add_child(&mut self, child: Rc<RefCell<dyn ChildWindow>>,
    args: LayoutArgs) -> Result<(), String>;

  /// Adds an inner layout
  ///
  /// If an error occurred, the returned Result contains a string explaining why it failed.
  fn add_layout(&mut self, layout: Rc<RefCell<dyn Layout>>,
    args: LayoutArgs) -> Result<(), String>;

  /// Removes all items from the layout
  fn clear(&mut self);

  /// The only layout that performs any actions in this method is the TabLayout
  fn close_tab(&mut self, tab_uuid: Uuid);

  /// Gets the child that contains the specified screen pixel location
  fn get_child_at(&self, x: f64, y: f64) -> Option<Rc<RefCell<dyn ChildWindow>>>;

  /// Gets the child window with the specified ID
  fn get_child_with_id(&mut self, uuid: Uuid) -> Option<Rc<RefCell<dyn ChildWindow>>>;

  /// Gets the child layout with the specified ID
  fn get_layout_with_id(&self, uuid: Uuid) -> Option<Rc<RefCell<dyn Layout>>>;

  /// Gets the list of all descendent layouts of the specified type
  fn get_layouts_of_type(&self, layout_type: LayoutType) -> Vec<Rc<RefCell<dyn Layout>>>;

  /// Gets the type of layout
  fn get_type(&self) -> LayoutType;

  /// The only layout that performs any actions in this method is the TabLayout
  fn set_active_tab_by_uuid(&mut self, tab_uuid: Uuid);

  /// Sets the fill algorithm
  fn set_fill(&mut self, algorithm: Box<LayoutFill>);

  /// Sets the layout's maximum size
  fn set_max_size(&mut self, width: f64, height: f64);

  /// LayoutBase pass-through functions ///

  fn get_uuid(&self) -> Uuid;
  fn set_uuid(&mut self, uuid: Uuid);

  fn get_event_loop(&self) -> Rc<EventLoopProxy<UserEvent>>;
  fn set_event_loop(&mut self, event_loop: Rc<EventLoopProxy<UserEvent>>);

  fn get_main_win_uuid(&self) -> Uuid;
  fn set_main_win_uuid(&mut self, main_win_uuid: Uuid);

  fn get_pixmap(&self) -> Pixmap;
  fn set_pixmap(&mut self, pixmap: Pixmap);

  fn get_main_win_x(&self) -> f64;
  fn set_main_win_x(&mut self, main_win_x: f64);

  fn get_main_win_y(&self) -> f64;
  fn set_main_win_y(&mut self, main_win_y: f64);

  fn get_location(&self) -> (f64, f64);

  fn get_width(&self) -> f64;
  fn set_width(&mut self, width: f64);

  fn get_height(&self) -> f64;
  fn set_height(&mut self, height: f64);

  fn get_size(&self) -> (f64, f64);

  fn get_name(&self) -> String;
  fn set_name(&mut self, name: String);

  /// Lays out the child windows within the parent.
  ///
  /// main_win_x and main_win_y are the relative the main window
  /// width and height are the size of the layout's region
  ///
  /// Returns a pixmap onto which the window has been drawn
  fn layout(&mut self, main_win_x: f64, main_win_y: f64, width: f64, height: f64) -> Pixmap;

  /// Causes the layout to redraw its contents
  fn update(&mut self);
}
