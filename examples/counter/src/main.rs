use tantu::prelude::*;

fn counter() -> impl View {
    SizedBox::shrink()
}

fn main() -> tantu::Result<()> {
    App::new()
        .window(Window::new("Counter").size(400.0, 300.0), counter)
        .run()
}
