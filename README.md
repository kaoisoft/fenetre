# fenetre - GUI library in Pure Rust

```toml
[dependencies]
winit = "0.30.11"
softbuffer = "0.4.6"
tiny-skia = "0.11.4"
fontdue = "0.9.3"
image = "0.25.6"
chrono = "0.4.41"

[dependencies.uuid]
version = "1.17.0"
features = [
    "v4",                # Lets you generate random UUIDs
    "fast-rng",          # Use a faster (but still sufficiently random) RNG
    "macro-diagnostics", # Enable better diagnostics for compile-time UUIDs
]
```

fenetre is a pure Rust-based GUI library that uses winit, softbuffer, and tiny-skia to create
and manage windows.

An application using fenetre consists of windows, such an MainApp, Label, Button, LineEdit, etc.
and layouts, such as RowLayout, BorderLayout, etc. Layouts can contain nested layout, providing
a means to create complex displays.

## Cross-platform

fenetre does not contain any platform-specific code, but it has only been tested under Linux.

## Usage

Each application contains a MainApp window which contains all of the other windows and layout
that comprise the application. The MainApp window has a BorderLayout layout. An application
creates a layout for its components, sets the contents of that layout, and then sets the contents
of the MainApp by calling its set_contents() function.

See the examples sub-directory for examples applications that use fenetre. 

## Examples

### simple_window

Demonstrates how to create an application with a main window.

### editor

This example application is a simple text editor.

## Custom Windows and Layouts

To assist in creating custom windows and layouts, two template files exist, 
child_window_template.txt and layout_template.txt. These templates contain default
implementations of all of the traits that must be implemented.

## License

fenetre is licensed under Apache 2.0 (see [LICENSE](LICENSE)).
