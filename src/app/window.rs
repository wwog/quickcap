use crate::capscreen::capscreen;
use crate::capscreen::enumerate::enumerate_windows;
use crate::app::user_event::UserEvent;
use std::{
    sync::{Arc, Condvar, Mutex},
    time::Instant,
};

use tao::{
    event_loop::EventLoop,
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
}

struct CaptureState {
    frame: Option<crate::capscreen::Frame>,
    error: Option<String>,
    done: bool,
}

impl AppWindow {
    pub fn new(monitor: MonitorHandle, event_loop: &EventLoop<UserEvent>) -> Self {
        let proxy = event_loop.create_proxy();
        #[cfg(target_os = "macos")]
        let (position, size) = {
            let scale_factor = monitor.scale_factor();
            let position = monitor.position().to_logical::<f64>(scale_factor);
            let size = monitor.size().to_logical::<f64>(scale_factor);
            log::info!(
                "create attributes: position: {:?}, size: {:?}",
                position,
                size
            );
            (position, size)
        };
        #[cfg(target_os = "windows")]
        let (position, size) = unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{
                GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
                SM_YVIRTUALSCREEN,
            };

            let cx_virtual_screen = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let cy_virtual_screen = GetSystemMetrics(SM_CYVIRTUALSCREEN);
            let x_virtual_screen = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let y_virtual_screen = GetSystemMetrics(SM_YVIRTUALSCREEN);
            // 使用虚拟桌面原点和整体尺寸，保证跨屏时位置正确
            let position =  tao::dpi::LogicalPosition::new(x_virtual_screen as f64, y_virtual_screen as f64);
            let size =  tao::dpi::LogicalSize::new(cx_virtual_screen as f64, cy_virtual_screen as f64);
            log::info!(
                "create attributes: position={:?}, size={:?}",
                position,
                size
            );
            (position, size)
        };
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

        let capture_state: Arc<(Mutex<CaptureState>, Condvar)> = Arc::new((
            Mutex::new(CaptureState {
                frame: None,
                error: None,
                done: false,
            }),
            Condvar::new(),
        ));

        // 启动后台截图线程
        let capture_state_for_thread = Arc::clone(&capture_state);
        let monitor_for_capture = monitor.clone();
        std::thread::spawn(move || {
            let start_capscreen_time = Instant::now();
            let result = capscreen(&monitor_for_capture);
            log::info!("capscreen time: {:?}", start_capscreen_time.elapsed());

            let (lock, cvar) = &*capture_state_for_thread;
            let mut state = lock.lock().unwrap();
            match result {
                Ok(frame) => {
                    state.frame = Some(frame);
                }
                Err(e) => {
                    log::error!("capscreen failed: {:?}", e);
                    state.error = Some(format!("capscreen failed: {:?}", e));
                }
            }
            state.done = true;
            cvar.notify_all();
        });

        let capture_state_for_bg = Arc::clone(&capture_state);
        let monitor_for_enum = monitor.clone();

        let webview = WebViewBuilder::new()
            .with_transparent(true)
            .with_initialization_script(include_str!("preload.js"))
            .with_accept_first_mouse(true)
            .with_ipc_handler(move |req| {
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
                        let (lock, cvar) = &*capture_state_for_bg;
                        let mut state = lock.lock().unwrap();

                        // 如果截图尚未完成，则等待
                        while !state.done {
                            state = cvar.wait(state).unwrap();
                        }

                        // 检查是否有错误
                        if let Some(error) = &state.error {
                            log::error!("Capture error: {}", error);
                            return Response::builder()
                                .status(500)
                                .header(header::CONTENT_TYPE, "text/plain")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(error.clone().into_bytes())
                                .unwrap()
                                .map(Into::into);
                        }

                        // 获取截图数据
                        if let Some(frame) = &state.frame {
                            let data = frame.data.clone();
                            Response::builder()
                                .header(header::CONTENT_TYPE, "application/octet-stream")
                                .header("Access-Control-Allow-Origin", "*")
                                .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                                .header("Access-Control-Allow-Headers", "*")
                                .header(
                                    "Access-Control-Expose-Headers",
                                    "x-frame-width, x-frame-height",
                                )
                                .header("x-frame-width", frame.width.to_string())
                                .header("x-frame-height", frame.height.to_string())
                                .status(200)
                                .body(data)
                                .unwrap()
                                .map(Into::into)
                        } else {
                            Response::builder()
                                .status(500)
                                .header(header::CONTENT_TYPE, "text/plain")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(b"No frame data available".to_vec())
                                .unwrap()
                                .map(Into::into)
                        }
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
        }
    }
}
