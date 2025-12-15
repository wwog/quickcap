use crate::app::capscreen::enumerate::enumerate_windows;
use crate::app::capscreen::{Frame, capscreen, configure_overlay_window};
use std::{sync::Arc, time::Instant};
use tao::{
    event_loop::{EventLoop, EventLoopProxy},
    monitor::MonitorHandle,
    platform::macos::{MonitorHandleExtMacOS, WindowBuilderExtMacOS},
    window::{Window, WindowBuilder},
};
use wry::{
    WebView, WebViewBuilder,
    http::{Response, header},
};

use crate::app::AppEvent;

#[allow(dead_code)]
pub struct AppWindow {
    pub window: Arc<Window>,
    pub webview: WebView,
    pub monitor: MonitorHandle,
    pub frame: Frame,
}

impl AppWindow {
    pub fn new(monitor: MonitorHandle, event_loop: &EventLoop<AppEvent>) -> Self {
        let start_time = Instant::now();
        let monitor_id = monitor.native_id();
        let proxy = event_loop.create_proxy();
        let scale_factor = monitor.scale_factor();
        let position = monitor.position().to_logical::<f64>(scale_factor);
        let size = monitor.size().to_logical::<f64>(scale_factor);
        log::info!("create attributes: position: {:?}, size: {:?}", position, size);
        let win_builder = WindowBuilder::new();
        let window = Arc::new(
            win_builder
                .with_position(position)
                .with_inner_size(size)
                .with_decorations(false)
                .with_has_shadow(false)
                .with_resizable(false)
                .with_transparent(true)
                .build(event_loop)
                .unwrap(),
        );

        // configure_overlay_window(&window);

        let frame = capscreen(monitor_id).unwrap();

        let data_arc = Arc::clone(&frame.data);
        let frame_width = frame.width;
        let frame_height = frame.height;

        let webview = WebViewBuilder::new()
            .with_html(include_str!("demo.html"))
            .with_devtools(true)
            .with_transparent(true)
            .with_initialization_script(include_str!("preload.js"))
            .with_custom_protocol("quickcap".into(), move |_name, req| {
                let path = req.uri().to_string();
                let method = req.method();
                log::info!("path: {:?}, method: {:?}", path, method);

                if method.as_str() == "OPTIONS" {
                    return Response::builder()
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                        .header("Access-Control-Allow-Headers", "*")
                        .status(200)
                        .body(vec![])
                        .unwrap()
                        .map(Into::into);
                }

                match path.as_str() {
                    "quickcap://bg/" | "quickcap://bg" => {
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
                    "quickcap://windows/" | "quickcap://windows" => {
                        let windows = enumerate_windows(monitor_id);
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
                    _ => Response::builder()
                        .status(404)
                        .body(vec![])
                        .unwrap()
                        .map(Into::into),
                }
            })
            .with_ipc_handler(Self::ipc_handler(proxy))
            .build(&window)
            .unwrap();

        log::info!("frame time: {:?}", start_time.elapsed());

        Self {
            window,
            webview,
            monitor,
            frame,
        }
    }

    fn ipc_handler(
        proxy: EventLoopProxy<AppEvent>,
    ) -> impl Fn(wry::http::Request<String>) + 'static {
        move |req| {
            log::info!("IPC: {:?}", req);
            let body = req.body();
            if body.starts_with("str:") {
                let action = body.split(":").nth(1).unwrap();
                match action {
                    "exit" => {
                        _ = proxy.send_event(AppEvent::Exit);
                    }
                    _ => {
                        log::warn!("Unknown action: {}", action);
                    }
                }
            }
        }
    }
}
