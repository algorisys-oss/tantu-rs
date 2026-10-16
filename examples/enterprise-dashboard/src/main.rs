//! Opens the order dashboard with the default desktop renderer and shell.
#![forbid(unsafe_code)]

use tantu::prelude::*;

fn main() -> tantu::Result<()> {
    App::new()
        .window(
            Window::new("Tantu · Orders").size(1280.0, 800.0),
            enterprise_dashboard::dashboard,
        )
        .run()
}
