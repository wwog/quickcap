use std::collections::HashMap;
use std::thread;

use crate::cap::{capture_screen};
use super::window::AppWindow;
use tao::event::{ElementState, Event, KeyEvent, WindowEvent};
use tao::event_loop::{ControlFlow, DeviceEventFilter, EventLoop};
use tao::keyboard::Key;
use tao::monitor::MonitorHandle;
use tao::window::{WindowBuilder, WindowId};
use wry::WebViewBuilder;

pub struct App {
    event_loop: EventLoop<()>,
    windows: HashMap<WindowId, AppWindow>,
    monitors: Vec<MonitorHandle>,
}

impl App {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
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
        self.create_window();
        self.event_loop.run(move |event, _target, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
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
                        // 只在第一次触发时打印日志并退出
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
    fn create_window(&mut self) -> () {
        for (index, monitor) in self.monitors.iter().enumerate() {
            let title = format!("quickcap-{}", index);
            log::info!("Creating window for monitor: {:?}", title);
            let position = monitor.position();
            let size = monitor.size();


            let display_id = index;
            thread::Builder::new()
                .name(format!("capture-screen-{}", index))
                .spawn(move || {
                    match capture_screen(display_id, true) {
                        Ok(_) => {
                            log::info!("Capture screen for display {} initialized successfully", display_id);
                        }
                        Err(e) => {
                            log::error!("Capture screen for display {} initialized failed: {:?}", display_id, e);
                        }
                    }
                })
                .unwrap_or_else(|e| {
                    log::error!("Failed to spawn capture-screen thread: {}", e);
                    panic!("Failed to spawn capture-screen thread");
                });

            let window = WindowBuilder::new()
                .with_title(title)
                .with_background_color((0, 0, 0, 100))
                .with_position(position)
                .with_inner_size(size)
                .with_resizable(false)
                .with_decorations(false)
                .with_transparent(true)
                .build(&self.event_loop)
                .unwrap();
            let window_id = window.id();

            let mut app_window = AppWindow::new(window, index);
            
            let web_view = WebViewBuilder::new()
                .with_transparent(true)
                .with_background_color((0, 0, 0, 0))
                .with_html(r#"<html><body><h1 style="color: white;">Hello, World!</h1></body></html>"#)
                .build(&app_window)
                .unwrap();

            app_window.set_webview(web_view);

            self.windows.insert(window_id, app_window);
        }
    }
}
