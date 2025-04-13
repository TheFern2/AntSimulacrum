extern crate sfml;

use sfml::graphics::*;
use sfml::system::*;
use sfml::window::*;

fn main() {
    let mut window = RenderWindow::new(
        (800, 600),
        "SFML VertexArray accessors Example",
        Style::CLOSE,
        &Default::default(),
    )
    .unwrap();
    window.set_vertical_sync_enabled(true);

    let mut rectangle = RectangleShape::new();
    rectangle.set_size(Vector2f::new(100.0, 100.0));
    rectangle.set_fill_color(Color::rgb(255, 0, 100));
    rectangle.set_position(Vector2f::new(100.0, 100.0));

    loop {
        // events
        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => {
                    window.close();
                    return;
                }
                _ => {}
            }
        }

        // drawing
        window.clear(Color::BLACK);
        window.draw(&rectangle);
        window.display();
    }
}