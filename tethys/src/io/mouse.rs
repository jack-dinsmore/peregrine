use winit::event::MouseButton;

pub struct Mouse {
    left: bool,
    right: bool,
    middle: bool,
}

impl Mouse {
    pub(crate) fn new() -> Self {
        Self {
            left: false,
            right: false,
            middle: false,
        }
    }

    pub(crate) fn update(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.left = true,
            MouseButton::Right => self.right = true,
            MouseButton::Middle => self.middle = true,
            MouseButton::Back => (),
            MouseButton::Forward => (),
            MouseButton::Other(_) => (),
        }
    }
}