use tantu::prelude::*;

fn counter() -> impl View {
    let count = signal(0);
    Padding::all(16.0).child(
        Column::new()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(8.0)
            .child(Text::new(move || format!("Count: {}", count.get())).style(TextStyle::title()))
            .child(Button::new("Increment").on_press(move || count.update(|c| *c += 1))),
    )
}

fn main() -> tantu::Result<()> {
    App::new()
        .window(Window::new("Counter").size(400.0, 300.0), counter)
        .run()
}
