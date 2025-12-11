use std::fmt::Error;
use std::fs::File;
use std::io::Read;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use screencapturekit::{
    CMSampleBuffer,
    cm::CVPixelBuffer,
    prelude::{PixelFormat, SCContentFilter, SCStreamConfiguration, SCStreamOutputType},
    stream::{SCStream, SCStreamOutput},
};

use super::{error::CaptureError, result::CaptureResult};

fn get_system_version() -> Result<f32, Error> {
    let path = "/System/Library/CoreServices/SystemVersion.plist";
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    println!("contents: {}", contents);
    // 找ProductVersion，使用文本搜索不用正则
    let search_text = "ProductVersion</key>\n";
    let mut index = contents.find(search_text).unwrap();
    index += search_text.len();
    let version = contents[index..].split("\n").next().unwrap().to_string();
    // 找到起始标签的结束位置
    let start = version.find('>').unwrap() + 1;
    // 找到结束标签的开始位置
    let end = version.rfind('<').unwrap();

    let v_str = version[start..end].to_string();
    Ok(v_str.parse::<f32>().unwrap())
}

/// 获取显示器的原始物理分辨率
///
/// 通过 Core Graphics API 获取显示器的 native mode（最高分辨率）
fn get_native_resolution(display_native_id: u32) -> Result<(usize, usize), CaptureError> {
    use core_graphics::display::CGDisplay;

    let display = CGDisplay::new(display_native_id);

    // 获取当前显示模式
    if let Some(current_mode) = display.display_mode() {
        // 获取像素宽度和高度（物理分辨率）
        let width = current_mode.pixel_width() as usize;
        let height = current_mode.pixel_height() as usize;

        log::debug!(
            "Display {} native resolution: {}x{}",
            display_native_id,
            width,
            height
        );

        return Ok((width, height));
    }

    // 如果无法获取，返回错误
    Err(CaptureError::ContentNotAvailable(format!(
        "无法获取显示器 {} 的显示模式",
        display_native_id
    )))
}

/// # 参数
/// * `display_id` - 显示器索引
/// * `show_cursor` - 是否显示光标
pub fn capture_screen(
    display_native_id: u32,
    show_cursor: bool,
) -> Result<CaptureResult, CaptureError> {
    use screencapturekit::prelude::SCShareableContent;

    let start_time = Instant::now();
    log::info!(
        "Start to initialize capture screen for (display_native_id: {})",
        display_native_id
    );

    let content =
        SCShareableContent::get().map_err(|e| CaptureError::ContentNotAvailable(e.to_string()))?;

    let displays = content.displays();
    if displays.is_empty() {
        return Err(CaptureError::ContentNotAvailable(
            "No displays found".to_string(),
        ));
    }
    // 找到 display_native_id 对应的 display
    let display = displays
        .iter()
        .find(|d| d.display_id() == display_native_id)
        .ok_or(CaptureError::DisplayNotFound(display_native_id))?;
    log::info!("Display: {:?}", display);
    let (width, height) = get_native_resolution(display_native_id)?;

    log::info!(
        "Display {} resolution: {}x{}",
        display_native_id,
        width,
        height
    );

    let filter = SCContentFilter::builder().display(display).build();

    let stream_config = SCStreamConfiguration::new()
        .with_shows_cursor(show_cursor)
        .with_width(width as u32)
        .with_height(height as u32)
        .with_pixel_format(PixelFormat::BGRA);

    let handler = SingleFrameHandler::new();
    let captured_buffer = handler.get_captured();

    // 创建流
    let mut stream = SCStream::new(&filter, &stream_config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);

    stream
        .start_capture()
        .map_err(|e| CaptureError::StreamError(format!("{:?}", e)))?;

    let wait_start = Instant::now();
    let timeout = Duration::from_millis(1000);
    let mut frame_received = false;

    while wait_start.elapsed() < timeout {
        std::thread::sleep(Duration::from_millis(10));
        if let Ok(buffer) = captured_buffer.lock() {
            if buffer.is_some() {
                frame_received = true;
                break;
            }
        }
    }
    let _ = stream.stop_capture();
    if !frame_received {
        return Err(CaptureError::Timeout);
    }

    let mut buffer = captured_buffer.lock().unwrap();
    if let Some(pixel_buffer) = buffer.take() {
        let width = pixel_buffer.width();
        let height = pixel_buffer.height();

        log::info!(
            "Capture screen success in {}ms (zero-copy)",
            start_time.elapsed().as_millis()
        );
        Ok(CaptureResult {
            pixel_buffer,
            width,
            height,
            native_id: display_native_id,
            show_cursor,
        })
    } else {
        Err(CaptureError::ImageProcessingError(
            "No data received".to_string(),
        ))
    }
}

/// 单帧捕获处理器（零拷贝）
struct SingleFrameHandler {
    captured_buffer: Arc<Mutex<Option<CVPixelBuffer>>>,
}

impl SingleFrameHandler {
    fn new() -> Self {
        Self {
            captured_buffer: Arc::new(Mutex::new(None)),
        }
    }

    fn get_captured(&self) -> Arc<Mutex<Option<CVPixelBuffer>>> {
        Arc::clone(&self.captured_buffer)
    }
}

impl SCStreamOutput for SingleFrameHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        if of_type != SCStreamOutputType::Screen {
            return;
        }

        let Some(buffer) = sample.image_buffer() else {
            return;
        };

        // 零拷贝：只克隆 CVPixelBuffer（增加引用计数，不拷贝数据）
        if let Ok(mut data) = self.captured_buffer.lock() {
            if data.is_none() {
                *data = Some(buffer.clone());
                log::debug!("Captured frame buffer (zero-copy)");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 保存截屏为 PNG 图片（仅用于测试）
    pub fn save_capture_as_png(capture: &CaptureResult, path: &str) -> Result<(), String> {
        use image::{ImageBuffer, Rgba};

        // 锁定像素缓冲区
        let guard = capture
            .lock_for_read()
            .map_err(|e| format!("锁定像素缓冲区失败: {:?}", e))?;

        let width = capture.width as u32;
        let height = capture.height as u32;
        let data = guard.as_slice();
        let bytes_per_row = guard.bytes_per_row();

        // BGRA 转 RGBA
        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height as usize {
            let row_start = y * bytes_per_row;
            for x in 0..width as usize {
                let pixel_start = row_start + x * 4;
                if pixel_start + 3 < data.len() {
                    let b = data[pixel_start];
                    let g = data[pixel_start + 1];
                    let r = data[pixel_start + 2];
                    let a = data[pixel_start + 3];

                    rgba_data.push(r);
                    rgba_data.push(g);
                    rgba_data.push(b);
                    rgba_data.push(a);
                }
            }
        }

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, rgba_data)
                .ok_or_else(|| "创建图像缓冲区失败".to_string())?;

        img.save(path).map_err(|e| format!("保存图片失败: {}", e))?;

        Ok(())
    }

    #[test]
    fn test_get_system_version() {
        let system_version = get_system_version();
        println!("system_version: {:?}", system_version);
    }

    #[test]
    fn test_capture_and_save() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .init();

        //获取所有显示器
        use core_graphics::display::CGDisplay;
        let displays = CGDisplay::active_displays()
            .map_err(|e| CaptureError::ContentNotAvailable(e.to_string()))
            .unwrap();
        for display in displays {
            match capture_screen(display, true) {
                Ok(capture) => {
                    println!("  ✓ 截屏成功");
                    println!("    分辨率: {}x{}", capture.width, capture.height);
                    println!("    显示器 ID: {}", capture.native_id);
                    println!("    是否显示鼠标: {}", capture.show_cursor);
                    let output_path = format!("test_screenshot_{}.png", capture.native_id);
                    match save_capture_as_png(&capture, &output_path) {
                        Ok(_) => {
                            // 验证文件是否存在并显示大小
                            if let Ok(metadata) = std::fs::metadata(output_path) {
                                println!(
                                    "  ✓ 文件大小: {:.2} MB",
                                    metadata.len() as f64 / 1024.0 / 1024.0
                                );
                            } else {
                                panic!("  ✗ 文件未找到");
                            }
                        }
                        Err(e) => {
                            panic!("  ✗ 保存图片失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("  ✗ 截屏失败: {:?}", e);
                }
            }
        }
    }
}
