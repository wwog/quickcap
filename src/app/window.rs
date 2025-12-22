use crate::app::user_event::UserEvent;
use crate::capscreen::capscreen;
use crate::capscreen::enumerate::enumerate_windows;
use arboard::ImageData;
use png::{BitDepth, ColorType, Encoder, Filter};
use std::{
    borrow::Cow,
    fs::File,
    io::BufWriter,
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

#[allow(unused_imports)]
#[cfg(target_os = "macos")]
use tao::platform::macos::MonitorHandleExtMacOS;

use chrono::Local;
use dirs;
use rfd::FileDialog;
use std::path::PathBuf;
use wry::{
    WebView, WebViewBuilder,
    http::{Response, header},
};

// static FILEDATA: &[u8] = include_bytes!("demo.html");
static FILEDATA: &[u8] = include_bytes!("index.html");

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
            log::error!(
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
            let position =
                tao::dpi::PhysicalPosition::new(x_virtual_screen as f64, y_virtual_screen as f64);
            let size =
                tao::dpi::PhysicalSize::new(cx_virtual_screen as f64, cy_virtual_screen as f64);
            log::error!(
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
            .with_min_inner_size(size)
            .with_minimizable(false)
            .with_maximizable(false);
        // .with_always_on_top(true);

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

        // crate::capscreen::configure_overlay_window(&window);

        let capture_state: Arc<(Mutex<CaptureState>, Condvar)> = Arc::new((
            Mutex::new(CaptureState {
                frame: None,
                error: None,
                done: false,
            }),
            Condvar::new(),
        ));

        let capture_state_for_thread = Arc::clone(&capture_state);
        let monitor_for_capture = monitor.clone();
        std::thread::spawn(move || {
            let start_capscreen_time = Instant::now();
            let result = capscreen(&monitor_for_capture);
            log::error!("capscreen time: {:?}", start_capscreen_time.elapsed());

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

        let window_for_dialog = Arc::clone(&window);
        let capture_state_for_bg = Arc::clone(&capture_state);
        let monitor_for_enum = monitor.clone();

        let webview = WebViewBuilder::new()
            .with_devtools(true)
            .with_transparent(true)
            .with_initialization_script(include_str!("preload.js"))
            .with_ipc_handler(move |req| {
                let body = req.body();
                log::error!("ipc body: {:?}", body);
                match body.as_str() {
                    "exit" => {
                        proxy.send_event(UserEvent::Exit).unwrap_or_else(|e| {
                            log::error!("send event failed: {:?}", e);
                        });
                    }
                    _ => {
                        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(body) {
                            if msg.get("type").and_then(|t| t.as_str()) == Some("notify") {
                                if let (Some(method), Some(params)) = (
                                    msg.get("method").and_then(|m| m.as_str()),
                                    msg.get("params"),
                                ) {
                                    crate::StdRpcClient::global()
                                        .send_notification(method, Some(params.clone()));
                                }
                            }
                        }
                    }
                }
            })
            .with_custom_protocol("app".into(), move |_name, req| {
                let path = req.uri().path().to_string();
                // log::error!("path: {:?}", path);
                match path.as_str() {
                    "/save" => {
                        let headers = req.headers();
                        let width = headers
                            .get("x-frame-width")
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .parse::<u32>()
                            .unwrap();
                        let height = headers
                            .get("x-frame-height")
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .parse::<u32>()
                            .unwrap();

                        let body = req.into_body();

                        let download_dir =
                            dirs::download_dir().unwrap_or_else(|| PathBuf::from("/"));
                        let now = Local::now();
                        let date_str = now.format("%Y%m%d");
                        let time_str = now.format("%H%M%S");
                        let default_name = format!("screenshot{}{}.png", date_str, time_str);
                        let file_path = FileDialog::new()
                            .add_filter("PNG", &["png"])
                            .set_directory(&download_dir)
                            .set_can_create_directories(true)
                            .set_file_name(&default_name)
                            .set_parent(&*window_for_dialog)
                            .save_file();
                        if file_path.is_none() {
                            return Response::builder()
                                .status(201)
                                .body(b"cancel".to_vec())
                                .unwrap()
                                .map(Into::into);
                        }
                        let file = File::create(file_path.unwrap()).unwrap();
                        let writer = BufWriter::new(file);

                        let mut encoder = Encoder::new(writer, width, height);
                        encoder.set_color(ColorType::Rgba);
                        encoder.set_depth(BitDepth::Eight);

                        encoder.set_compression(png::Compression::NoCompression);
                        encoder.set_filter(Filter::NoFilter);
                        let start = Instant::now();
                        let mut png_writer = encoder.write_header().unwrap();
                        png_writer.write_image_data(&body).unwrap();
                        log::error!("save image time: {:?}", start.elapsed());
                        Response::builder()
                            .status(200)
                            .body(b"success".to_vec())
                            .unwrap()
                            .map(Into::into)
                    }
                    "/copy" => {
                        let headers = req.headers();
                        let width = headers
                            .get("x-frame-width")
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .parse::<usize>()
                            .unwrap();
                        let height = headers
                            .get("x-frame-height")
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .parse::<usize>()
                            .unwrap();
                        let body = req.body();
                        let start = Instant::now();
                        let image = ImageData {
                            width,
                            height,
                            bytes: Cow::Borrowed(body),
                        };
                        log::error!("create image time: {:?}", start.elapsed());
                        arboard::Clipboard::new().unwrap().set_image(image).unwrap();
                        log::error!("set image time: {:?}", start.elapsed());

                        Response::builder()
                            .status(200)
                            .body(b"success".to_vec())
                            .unwrap()
                            .map(Into::into)
                    }
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
                        log::error!(
                            "monitor: {}, windows: {:?}",
                            monitor_for_enum.native_id(),
                            windows
                        );
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
            .with_bounds(wry::Rect {
                position: tao::dpi::Position::Physical(tao::dpi::PhysicalPosition::new(0, 0)),
                size: tao::dpi::Size::Physical(tao::dpi::PhysicalSize::new(size.width as u32, size.height as u32)),
            })
            .build_as_child(&window)
            .unwrap();

        Self {
            window,
            webview,
            monitor,
        }
    }
}
