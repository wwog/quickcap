use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tao::{event::WindowEvent, rwh_06, window::Window};
use wry::WebView;

use crate::app::bg_surface::{self, BgSurface};

#[allow(dead_code)]
pub struct AppWindow {
    window: Arc<Window>,
    pub display_id: usize,
    webview: Option<WebView>,
    bg_surface: BgSurface,
}

impl AppWindow {
    pub fn new(window: Window, display_id: usize) -> Self {
        let window = Arc::new(window);
        let bg_surface = pollster::block_on(BgSurface::new(window.clone()));
        Self {
            window,
            display_id,
            webview: None,
            bg_surface,
        }
    }

    pub fn set_webview(&mut self, webview: WebView) {
        self.webview = Some(webview);
    }

    // 处理窗口事件
    pub fn handle_event(&self, event: &WindowEvent) {
        match event {
           
            _ => (),
        }
    }

    pub fn render(&self) {
        self.bg_surface.render().unwrap();
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
