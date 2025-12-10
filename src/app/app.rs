use crate::app::macos::capscreen;
use std::collections::HashMap;
use std::thread;

use super::window::AppWindow;
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
            thread::spawn(move || {
                let image = capscreen(native_id);
                if let Ok(image) = image {
                    
                } else {
                    return;
                }
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
