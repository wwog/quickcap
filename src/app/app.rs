use std::collections::HashMap;

use env_logger::fmt::style::{AnsiColor, Color, Style};
use std::io::Write;
use tao::{
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    window::WindowId,
};

use crate::{
    AppConfig, StdRpcClient,
    app::{user_event::UserEvent, window::AppWindow},
};
use std::time::Instant;
pub struct App {
    windows: HashMap<WindowId, AppWindow>,
    event_loop: EventLoop<UserEvent>,
    config: AppConfig,
}

impl App {
    /// 将标准错误接口的输出用作输出，标准输出的接口的输出用作STDIO
    pub fn new(config: Option<AppConfig>) -> Self {
        let config = config.unwrap_or_default();
        let mut logger_builder = env_logger::builder();
        println!("config: {:?}", config);
        logger_builder.format(|buf, record| {
            let style_gray = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));
            let style_cyan = Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::Cyan)))
                .bold();
            let style_green = Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::Green)))
                .bold();
            let style_white = Style::new().fg_color(Some(Color::Ansi(AnsiColor::White)));

            writeln!(
                buf,
                "{}{}{} {}{}{} {}{}{} {}{}{}",
                style_gray.render(),
                buf.timestamp_millis(),
                style_gray.render_reset(),
                style_green.render(),
                "[INFO]",
                style_green.render_reset(),
                style_cyan.render(),
                record.target(),
                style_cyan.render_reset(),
                style_white.render(),
                record.args(),
                style_white.render_reset(),
            )
        });
        logger_builder.init();
        log::error!("App::new");
        let start_time = Instant::now();
        let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
        let proxy = event_loop.create_proxy();

        StdRpcClient::init(
            move |req| {
                log::error!("RpcMessage: {:?}", req);
                if let Err(e) = proxy.send_event(UserEvent::RpcMessage(req)) {
                    log::error!("Failed to send event to GUI loop: {}", e);
                }
                Ok(serde_json::Value::Null)
            },
            move |notif| {
                log::error!("RpcNotification: {:?}", notif);
            },
        );

        // Windows和Macos的逻辑并不一致，Windows是用虚拟桌面
        #[cfg(target_os = "macos")]
        let windows = {
            let monitors = event_loop.available_monitors().collect::<Vec<_>>();
            let windows = monitors
                .into_iter()
                .map(|monitor| {
                    log::error!("Monitor: {:?}", monitor);
                    AppWindow::new(monitor, &event_loop, &config)
                })
                .map(|window| (window.window.id(), window))
                .collect();
            windows
        };
        #[cfg(target_os = "windows")]
        let windows = {
            let monitor = event_loop.primary_monitor().unwrap();
            // 保留一个窗口，在windows中monitor并不是必要参数，但macos先开发，所以保留一个传参
            // 后续优化点: 添加AppWindowBuilder，根据不同的操作系统创建不同的AppWindow
            let window = AppWindow::new(monitor, &event_loop, &config);
            HashMap::from([(window.window.id(), window)])
        };
        log::error!("windows time: {:?}", start_time.elapsed());

        Self {
            windows,
            event_loop,
            config,
        }
    }

    /// 运行应用，创建并运行事件循环
    /// 此方法永远不会返回，因为事件循环会一直阻塞运行
    pub fn run(mut self) -> ! {
        log::error!("App::run");
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = tao::event_loop::ControlFlow::Wait;
            match event {
                Event::NewEvents(tao::event::StartCause::Init) => {}
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                }
                | Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key: tao::keyboard::Key::Escape,
                                    ..
                                },
                            ..
                        },
                    ..
                }
                | Event::UserEvent(UserEvent::Exit) => {
                    if !self.windows.is_empty() {
                        log::error!("WindowEvent::CloseRequested");
                        self.windows.clear();
                        *control_flow = tao::event_loop::ControlFlow::Exit;
                    }
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::CursorEntered { .. },
                    ..
                } => {
                    if let Some(window) = self.windows.get_mut(&window_id) {
                        window.window.set_focus();
                    }
                }
                Event::UserEvent(UserEvent::RpcMessage(req)) => {
                    log::error!("RpcMessage: {:?}", req);
                }
                _ => {}
            }
        })
    }
}
