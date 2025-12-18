use crate::app::AppEvent;
use crate::app::capscreen::enumerate::enumerate_windows;
use crate::app::capscreen::{Frame, capscreen, configure_overlay_window};
use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};
use std::{sync::Arc, time::Instant};
#[cfg(target_os = "macos")]
use tao::platform::macos::WindowBuilderExtMacOS;
use tao::{
    event_loop::{EventLoop, EventLoopProxy},
    monitor::MonitorHandle,
    window::{Window, WindowBuilder},
};
use wgpu::rwh::HasWindowHandle;

#[cfg(target_os = "windows")]
use tao::platform::windows::{MonitorHandleExtWindows, WindowBuilderExtWindows};

use wry::{
    WebView, WebViewBuilder,
    http::{Response, header},
};

use dirs;
use rfd::FileDialog;
use std::path::PathBuf;

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
            .with_transparent(true);
        #[cfg(target_os = "macos")]
        {
            win_builder = win_builder
                .with_has_shadow(false)
                .with_position(position)
                .with_inner_size(size)
        }
        #[cfg(target_os = "windows")]
        {
            win_builder =
                win_builder.with_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
        }

        let window = Arc::new(win_builder.build(event_loop).unwrap());

        // configure_overlay_window(&window);

        let frame = capscreen(&monitor).unwrap();

        // 只克隆 Arc，不克隆底层数据
        let data_arc = Arc::clone(&frame.data);
        let frame_width = frame.width;
        let frame_height = frame.height;
        let monitor_for_enum = monitor.clone();

        let webview = WebViewBuilder::new()
            // .with_html(include_str!("demo.html"))
            // .with_html(include_str!("dist/index.html"))
            // .with_url("quickcap://index.html/")
            .with_url("http://localhost:5173/")
            .with_devtools(true)
            .with_transparent(true)
            .with_initialization_script(include_str!("preload.js"))
            .with_custom_protocol("quickcap".into(), move |_name, req| {
                let path = req.uri().to_string();
                let method = req.method();
                // log::info!("path: {:?}, method: {:?}", path, method);

                // 处理 OPTIONS 预检请求
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
                        let windows = enumerate_windows(&monitor_for_enum);
                        let json =
                            serde_json::to_string(&windows).unwrap_or_else(|_| "[]".to_string());
                        log::info!("quickcap://windows/ : {:#?}", json);
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
            } else if body.starts_with("clipboard:") {
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
            } else if body.starts_with("save:") {
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
        }
    }

    // 将图像复制到剪切板的辅助函数
    fn copy_image_to_clipboard(image_data: Vec<u8>) {
        match ClipboardContext::new() {
            Ok(mut ctx) => {
                let str = if cfg!(target_os = "macos") {
                    "public.png"
                } else {
                    "image/png"
                };
                match ctx.set_buffer(&str, image_data) {
                    Ok(_) => println!("图像已成功复制到剪贴板"),
                    Err(e) => eprintln!("写入剪贴板失败: {}", e),
                }
            }
            Err(e) => eprintln!("创建剪贴板上下文失败: {}", e),
        }
    }

    fn save_image_to_folder(image_data: Vec<u8>, proxy: EventLoopProxy<AppEvent>) {
        // 设置默认目录为下载目录，如果不存在则使用根目录
        let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("/"));

        // 生成默认文件名（使用友好格式）
        let now = chrono::Local::now();
        let date_str = now.format("%Y%m%d");
        let time_str = now.format("%H%M%S");
        let default_name = format!("screenshot{}{}.png", date_str, time_str);

        /* let file_path = DialogBuilder::file()
        .set_filename(&default_name)
        .set_owner(&window.window_handle().unwrap())
        .set_location(&download_dir)
        .add_filter("PNG 图像", &["png"])
        .save_single_file()
        .show()
        .unwrap(); */

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
