use crate::app::capscreen::enumerate::enumerate_windows;
use crate::app::capscreen::{Frame, capscreen};
use crate::app::user_event::UserEvent;
use std::{sync::Arc, time::Instant};

use tao::{
    event_loop::{EventLoop},
    monitor::MonitorHandle,
    window::{Window, WindowBuilder},
};

#[allow(unused_imports)]
#[cfg(target_os = "windows")]
use tao::platform::windows::{MonitorHandleExtWindows, WindowBuilderExtWindows};

use wry::{
    WebView, WebViewBuilder,
    http::{Response, header},
};

static FILEDATA: &[u8] = include_bytes!("demo.html");

#[allow(dead_code)]
pub struct AppWindow {
    pub window: Arc<Window>,
    pub webview: WebView,
    pub monitor: MonitorHandle,
    pub frame: Frame,
}

impl AppWindow {
    pub fn new(monitor: MonitorHandle, event_loop: &EventLoop<UserEvent>) -> Self {
        let proxy = event_loop.create_proxy();
        let scale_factor = monitor.scale_factor();
        let position = monitor.position().to_logical::<f64>(scale_factor);
        let size = monitor.size().to_logical::<f64>(scale_factor);
        log::info!(
            "create attributes: position: {:?}, size: {:?}",
            position,
            size
        );
        let mut win_builder = WindowBuilder::new()
            .with_decorations(false)
            .with_resizable(false)
            .with_transparent(true)
            .with_position(position)
            .with_inner_size(size);

        #[cfg(target_os = "macos")]
        {
            use tao::platform::macos::WindowBuilderExtMacOS;
            win_builder = win_builder.with_has_shadow(false)
        }
        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowBuilderExtWindows;
            win_builder = win_builder.with_undecorated_shadow(false);
        }
        let window = Arc::new(win_builder.build(event_loop).unwrap());

        let start_capscreen_time = Instant::now();
        let frame = capscreen(&monitor).unwrap();
        log::info!("capscreen time: {:?}", start_capscreen_time.elapsed());

        let data_arc = Arc::clone(&frame.data);
        let frame_width = frame.width;
        let frame_height = frame.height;
        let monitor_for_enum = monitor.clone();

        let webview = WebViewBuilder::new()
            .with_transparent(true)
            .with_initialization_script(include_str!("preload.js"))
            .with_accept_first_mouse(true)
            .with_ipc_handler(move |req|{
                let body = req.body();
                log::info!("ipc body: {:?}", body);
                match body.as_str() {
                    "exit" => {
                        proxy.send_event(UserEvent::Exit).unwrap_or_else(|e| {
                            log::error!("send event failed: {:?}", e);
                        });
                    }
                    _ => {}
                }
            })
            .with_custom_protocol("app".into(), move |_name, req| {
                let path = req.uri().path().to_string();
                log::info!("path: {:?}", path);
                match path.as_str() {
                    "/bg" => {
                        let data = Arc::try_unwrap(Arc::clone(&data_arc))
                            .unwrap_or_else(|arc| (*arc).clone());
                        Response::builder()
                            .header(header::CONTENT_TYPE, "application/octet-stream")
                            .header("Access-Control-Allow-Origin", "*")
                            .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                            .header("Access-Control-Allow-Headers", "*")
                            .header(
                                "Access-Control-Expose-Headers",
                                "x-frame-width, x-frame-height",
                            )
                            .header("x-frame-width", frame_width.to_string())
                            .header("x-frame-height", frame_height.to_string())
                            .status(200)
                            .body(data)
                            .unwrap()
                            .map(Into::into)
                    }
                    "/windows" => {
                        let windows = enumerate_windows(&monitor_for_enum);
                        let json =
                            serde_json::to_string(&windows).unwrap_or_else(|_| "[]".to_string());
                        Response::builder()
                            .header(header::CONTENT_TYPE, "application/json")
                            .header("Access-Control-Allow-Origin", "*")
                            .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                            .header("Access-Control-Allow-Headers", "*")
                            .status(200)
                            .body(json.into_bytes())
                            .unwrap()
                            .map(Into::into)
                    }
                    "/" => Response::builder()
                        .header(header::CONTENT_TYPE, "text/html")
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                        .header("Access-Control-Allow-Headers", "*")
                        .status(200)
                        .body(FILEDATA.to_vec())
                        .unwrap()
                        .map(Into::into),
                    _ => Response::builder()
                        .status(404)
                        .body(vec![])
                        .unwrap()
                        .map(Into::into),
                }
            })
            .with_transparent(true)
            .with_url("app://localhost")
            .build(&window)
            .unwrap();

        Self {
            window,
            webview,
            monitor,
            frame,
        }
    }
}
