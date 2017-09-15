#[macro_use] extern crate allegro;
extern crate allegro_window;
extern crate event_loop;
extern crate input;
extern crate window;

use allegro_window::*;
use event_loop::EventLoop;
use input::CloseEvent;
use window::*;

allegro_main! {
    let mut window: AllegroWindow = WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut events = event_loop::Events::new(window.get_event_settings());
    while let Some(event) = events.next(&mut window) {
        if let Some(_) = event.close_args() {
            window.set_should_close(true);
        }
    }
}
