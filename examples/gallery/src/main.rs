//! Runs the widget gallery: `cargo run -p gallery`.

use gallery::gallery;
use tantu::prelude::*;

fn main() -> tantu::Result<()> {
    App::new()
        .window(Window::new("Tantu · Gallery").size(960.0, 640.0), gallery)
        .run()
}
