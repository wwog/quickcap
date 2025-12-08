use std::collections::HashMap;

use super::window::AppWindow;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, DeviceEventFilter, EventLoop};
use tao::window::{WindowBuilder, WindowId};

pub struct App {
    event_loop: EventLoop<()>,
    windows: HashMap<WindowId, AppWindow>,
}

impl App {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        event_loop.set_device_event_filter(DeviceEventFilter::Never);
        let window_builder = WindowBuilder::new();
        let window = window_builder.build(&event_loop).unwrap();
        Self { event_loop, windows: HashMap::new() }
    }
}

impl App {
    pub fn run(self) {
        self.event_loop.run(move |event, target, control_flow| {
            *control_flow = ControlFlow::Wait;

            println!("{:?}", event);

            match event {
                // Event::NewEvents(start_cause) => todo!(),
                // Event::WindowEvent { window_id, event } => todo!(),
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
}
