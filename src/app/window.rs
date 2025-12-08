use std::ops::{Deref, DerefMut};

use tao::{event::WindowEvent, rwh_06, window::Window};
use wry::WebView;

#[allow(dead_code)]
pub struct AppWindow {
    window: Window,
    pub display_id: usize,
    webview: Option<WebView>,
}

impl AppWindow {
    pub fn new(window: Window, display_id: usize) -> Self {
        Self { 
            window, 
            display_id,
            webview: None,
        }
    }

    pub fn set_webview(&mut self, webview: WebView) {
        self.webview = Some(webview);
    }

    // 处理窗口事件
    pub fn handle_event(&self, event: &WindowEvent) {
        // 处理窗口事件
        log::debug!("Window event received: {:?}", event);
    }
}

impl Deref for AppWindow {
    type Target = Window;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl DerefMut for AppWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.window
    }
}

impl rwh_06::HasWindowHandle for AppWindow {
    fn window_handle(&self) -> Result<rwh_06::WindowHandle<'_>, rwh_06::HandleError> {
        self.window.window_handle()
    }
}

impl rwh_06::HasDisplayHandle for AppWindow {
    fn display_handle(&self) -> Result<rwh_06::DisplayHandle<'_>, rwh_06::HandleError> {
        self.window.display_handle()
    }
}