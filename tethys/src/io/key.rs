use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Clone, Copy, Debug)]
pub enum Key {
    Char(char),
    Escape,
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
    Left,
    Right,
    Up,
    Down,
}

pub struct KeyState {
    states: [bool; 256],
    shift_lr: (bool, bool),
    ctrl_lr: (bool, bool),
    alt_lr: (bool, bool),
    up: bool,
    left: bool,
    right: bool,
    down: bool,
    escape: bool,
}

impl Key {
    pub(crate) fn from_physical(physical_key: PhysicalKey) -> Option<Key> {
        match physical_key {
            PhysicalKey::Code(code) => match code {
                KeyCode::Backquote => Some(Self::Char('`')),
                KeyCode::Backslash => Some(Self::Char('\\')),
                KeyCode::BracketLeft => Some(Self::Char('[')),
                KeyCode::BracketRight => Some(Self::Char(']')),
                KeyCode::Comma => Some(Self::Char(',')),
                KeyCode::Digit0 => Some(Self::Char('0')),
                KeyCode::Digit1 => Some(Self::Char('1')),
                KeyCode::Digit2 => Some(Self::Char('2')),
                KeyCode::Digit3 => Some(Self::Char('3')),
                KeyCode::Digit4 => Some(Self::Char('4')),
                KeyCode::Digit5 => Some(Self::Char('5')),
                KeyCode::Digit6 => Some(Self::Char('6')),
                KeyCode::Digit7 => Some(Self::Char('7')),
                KeyCode::Digit8 => Some(Self::Char('8')),
                KeyCode::Digit9 => Some(Self::Char('9')),
                KeyCode::Equal => Some(Self::Char('=')),
                KeyCode::IntlBackslash => None,
                KeyCode::IntlRo => None,
                KeyCode::IntlYen => None,
                KeyCode::KeyA => Some(Self::Char('a')),
                KeyCode::KeyB => Some(Self::Char('b')),
                KeyCode::KeyC => Some(Self::Char('c')),
                KeyCode::KeyD => Some(Self::Char('d')),
                KeyCode::KeyE => Some(Self::Char('e')),
                KeyCode::KeyF => Some(Self::Char('f')),
                KeyCode::KeyG => Some(Self::Char('g')),
                KeyCode::KeyH => Some(Self::Char('h')),
                KeyCode::KeyI => Some(Self::Char('i')),
                KeyCode::KeyJ => Some(Self::Char('j')),
                KeyCode::KeyK => Some(Self::Char('k')),
                KeyCode::KeyL => Some(Self::Char('l')),
                KeyCode::KeyM => Some(Self::Char('m')),
                KeyCode::KeyN => Some(Self::Char('n')),
                KeyCode::KeyO => Some(Self::Char('o')),
                KeyCode::KeyP => Some(Self::Char('p')),
                KeyCode::KeyQ => Some(Self::Char('q')),
                KeyCode::KeyR => Some(Self::Char('r')),
                KeyCode::KeyS => Some(Self::Char('s')),
                KeyCode::KeyT => Some(Self::Char('t')),
                KeyCode::KeyU => Some(Self::Char('u')),
                KeyCode::KeyV => Some(Self::Char('v')),
                KeyCode::KeyW => Some(Self::Char('w')),
                KeyCode::KeyX => Some(Self::Char('x')),
                KeyCode::KeyY => Some(Self::Char('y')),
                KeyCode::KeyZ => Some(Self::Char('z')),
                KeyCode::Minus => Some(Self::Char('-')),
                KeyCode::Period => Some(Self::Char('.')),
                KeyCode::Quote => Some(Self::Char('\'')),
                KeyCode::Semicolon => Some(Self::Char(';')),
                KeyCode::Slash => Some(Self::Char('/')),
                KeyCode::AltLeft => Some(Self::LAlt),
                KeyCode::AltRight => Some(Self::RAlt),
                KeyCode::Backspace => None,
                KeyCode::CapsLock => None,
                KeyCode::ContextMenu => None,
                KeyCode::ControlLeft => Some(Self::LCtrl),
                KeyCode::ControlRight => Some(Self::RCtrl),
                KeyCode::Enter => Some(Self::Char('\n')),
                KeyCode::SuperLeft => None,
                KeyCode::SuperRight => None,
                KeyCode::ShiftLeft => Some(Self::LShift),
                KeyCode::ShiftRight => Some(Self::RShift),
                KeyCode::Space => Some(Self::Char(' ')),
                KeyCode::Tab => None,
                KeyCode::Convert => None,
                KeyCode::KanaMode => None,
                KeyCode::Lang1 => None,
                KeyCode::Lang2 => None,
                KeyCode::Lang3 => None,
                KeyCode::Lang4 => None,
                KeyCode::Lang5 => None,
                KeyCode::NonConvert => None,
                KeyCode::Delete => None,
                KeyCode::End => None,
                KeyCode::Help => None,
                KeyCode::Home => None,
                KeyCode::Insert => None,
                KeyCode::PageDown => None,
                KeyCode::PageUp => None,
                KeyCode::ArrowDown => Some(Self::Down),
                KeyCode::ArrowLeft => Some(Self::Left),
                KeyCode::ArrowRight => Some(Self::Right),
                KeyCode::ArrowUp => Some(Self::Up),
                KeyCode::NumLock => None,
                KeyCode::Numpad0 => None,
                KeyCode::Numpad1 => None,
                KeyCode::Numpad2 => None,
                KeyCode::Numpad3 => None,
                KeyCode::Numpad4 => None,
                KeyCode::Numpad5 => None,
                KeyCode::Numpad6 => None,
                KeyCode::Numpad7 => None,
                KeyCode::Numpad8 => None,
                KeyCode::Numpad9 => None,
                KeyCode::NumpadAdd => None,
                KeyCode::NumpadBackspace => None,
                KeyCode::NumpadClear => None,
                KeyCode::NumpadClearEntry => None,
                KeyCode::NumpadComma => None,
                KeyCode::NumpadDecimal => None,
                KeyCode::NumpadDivide => None,
                KeyCode::NumpadEnter => None,
                KeyCode::NumpadEqual => None,
                KeyCode::NumpadHash => None,
                KeyCode::NumpadMemoryAdd => None,
                KeyCode::NumpadMemoryClear => None,
                KeyCode::NumpadMemoryRecall => None,
                KeyCode::NumpadMemoryStore => None,
                KeyCode::NumpadMemorySubtract => None,
                KeyCode::NumpadMultiply => None,
                KeyCode::NumpadParenLeft => None,
                KeyCode::NumpadParenRight => None,
                KeyCode::NumpadStar => None,
                KeyCode::NumpadSubtract => None,
                KeyCode::Escape => Some(Self::Escape),
                KeyCode::Fn => None,
                KeyCode::FnLock => None,
                KeyCode::PrintScreen => None,
                KeyCode::ScrollLock => None,
                KeyCode::Pause => None,
                KeyCode::BrowserBack => None,
                KeyCode::BrowserFavorites => None,
                KeyCode::BrowserForward => None,
                KeyCode::BrowserHome => None,
                KeyCode::BrowserRefresh => None,
                KeyCode::BrowserSearch => None,
                KeyCode::BrowserStop => None,
                KeyCode::Eject => None,
                KeyCode::LaunchApp1 => None,
                KeyCode::LaunchApp2 => None,
                KeyCode::LaunchMail => None,
                KeyCode::MediaPlayPause => None,
                KeyCode::MediaSelect => None,
                KeyCode::MediaStop => None,
                KeyCode::MediaTrackNext => None,
                KeyCode::MediaTrackPrevious => None,
                KeyCode::Power => None,
                KeyCode::Sleep => None,
                KeyCode::AudioVolumeDown => None,
                KeyCode::AudioVolumeMute => None,
                KeyCode::AudioVolumeUp => None,
                KeyCode::WakeUp => None,
                KeyCode::Meta => None,
                KeyCode::Hyper => None,
                KeyCode::Turbo => None,
                KeyCode::Abort => None,
                KeyCode::Resume => None,
                KeyCode::Suspend => None,
                KeyCode::Again => None,
                KeyCode::Copy => None,
                KeyCode::Cut => None,
                KeyCode::Find => None,
                KeyCode::Open => None,
                KeyCode::Paste => None,
                KeyCode::Props => None,
                KeyCode::Select => None,
                KeyCode::Undo => None,
                KeyCode::Hiragana => None,
                KeyCode::Katakana => None,
                KeyCode::F1 => None,
                KeyCode::F2 => None,
                KeyCode::F3 => None,
                KeyCode::F4 => None,
                KeyCode::F5 => None,
                KeyCode::F6 => None,
                KeyCode::F7 => None,
                KeyCode::F8 => None,
                KeyCode::F9 => None,
                KeyCode::F10 => None,
                KeyCode::F11 => None,
                KeyCode::F12 => None,
                KeyCode::F13 => None,
                KeyCode::F14 => None,
                KeyCode::F15 => None,
                KeyCode::F16 => None,
                KeyCode::F17 => None,
                KeyCode::F18 => None,
                KeyCode::F19 => None,
                KeyCode::F20 => None,
                KeyCode::F21 => None,
                KeyCode::F22 => None,
                KeyCode::F23 => None,
                KeyCode::F24 => None,
                KeyCode::F25 => None,
                KeyCode::F26 => None,
                KeyCode::F27 => None,
                KeyCode::F28 => None,
                KeyCode::F29 => None,
                KeyCode::F30 => None,
                KeyCode::F31 => None,
                KeyCode::F32 => None,
                KeyCode::F33 => None,
                KeyCode::F34 => None,
                KeyCode::F35 => None,
                _ => None,
            },
            PhysicalKey::Unidentified(_) => None,
        }
    }
}

impl KeyState {
    pub fn new() -> Self {
        Self {
            states: [false; 256],
            shift_lr: (false, false),
            ctrl_lr: (false, false),
            alt_lr: (false, false),
            up: false,
            left: false,
            right: false,
            down: false,
            escape: false,
        }
    }

    fn get_mut(&mut self, key: Key) -> &mut bool {
        match key {
            Key::Char(c) => &mut self.states[c as usize],
            Key::Escape => &mut self.escape,
            Key::LShift => &mut self.shift_lr.0,
            Key::RShift => &mut self.shift_lr.1,
            Key::LCtrl => &mut self.ctrl_lr.0,
            Key::RCtrl => &mut self.ctrl_lr.1,
            Key::LAlt => &mut self.alt_lr.0,
            Key::RAlt => &mut self.alt_lr.1,
            Key::Left => &mut self.left,
            Key::Right => &mut self.right,
            Key::Up => &mut self.up,
            Key::Down => &mut self.down,
        }
    }

    fn get(&self, key: Key) -> bool {
        match key {
            Key::Char(c) => self.states[c as usize],
            Key::Escape => self.escape,
            Key::LShift => self.shift_lr.0,
            Key::RShift => self.shift_lr.1,
            Key::LCtrl => self.ctrl_lr.0,
            Key::RCtrl => self.ctrl_lr.1,
            Key::LAlt => self.alt_lr.0,
            Key::RAlt => self.alt_lr.1,
            Key::Left => self.left,
            Key::Right => self.right,
            Key::Up => self.up,
            Key::Down => self.down,
        }
    }

    pub(crate) fn set_down(&mut self, key: Key) {
        *self.get_mut(key) = true;
    }

    pub(crate) fn set_up(&mut self, key: Key) {
        *self.get_mut(key) = false;
    }

    pub fn is_down(&self, key: Key) -> bool {
        self.get(key)
    }
}