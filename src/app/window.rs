use crate::app::user_event::UserEvent;
use crate::capscreen::capscreen;
use crate::capscreen::enumerate::enumerate_windows;
use clipboard_rs::{Clipboard, ClipboardContext, RustImageData, common::RustImage};
use image::DynamicImage;
use std::{
    sync::{Arc, Condvar, Mutex},
    time::Instant,
};

use tao::{
    event_loop::{EventLoop, EventLoopProxy},
    monitor::MonitorHandle,
    window::{Window, WindowBuilder},
};
// use wgpu::rwh::HasWindowHandle;

#[allow(unused_imports)]
#[cfg(target_os = "windows")]
use tao::platform::windows::{MonitorHandleExtWindows, WindowBuilderExtWindows};

use crate::app::user_event::UserEvent as AppEvent;
use chrono::{DateTime, Local};
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
                tao::dpi::LogicalPosition::new(x_virtual_screen as f64, y_virtual_screen as f64);
            let size =
                tao::dpi::LogicalSize::new(cx_virtual_screen as f64, cy_virtual_screen as f64);
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
            log::error!("capscreen time: {:?}", start_capscreen_time.elapsed());

            // 只克隆 Arc，不克隆底层数据
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
            .with_devtools(true)
            .with_transparent(true)
            .with_initialization_script(include_str!("preload.js"))
            .with_accept_first_mouse(true)
            .with_ipc_handler(move |req| {
                let body = req.body();
                log::error!("ipc body: {:?}", body);
                match body.as_str() {
                    "exit" => {
                        proxy.send_event(UserEvent::Exit).unwrap_or_else(|e| {
                            log::error!("send event failed: {:?}", e);
                        });
                    }
                    body if body.starts_with("clipboard:") => {
                        // 处理剪切板操作
                        let base64_data = if body.starts_with("clipboard:base64:") {
                            // 新格式
                            body.split("clipboard:base64:").nth(1).unwrap_or("")
                        } else {
                            // 兼容旧格式（clipboard:base64）
                            body.split(":").nth(1).unwrap_or("")
                        };

                        if !base64_data.is_empty() {
                            match base64::decode(base64_data) {
                                Ok(image_data) => {
                                    Self::copy_image_to_clipboard(image_data);
                                }
                                Err(e) => {
                                    log::error!("Failed to decode base64 image data: {}", e);
                                }
                            }
                        }
                        proxy.send_event(AppEvent::Exit);
                    }
                    body if body.starts_with("save:") => {
                        let base64_data = body.split("save:").nth(1).unwrap_or("");
                        if !base64_data.is_empty() {
                            match base64::decode(base64_data) {
                                Ok(image_data) => {
                                    Self::save_image_to_folder(image_data, proxy.clone());
                                }
                                Err(e) => {
                                    log::error!("Failed to decode base64 image data: {}", e);
                                }
                            }
                        }
                    }
                    _ => {
                        // 尝试解析 JSON 消息
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
                    "/copy" => {
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
                        let Ok(ctx) = ClipboardContext::new() else {
                            log::error!("create clipboard context failed");
                            return Response::builder()
                                .status(400)
                                .body(b"create clipboard context failed".to_vec())
                                .unwrap()
                                .map(Into::into);
                        };
                        let image = image::ImageBuffer::from_vec(width, height, body).unwrap();
                        let image_data =
                            RustImageData::from_dynamic_image(DynamicImage::ImageRgba8(image));
                        let before_set_image_time = Instant::now();
                        let _ = ctx.set_image(image_data);
                        log::error!("set image time: {:?}", before_set_image_time.elapsed());
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

    // 将图像复制到剪切板的辅助函数
    fn copy_image_to_clipboard(image_data: Vec<u8>) {
        match ClipboardContext::new() {
            Ok(mut ctx) => {
                match RustImageData::from_bytes(&image_data) {
                    Ok(rust_image) => match ctx.set_image(rust_image) {
                        Ok(_) => println!("图像已成功复制到剪贴板"),
                        Err(e) => eprintln!("写入剪贴板失败: {}", e),
                    },
                    Err(e) => eprintln!("将图像数据转换为RustImageData失败: {}", e),
                }
            }
            Err(e) => eprintln!("创建剪贴板上下文失败: {}", e),
        }
    }

    fn save_image_to_folder(image_data: Vec<u8>, proxy: EventLoopProxy<UserEvent>) {
        // 设置默认目录为下载目录，如果不存在则使用根目录
        let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("/"));

        // 生成默认文件名（使用友好格式）
        let now = Local::now();
        let date_str = now.format("%Y%m%d");
        let time_str = now.format("%H%M%S");
        let default_name = format!("screenshot{}{}.png", date_str, time_str);

        // 使用文件保存对话框，用户可以查看和修改文件名
        let file_path = FileDialog::new()
            .add_filter("PNG", &["png"])
            .set_directory(&download_dir)
            .set_can_create_directories(true)
            .set_file_name(&default_name)
            .save_file();

        if let Some(file_path) = file_path {
            println!("用户选择的文件路径: {}", file_path.display());
            Self::copy_image_to_clipboard(image_data.clone());

            // 将图像数据保存到用户指定的路径
            if let Err(e) = std::fs::write(&file_path, image_data) {
                eprintln!("保存图像失败: {}", e);
            } else {
                println!("图像已成功保存到: {}", file_path.display());
            }
            proxy.send_event(AppEvent::Exit);
        } else {
            println!("用户取消了文件保存");
        }
    }
}
