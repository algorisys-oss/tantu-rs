//! Entry point for `cargo xtask`. See the library docs for the commands.

#![forbid(unsafe_code)]

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The workspace root is the parent of this package's directory.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the xtask package always lives inside the workspace root");
    let status = xtask::run(&args, root, &mut std::io::stdout(), &mut std::io::stderr());
    ExitCode::from(u8::try_from(status).unwrap_or(1))
}
