use std::collections::HashMap;

use tao::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    window::WindowId,
};

use crate::app::{user_event::UserEvent, window::AppWindow};
use std::time::Instant;

pub struct App {
    windows: HashMap<WindowId, AppWindow>,
    event_loop: EventLoop<UserEvent>,
}

impl App {
    pub fn new() -> Self {
        log::info!("App::new");
        let start_time = Instant::now();
        let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
        // let monitors = event_loop.available_monitors().collect::<Vec<_>>();
        // let windows = monitors
        //     .into_iter()
        //     .map(|monitor| {
        //         log::info!("Monitor: {:?}", monitor);
        //         AppWindow::new(monitor, &event_loop)
        //     })
        //     .map(|window| (window.window.id(), window))
        //     .collect();
        let monitor = event_loop.primary_monitor().unwrap();
        let window = AppWindow::new(monitor, &event_loop);
        let windows = HashMap::from([(window.window.id(), window)]);
        log::info!("windows time: {:?}", start_time.elapsed());

        Self {
            windows,
            event_loop,
        }
    }

    /// 运行应用，创建并运行事件循环
    /// 此方法永远不会返回，因为事件循环会一直阻塞运行
    pub fn run(mut self) -> ! {
        log::info!("App::run");
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = tao::event_loop::ControlFlow::Wait;
            match event {
                Event::NewEvents(tao::event::StartCause::Init) => {
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                }
                | Event::UserEvent(UserEvent::Exit) => {
                    if !self.windows.is_empty() {
                        log::info!("WindowEvent::CloseRequested");
                        self.windows.clear();
                        *control_flow = tao::event_loop::ControlFlow::Exit;
                    }
                }
                _ => {}
            }
        })
    }
}
