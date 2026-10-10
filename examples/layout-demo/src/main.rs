//! Runs the layout demo: `cargo run -p layout-demo`.

use layout_demo::layout_demo;
use tantu::prelude::*;

fn main() -> tantu::Result<()> {
    App::new()
        .window(
            Window::new("Tantu · Layout demo").size(800.0, 600.0),
            layout_demo,
        )
        .run()
}
