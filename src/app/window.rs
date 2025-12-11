use std::sync::{Arc, Mutex};

use tao::{event::WindowEvent, rwh_06, window::Window};
use wry::WebView;

use crate::app::bg_surface::BgSurface;

#[allow(dead_code)]
pub struct AppWindow {
    window: Arc<Window>,
    pub display_id: usize,
    pub display_native_id: u32,
    webview: Option<WebView>,
    bg_surface: BgSurface,
    // 存储截屏数据：RGBA 字节数组、宽度、高度
    screenshot_data: Arc<Mutex<Option<ScreenshotData>>>,
}

#[derive(Clone)]
pub struct ScreenshotData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl AppWindow {
    pub fn new(window: Window, display_id: usize, display_native_id: u32) -> Self {
        let window = Arc::new(window);
        let bg_surface = pollster::block_on(BgSurface::new(window.clone()));
        Self {
            window,
            display_id,
            display_native_id,
            webview: None,
            bg_surface,
            screenshot_data: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_webview(&mut self, webview: WebView) {
        self.webview = Some(webview);
    }

    // 设置截屏数据
    pub fn set_screenshot_data(&self, data: Vec<u8>, width: u32, height: u32) {
        let screenshot = ScreenshotData { data, width, height };
        *self.screenshot_data.lock().unwrap() = Some(screenshot);
    }

    // 提取区域数据 [x, y, w, h]
    pub fn extract_region(&self, x: u32, y: u32, w: u32, h: u32) -> Option<Vec<u8>> {
        let screenshot_guard = self.screenshot_data.lock().unwrap();
        let screenshot = screenshot_guard.as_ref()?;
        
        // 边界检查
        if x + w > screenshot.width || y + h > screenshot.height {
            log::warn!(
                "Region out of bounds: ({}, {}) size ({}, {}) vs image size ({}, {})",
                x, y, w, h, screenshot.width, screenshot.height
            );
            return None;
        }

        let bytes_per_pixel = 4; // RGBA
        let source_stride = screenshot.width * bytes_per_pixel;
        
        let mut region_data = Vec::with_capacity((w * h * bytes_per_pixel) as usize);
        
        // 从源图像中提取指定区域
        for row in 0..h {
            let source_start = ((y + row) * source_stride + x * bytes_per_pixel) as usize;
            let source_end = source_start + (w * bytes_per_pixel) as usize;
            if source_end <= screenshot.data.len() {
                region_data.extend_from_slice(&screenshot.data[source_start..source_end]);
            }
        }
        
        Some(region_data)
    }

    // 处理窗口事件
    pub fn handle_event(&self, event: &WindowEvent) {
        match event {
           
            _ => (),
        }
    }

    pub fn render(
        &self,
        frame_data: &[u8],
        frame_width: u32,
        frame_height: u32,
        bytes_per_row: u32,
    ) {
        self.bg_surface
            .render(frame_data, frame_width, frame_height, bytes_per_row)
            .unwrap();
    }

    // 获取 webview 的引用（用于 IPC 响应）
    pub fn get_webview(&self) -> Option<&WebView> {
        self.webview.as_ref()
    }

    // 获取 window 的引用（用于创建 webview）
    pub fn get_window(&self) -> &Arc<Window> {
        &self.window
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
