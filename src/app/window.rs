use super::super::cap::capture_screen;
use std::{io::Error, thread::{self, JoinHandle}};

use tao::{event::WindowEvent, window::Window};

#[allow(dead_code)]
pub struct AppWindow {
    window: Window,
    display_id: usize,
    handle: Result<JoinHandle<()>, Error>,
}

impl AppWindow {
    pub fn new(window: Window, display_id: usize) -> Self {
        let handle = thread::Builder::new()
            .name(format!("capture-screen-{}", display_id))
            .spawn(move || {
                capture_screen(display_id, true).unwrap();
            });

        Self {
            window,
            display_id,
            handle,
        }
    }

    // 处理窗口事件
    pub fn handle_event(&self, event: &WindowEvent) {
        // 处理窗口事件
        log::debug!("Window event received: {:?}", event);
        // 在这里添加具体的事件处理逻辑
    }
}
