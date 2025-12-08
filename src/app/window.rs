use std::thread::spawn;

use tao::{event::WindowEvent, window::Window};

#[allow(dead_code)]
pub struct AppWindow {
    window: Window,
}

impl AppWindow {
    pub fn new(window: Window) -> Self {
        spawn(|| {
            
        });
        Self { window }
    }

    // 处理窗口事件
    pub fn handle_event(&self, event: &WindowEvent) {
        // 处理窗口事件
        log::debug!("Window event received: {:?}", event);
        // 在这里添加具体的事件处理逻辑
    }
}
