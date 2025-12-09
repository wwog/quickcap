use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use super::window::AppWindow;
use crate::cap::{capture_screen, result::CaptureResult};
use tao::event::{ElementState, Event, KeyEvent, WindowEvent};
use tao::event_loop::{
    ControlFlow, DeviceEventFilter, EventLoop, EventLoopBuilder, EventLoopProxy,
};
use tao::keyboard::Key;
use tao::monitor::MonitorHandle;
use tao::platform::macos::MonitorHandleExtMacOS;
use tao::window::{WindowBuilder, WindowId};

/// 自定义应用事件
#[derive(Debug)]
pub enum AppEvent {
    /// 请求退出应用
    Exit,
    /// 截屏完成事件
    ScreenCaptured {
        display_native_id: u32,
        capture: Arc<CaptureResult>,
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
                    AppEvent::ScreenCaptured {
                        display_native_id,
                        capture,
                    } => {
                        // 查找对应 display_id 的窗口
                        if let Some(window) = self
                            .windows
                            .values()
                            .find(|w| w.display_native_id == display_native_id)
                        {
                            // 锁定像素数据并渲染
                            match capture.lock_for_read() {
                                Ok(guard) => {
                                    let data = guard.as_slice();
                                    let width = capture.width as u32;
                                    let height = capture.height as u32;
                                    let bytes_per_row = capture.bytes_per_row() as u32;

                                    window.render(data, width, height, bytes_per_row);
                                    log::info!(
                                        "Rendered screenshot for display {} ({}x{})",
                                        display_native_id,
                                        width,
                                        height
                                    );
                                }
                                Err(e) => {
                                    log::error!("Failed to lock capture data: {:?}", e);
                                }
                            }
                        }
                    }
                },
                Event::RedrawRequested(window_id) => {
                    log::info!("Event::RedrawRequested: {:?}", window_id);
                    // 不再渲染占位帧，等待真实截屏数据
                }
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
    ) -> impl Fn(wry::http::Request<String>) + 'static {
        move |request: wry::http::Request<String>| {
            let body = request.body();
            log::info!("Received IPC message: {}", body);

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
            <script>
                // 监听键盘事件
                document.addEventListener('keydown', function(e) {
                    console.log('Key pressed:', e.key);
                    if (e.key === 'Escape') {
                        window.ipc.postMessage('escape_pressed');
                    }
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

            let proxy_for_capture = proxy.clone();
            thread::Builder::new()
                .name(format!("capture-screen-{}", index))
                .spawn(move || match capture_screen(native_id, false) {
                    Ok(capture) => {
                        log::info!(
                            "Capture screen for display {} initialized successfully ({}x{})",
                            native_id,
                            capture.width,
                            capture.height
                        );
                        // 将截屏数据发送到主事件循环
                        if let Err(e) = proxy_for_capture.send_event(AppEvent::ScreenCaptured {
                            display_native_id: native_id,
                            capture: Arc::new(capture),
                        }) {
                            log::error!("Failed to send screen capture event: {:?}", e);
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Capture screen for display {} initialized failed: {:?}",
                            native_id,
                            e
                        );
                    }
                })
                .unwrap_or_else(|e| {
                    log::error!("Failed to spawn capture-screen thread: {}", e);
                    panic!("Failed to spawn capture-screen thread");
                });

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

            let app_window = AppWindow::new(window, index, native_id);
            self.windows.insert(window_id, app_window);
        }
    }
}
