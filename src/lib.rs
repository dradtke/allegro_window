extern crate allegro;
extern crate core;
extern crate event_loop;
extern crate input;
extern crate window;

use core::convert::From;
use input::{ButtonState, Button, Input};
use input::keyboard::Key;
use window::{AdvancedWindow, Window};

pub struct AllegroWindow {
    display: allegro::Display,
    event_queue: allegro::EventQueue,
    core: allegro::Core,

    exit_on_esc: bool,
    title: String,
    event_settings: event_loop::EventSettings,

    should_close: bool,
}

impl window::BuildFromWindowSettings for AllegroWindow {
    fn build_from_window_settings(settings: &window::WindowSettings) -> Result<AllegroWindow, String> {
        let size = settings.get_size();

        let core = allegro::Core::init()?;
        let display = allegro::Display::new(&core, size.width as i32, size.height as i32).map_err(|_| String::from("failed to create display"))?;
        let event_queue = allegro::EventQueue::new(&core).map_err(|_| String::from("failed to create event queue"))?;

        core.install_mouse().map_err(|_| "failed to install mouse")?;
        core.install_keyboard().map_err(|_| "failed to install mouse")?;

        event_queue.register_event_source(display.get_event_source());
        event_queue.register_event_source(core.get_mouse_event_source().unwrap());
        event_queue.register_event_source(core.get_keyboard_event_source().unwrap());

        display.set_window_title(&settings.get_title());

        Ok(AllegroWindow{
            core,
            display,
            event_queue,

            exit_on_esc: settings.get_exit_on_esc(),
            title: settings.get_title(),
            event_settings: event_loop::EventSettings::new(),

            should_close: false,
        })
    }
}

impl Window for AllegroWindow {
    fn set_should_close(&mut self, value: bool) {
        self.should_close = value;
    }

    fn should_close(&self) -> bool {
        self.should_close
    }

    fn size(&self) -> window::Size {
        window::Size{
            width: self.display.get_width() as u32,
            height: self.display.get_height() as u32,
        }
    }

    fn swap_buffers(&mut self) {
        // I'm not entirely sure if this is the right implementation, but it
        // seems like this would be the correct operation here.
        self.core.flip_display();
    }

    fn wait_event(&mut self) -> Input {
        let event;
        loop {
            match self.event_queue.wait_for_event() {
                allegro::Event::NoEvent => (),
                e => {
                    event = self.translate_event(e);
                    break;
                },
            }
        }
        self.handle_closings(&event);
        event
    }

    fn wait_event_timeout(&mut self, timeout: std::time::Duration) -> Option<Input> {
        match self.event_queue.wait_for_event_timed(timeout.as_secs() as f64) {
            allegro::Event::NoEvent => None,
            e => {
                let event = self.translate_event(e);
                self.handle_closings(&event);
                Some(event)
            },
        }
    }

    fn poll_event(&mut self) -> Option<Input> {
        match self.event_queue.get_next_event() {
            allegro::Event::NoEvent => None,
            e => {
                let event = self.translate_event(e);
                self.handle_closings(&event);
                Some(event)
            },
        }
    }

    fn draw_size(&self) -> window::Size {
        self.size()
    }
}

impl AdvancedWindow for AllegroWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn set_title(&mut self, value: String) {
        self.display.set_window_title(&value);
        self.title = value;
    }

    fn get_exit_on_esc(&self) -> bool {
        self.exit_on_esc
    }

    fn set_exit_on_esc(&mut self, value: bool) {
        self.exit_on_esc = value;
    }

    fn set_capture_cursor(&mut self, value: bool) {
        if value {
            self.core.grab_mouse(&self.display).unwrap();
        } else {
            self.core.ungrab_mouse().unwrap();
        }
    }

    fn show(&mut self) {
        panic!("not implemented");
    }

    fn hide(&mut self) {
        panic!("not implemented");
    }

    fn get_position(&self) -> Option<window::Position> {
        let (x, y) = self.display.get_window_position();
        Some(window::Position{x, y})
    }

    fn set_position<P: Into<window::Position>>(&mut self, value: P) {
        let position = value.into();
        self.display.set_window_position(position.x, position.y);
    }

    fn title(mut self, value: String) -> AllegroWindow {
        self.set_title(value);
        self
    }

    fn exit_on_esc(mut self, value: bool) -> AllegroWindow {
        self.set_exit_on_esc(value);
        self
    }

    fn capture_cursor(mut self, value: bool) -> AllegroWindow {
        self.set_capture_cursor(value);
        self
    }

    fn position<P: Into<window::Position>>(mut self, value: P) -> AllegroWindow {
        self.set_position(value);
        self
    }
}

impl event_loop::EventLoop for AllegroWindow {
    fn get_event_settings(&self) -> event_loop::EventSettings {
        self.event_settings
    }

    fn set_event_settings(&mut self, settings: event_loop::EventSettings) {
        self.event_settings = settings;
    }
}

impl AllegroWindow {
    fn handle_closings(&mut self, event: &Input) {
        if self.exit_on_esc {
            if let &Input::Button(input::ButtonArgs{state: ButtonState::Press, button: Button::Keyboard(Key::Escape), ..}) = event {
                self.should_close = true
            }
        }
    }

    fn translate_event(&self, event: allegro::Event) -> Input {
        use allegro::Event::*;
        match event {
            NoEvent => panic!("received no event!"),
            DisplayClose{..} => Input::Close(input::CloseArgs),
            DisplayResize{width, height, ..} => Input::Resize(width as u32, height as u32),
            JoystickAxes{..} | JoystickButtonDown{..} | JoystickButtonUp{..} | JoystickConfiguration{..} => panic!("joystick events not supported"),
            KeyDown{keycode, ..} => Input::Button(input::ButtonArgs{
                state: ButtonState::Press,
                button: Button::Keyboard(self.translate_keycode(keycode)),
                scancode: None,
            }),
            KeyUp{keycode, ..} => Input::Button(input::ButtonArgs{
                state: ButtonState::Release,
                button: Button::Keyboard(self.translate_keycode(keycode)),
                scancode: None,
            }),
            KeyChar{unichar, ..} => Input::Text(unichar.to_string()),
            MouseAxes{dx, dy, ..} => Input::Move(input::Motion::MouseRelative(dx as f64, dy as f64)),
            MouseButtonDown{button, ..} => Input::Button(input::ButtonArgs{
                state: ButtonState::Press,
                button: Button::Mouse(self.translate_mouse_button(button)),
                scancode: None,
            }),
            MouseButtonUp{button, ..} => Input::Button(input::ButtonArgs{
                state: ButtonState::Release,
                button: Button::Mouse(self.translate_mouse_button(button)),
                scancode: None,
            }),
            MouseWarped{x, y, ..} => Input::Move(input::Motion::MouseCursor(x as f64, y as f64)),
            MouseEnterDisplay{..} => Input::Cursor(true),
            MouseLeaveDisplay{..} => Input::Cursor(false),
            TimerTick{..} => panic!("timer events not supported"),
        }
    }

    fn translate_mouse_button(&self, button: u32) -> input::mouse::MouseButton {
        use input::mouse::MouseButton::*;
        if button & 1 != 0 {
            Left
        } else if button & 2 != 0 {
            Right
        } else if button & 4 != 0 {
            Middle
        } else {
            panic!("unknown mouse button: {}", button)
        }
    }

    fn translate_keycode(&self, keycode: allegro::keycodes::KeyCode) -> Key {
        use allegro::keycodes::KeyCode;
        match keycode {
            KeyCode::A => Key::A,
            KeyCode::B => Key::B,
            KeyCode::C => Key::C,
            KeyCode::D => Key::D,
            KeyCode::E => Key::E,
            KeyCode::F => Key::F,
            KeyCode::G => Key::G,
            KeyCode::H => Key::H,
            KeyCode::I => Key::I,
            KeyCode::J => Key::J,
            KeyCode::K => Key::K,
            KeyCode::L => Key::L,
            KeyCode::M => Key::M,
            KeyCode::N => Key::N,
            KeyCode::O => Key::O,
            KeyCode::P => Key::P,
            KeyCode::Q => Key::Q,
            KeyCode::R => Key::R,
            KeyCode::S => Key::S,
            KeyCode::T => Key::T,
            KeyCode::U => Key::U,
            KeyCode::V => Key::V,
            KeyCode::W => Key::W,
            KeyCode::X => Key::X,
            KeyCode::Y => Key::Y,
            KeyCode::Z => Key::Z,
            KeyCode::_0 => Key::D0,
            KeyCode::_1 => Key::D1,
            KeyCode::_2 => Key::D2,
            KeyCode::_3 => Key::D3,
            KeyCode::_4 => Key::D4,
            KeyCode::_5 => Key::D5,
            KeyCode::_6 => Key::D6,
            KeyCode::_7 => Key::D7,
            KeyCode::_8 => Key::D8,
            KeyCode::_9 => Key::D9,
            KeyCode::Pad0 => Key::NumPad0,
            KeyCode::Pad1 => Key::NumPad1,
            KeyCode::Pad2 => Key::NumPad2,
            KeyCode::Pad3 => Key::NumPad3,
            KeyCode::Pad4 => Key::NumPad4,
            KeyCode::Pad5 => Key::NumPad5,
            KeyCode::Pad6 => Key::NumPad6,
            KeyCode::Pad7 => Key::NumPad7,
            KeyCode::Pad8 => Key::NumPad8,
            KeyCode::Pad9 => Key::NumPad9,
            KeyCode::F1 => Key::F1,
            KeyCode::F2 => Key::F2,
            KeyCode::F3 => Key::F3,
            KeyCode::F4 => Key::F4,
            KeyCode::F5 => Key::F5,
            KeyCode::F6 => Key::F6,
            KeyCode::F7 => Key::F7,
            KeyCode::F8 => Key::F8,
            KeyCode::F9 => Key::F9,
            KeyCode::F10 => Key::F10,
            KeyCode::F11 => Key::F11,
            KeyCode::F12 => Key::F12,
            KeyCode::Escape => Key::Escape,
            KeyCode::Minus => Key::Minus,
            KeyCode::Equals => Key::Equals,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Tab => Key::Tab,
            KeyCode::Openbrace => Key::LeftBracket,
            KeyCode::Closebrace => Key::RightBracket,
            KeyCode::Enter => Key::Return,
            KeyCode::Semicolon => Key::Semicolon,
            KeyCode::Quote => Key::Quote,
            KeyCode::Backslash => Key::Backslash,
            KeyCode::Backslash2 => Key::Backslash,
            KeyCode::Comma => Key::Comma,
            KeyCode::Slash => Key::Slash,
            KeyCode::Space => Key::Space,
            KeyCode::Insert => Key::Insert,
            KeyCode::Delete => Key::Delete,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PgUp => Key::PageUp,
            KeyCode::PgDn => Key::PageDown,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::PadMinus => Key::NumPadMinus,
            KeyCode::PadPlus => Key::NumPadPlus,
            KeyCode::PadEnter => Key::NumPadEnter,
            KeyCode::PrintScreen => Key::PrintScreen,
            KeyCode::Pause => Key::Pause,
            KeyCode::At => Key::At,
            KeyCode::Colon2 => Key::NumPadColon,
            KeyCode::PadEquals => Key::NumPadEquals,
            KeyCode::Backquote => Key::Backquote,
            KeyCode::Semicolon2 => Key::Semicolon,
            KeyCode::Unknown => Key::Unknown,
            KeyCode::LShift => Key::LShift,
            KeyCode::RShift => Key::RShift,
            KeyCode::LCtrl => Key::LCtrl,
            KeyCode::RCtrl => Key::RCtrl,
            KeyCode::Alt => Key::LAlt,
            KeyCode::AltGr => Key::RAlt,
            KeyCode::LWin => Key::LGui,
            KeyCode::RWin => Key::RGui,
            KeyCode::Menu => Key::Menu,
            KeyCode::ScrollLock => Key::ScrollLock,
            KeyCode::NumLock => Key::NumLockClear,
            KeyCode::CapsLock => Key::CapsLock,
            k => panic!("unknown key: {}", k as u32),
        }
    }
}
