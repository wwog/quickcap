use crate::app::macos::capscreen;
use std::collections::HashMap;
use base64::Engine;

use super::window::AppWindow;
use tao::event::{ElementState, Event, KeyEvent, WindowEvent};
use tao::event_loop::{
    ControlFlow, DeviceEventFilter, EventLoop, EventLoopBuilder, EventLoopProxy,
};
use tao::keyboard::Key;
use tao::monitor::MonitorHandle;
use tao::platform::macos::MonitorHandleExtMacOS;
use tao::window::{WindowBuilder, WindowId};
use wry::WebViewBuilder;

/// 自定义应用事件
#[derive(Debug)]
pub enum AppEvent {
    /// 请求退出应用
    Exit,
    /// 区域数据提取请求
    ExtractRegion {
        window_id: WindowId,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    },
}

pub struct App {
    event_loop: EventLoop<AppEvent>,
    windows: HashMap<WindowId, AppWindow>,
    monitors: Vec<MonitorHandle>,
}

impl App {
    pub fn new() -> Self {
        let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
        let monitors: Vec<MonitorHandle> = event_loop.available_monitors().collect();
        log::info!("Found {} monitors", monitors.len());
        for monitor in &monitors {
            log::info!(
                "{}, {:?}, {:?}",
                monitor.name().unwrap_or("Unknown".to_string()),
                monitor.position(),
                monitor.size()
            );
        }
        let windows = HashMap::with_capacity(monitors.len());
        event_loop.set_device_event_filter(DeviceEventFilter::Never);

        Self {
            event_loop,
            windows,
            monitors,
        }
    }
}

impl App {
    pub fn run(mut self) -> ! {
        let proxy = self.event_loop.create_proxy();
        self.create_window(proxy);
        self.event_loop.run(move |event, _target, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(event_type) => match event_type {
                    tao::event::StartCause::Init => {
                        log::info!("Event::NewEvents: Init");
                    }
                    _ => (),
                },
                Event::UserEvent(user_event) => match user_event {
                    AppEvent::Exit => {
                        if !self.windows.is_empty() {
                            log::info!("App Exit: user event");
                            self.windows.clear();
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    AppEvent::ExtractRegion {
                        window_id,
                        x,
                        y,
                        w,
                        h,
                    } => {
                        if let Some(app_window) = self.windows.get(&window_id) {
                            if let Some(region_data) = app_window.extract_region(x, y, w, h) {
                                // 将数据编码为 base64
                                let base64_data = base64::engine::general_purpose::STANDARD
                                    .encode(&region_data);
                                
                                // 通过 webview 的 evaluate_script 返回数据
                                if let Some(webview) = app_window.get_webview() {
                                    let script = format!(
                                        r#"
                                        if (window.onRegionData) {{
                                            window.onRegionData({{
                                                x: {},
                                                y: {},
                                                w: {},
                                                h: {},
                                                data: "{}"
                                            }});
                                        }}
                                        "#,
                                        x, y, w, h, base64_data
                                    );
                                    
                                    if let Err(e) = webview.evaluate_script(&script) {
                                        log::error!("Failed to evaluate script: {:?}", e);
                                    }
                                }
                            } else {
                                log::warn!("Failed to extract region data");
                                // 返回错误
                                if let Some(webview) = app_window.get_webview() {
                                    let script = r#"
                                        if (window.onRegionData) {
                                            window.onRegionData({
                                                error: "Failed to extract region"
                                            });
                                        }
                                    "#;
                                    if let Err(e) = webview.evaluate_script(script) {
                                        log::error!("Failed to evaluate script: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                },
                Event::WindowEvent {
                    window_id, event, ..
                } => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::Destroyed
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Escape,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        if !self.windows.is_empty() {
                            log::info!("App Exit: window {:?}", window_id);
                            self.windows.clear();
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    _ => {
                        if let Some(window) = self.windows.get(&window_id) {
                            // 如果窗口有耗时事件存在，则考虑异步或者多线程
                            window.handle_event(&event);
                        }
                    }
                },
                _ => (),
            }
        })
    }
}

// Private methods
impl App {
    /// 创建 WebView IPC 处理器
    fn create_ipc_handler(
        proxy: EventLoopProxy<AppEvent>,
        window_id: WindowId,
    ) -> impl Fn(wry::http::Request<String>) + 'static {
        move |request: wry::http::Request<String>| {
            let body = request.body();
            log::info!("Received IPC message: {}", body);

            // 尝试解析 JSON 格式的区域请求
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body) {
                // 检查是否是区域提取请求
                if let Some(arr) = json_value.as_array() {
                    if arr.len() == 4 {
                        if let (Some(x), Some(y), Some(w), Some(h)) = (
                            arr[0].as_u64(),
                            arr[1].as_u64(),
                            arr[2].as_u64(),
                            arr[3].as_u64(),
                        ) {
                            let x = x as u32;
                            let y = y as u32;
                            let w = w as u32;
                            let h = h as u32;
                            
                            log::info!("Extracting region: x={}, y={}, w={}, h={}", x, y, w, h);
                            
                            // 通过 EventLoopProxy 发送区域提取请求到主事件循环
                            if let Err(e) = proxy.send_event(AppEvent::ExtractRegion {
                                window_id,
                                x,
                                y,
                                w,
                                h,
                            }) {
                                log::error!("Failed to send extract region event: {:?}", e);
                            }
                            return;
                        }
                    }
                }
            }

            // 处理来自 WebView 的消息
            match body.as_str() {
                "escape_pressed" => {
                    log::info!("Escape key pressed in WebView, sending exit event");
                    // 通过 EventLoopProxy 发送退出事件到主事件循环
                    if let Err(e) = proxy.send_event(AppEvent::Exit) {
                        log::error!("Failed to send exit event: {:?}", e);
                    }
                }
                _ => {
                    log::debug!("Unknown IPC message: {}", body);
                }
            }
        }
    }

    /// 获取 WebView HTML 内容
    fn get_webview_html() -> &'static str {
        r#"
        <html>
        <head>
            <style>
                body {
                    margin: 0;
                    padding: 0;
                    background: transparent;
                }
                h1 {
                    color: white;
                    font-family: system-ui, -apple-system, sans-serif;
                }
            </style>
        </head>
        <body>
            <h1>Hello, World!</h1>
            <button id="capture-button">Capture</button>
            <script>
                // 监听键盘事件
                document.addEventListener('keydown', function(e) {
                    console.log('Key pressed:', e.key);
                    if (e.key === 'Escape') {
                        window.ipc.postMessage('escape_pressed');
                    }
                });

                // 区域数据回调函数
                window.onRegionData = function(result) {
                    console.log('Region data received:', result);
                    if (result.error) {
                        console.error('Error:', result.error);
                    } else {
                        console.log('Region extracted:', result.x, result.y, result.w, result.h);
                        console.log('Data length:', result.data.length);
                        // 这里可以处理返回的区域数据
                        // result.data 是 base64 编码的 RGBA 数据
                    }
                };

                document.getElementById('capture-button').addEventListener('click', function() {
                    window.ipc.postMessage(JSON.stringify([100, 100, 200, 200]));
                });

            </script>
        </body>
        </html>
        "#
    }

    fn create_window(&mut self, proxy: EventLoopProxy<AppEvent>) -> () {
        for (index, monitor) in self.monitors.iter().enumerate() {
            let title = format!("quickcap-{}", index);
            let native_id = monitor.native_id();
            log::info!(
                "Creating window for monitor: {:?}, native_id: {:?}",
                title,
                native_id
            );
            let position = monitor.position();
            let size = monitor.size();

            let window = WindowBuilder::new()
                .with_title(title)
                .with_position(position)
                .with_inner_size(size)
                .with_resizable(false)
                .with_decorations(false)
                .with_transparent(true)
                .build(&self.event_loop)
                .unwrap();
            let window_id = window.id();

            let mut app_window = AppWindow::new(window, index, native_id);
            let image = capscreen(native_id);
            if let Ok(image) = image {
                log::info!("Captured image: {:?}", image);
                let image_data = image.rgba_data().unwrap();
                let image_data = image_data.as_slice();
                let image_width = image.width() as u32;
                let image_height = image.height() as u32;
                
                // 保存截屏数据到 AppWindow
                app_window.set_screenshot_data(image_data.to_vec(), image_width, image_height);
                
                app_window.render(image_data, image_width, image_height, image_width * 4);
            }
            
            // 从 AppWindow 中获取 window 引用来创建 webview
            let window_ref = app_window.get_window().clone();
            
            // 创建 IPC 处理器（传入 window_id）
            let ipc_handler = App::create_ipc_handler(proxy.clone(), window_id);
            
            let webview = WebViewBuilder::new()
                .with_html(App::get_webview_html())
                .with_transparent(true)
                .with_ipc_handler(ipc_handler)
                .build(&window_ref)
                .unwrap();
            
            app_window.set_webview(webview);
            self.windows.insert(window_id, app_window);
        }
    }
}
