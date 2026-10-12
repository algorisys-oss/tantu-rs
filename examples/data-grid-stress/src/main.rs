//! Opens the million-row workload with the default desktop renderer and shell.
#![forbid(unsafe_code)]

use tantu::prelude::*;

fn main() -> tantu::Result<()> {
    App::new()
        .window(
            Window::new("Tantu · DataGrid stress").size(1280.0, 720.0),
            data_grid_stress::stress_view,
        )
        .run()
}
