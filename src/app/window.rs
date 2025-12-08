use tao::{event::WindowEvent, window::Window};

#[allow(dead_code)]
pub struct AppWindow {
    window: Window,
    pub display_id: usize,
}

impl AppWindow {
    pub fn new(window: Window, display_id: usize) -> Self {
        Self { window, display_id }
    }

    // 处理窗口事件
    pub fn handle_event(&self, event: &WindowEvent) {
        // 处理窗口事件
        log::debug!("Window event received: {:?}", event);
    }
}
