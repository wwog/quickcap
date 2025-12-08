use std::collections::HashMap;

use super::window::AppWindow;
use tao::event::{ElementState, Event, KeyEvent, WindowEvent};
use tao::event_loop::{ControlFlow, DeviceEventFilter, EventLoop};
use tao::keyboard::Key;
use tao::monitor::MonitorHandle;
use tao::window::{WindowBuilder, WindowId};
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
                        log::info!(
                            "App Exited Reason: {:?}",
                            format!("WindowEvent::CloseRequested for window: {:?}", window_id)
                        );
                        self.windows.clear();
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => if let Some(window) = self.windows.get(&window_id) {
                        // 在主线程中处理窗口事件
                        window.handle_event(&event);
                    },
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
            // 每个窗口都有自己的事件通道和处理线程
            let app_window = AppWindow::new(window);
            self.windows.insert(window_id, app_window);
        }
    }
}
