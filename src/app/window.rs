use tao::window::Window;

pub struct AppWindow {
    window: Window,
}

impl AppWindow {
    pub fn new(window: Window) -> Self {
        Self { window }
    }
}