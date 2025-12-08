use std::collections::HashMap;

use super::window::AppWindow;
use tao::dpi::PhysicalPosition;
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
        event_loop.set_device_event_filter(DeviceEventFilter::Never);

        Self {
            event_loop,
            windows: HashMap::new(),
            monitors,
        }
    }
}

impl App {
    pub fn run(mut self) {
        self.create_window();
        self.event_loop.run(move |event, _target, control_flow| {
            *control_flow = ControlFlow::Wait;

            // println!("{:?}", event);

            match event {
                // Event::NewEvents(start_cause) => todo!(),
                Event::WindowEvent {
                    window_id, event, ..
                } => match event {
                    WindowEvent::CloseRequested => {
                        self.windows.remove(&window_id);
                        log::info!("Window closed: {:?}", window_id);
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        log::info!("Keyboard input: {:?}", logical_key);
                        if logical_key == Key::Escape {
                            self.windows.remove(&window_id);
                            log::info!("Window closed: {:?}", window_id);
                        }
                    }
                    _ => (),
                },
                // Event::DeviceEvent { device_id, event } => todo!(),
                // Event::UserEvent(_) => todo!(),
                // Event::Suspended => todo!(),
                // Event::Resumed => todo!(),
                // Event::MainEventsCleared => todo!(),
                // Event::RedrawRequested(window_id) => todo!(),
                // Event::RedrawEventsCleared => todo!(),
                // Event::LoopDestroyed => todo!(),
                // Event::Opened { urls } => todo!(),
                // Event::Reopen { has_visible_windows } => todo!(),
                _ => (),
            }
        })
    }

    pub fn create_window(&mut self) -> () {
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
            let app_window = AppWindow::new(window);
            self.windows.insert(window_id, app_window);
        }
    }
}
