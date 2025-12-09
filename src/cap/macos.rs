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

/// 获取显示器的原始物理分辨率
///
/// 通过 Core Graphics API 获取显示器的 native mode（最高分辨率）
fn get_native_resolution(display_id: usize) -> Result<(usize, usize), CaptureError> {
    use core_graphics::display::CGDisplay;

    // 获取主显示器或指定显示器的 ID
    let display_ids = CGDisplay::active_displays()
        .map_err(|e| CaptureError::ContentNotAvailable(format!("获取显示器列表失败: {:?}", e)))?;

    if display_id >= display_ids.len() {
        return Err(CaptureError::DisplayNotFound(display_id));
    }

    let cg_display_id = display_ids[display_id];
    let display = CGDisplay::new(cg_display_id);

    // 获取当前显示模式
    if let Some(current_mode) = display.display_mode() {
        // 获取像素宽度和高度（物理分辨率）
        let width = current_mode.pixel_width() as usize;
        let height = current_mode.pixel_height() as usize;

        log::debug!(
            "Display {} native resolution: {}x{}",
            display_id,
            width,
            height
        );

        return Ok((width, height));
    }

    // 如果无法获取，返回错误
    Err(CaptureError::ContentNotAvailable(format!(
        "无法获取显示器 {} 的显示模式",
        display_id
    )))
}

/// # 参数
/// * `display_id` - 显示器索引
/// * `show_cursor` - 是否显示光标
/// * `use_native_resolution` - 是否使用原始物理分辨率（true=物理分辨率，false=逻辑分辨率）
pub fn capture_screen(
    display_id: usize,
    show_cursor: bool,
    use_native_resolution: bool,
) -> Result<CaptureResult, CaptureError> {
    use screencapturekit::prelude::SCShareableContent;

    let start_time = Instant::now();
    log::info!(
        "Start to initialize capture screen for display {} (native_res: {})",
        display_id,
        use_native_resolution
    );

    let content =
        SCShareableContent::get().map_err(|e| CaptureError::ContentNotAvailable(e.to_string()))?;

    let displays = content.displays();
    if displays.is_empty() {
        return Err(CaptureError::ContentNotAvailable(
            "No displays found".to_string(),
        ));
    }
    let display = displays
        .get(display_id)
        .ok_or(CaptureError::DisplayNotFound(display_id))?;

    let (width, height) = if use_native_resolution {
        get_native_resolution(display_id)?
    } else {
        (display.width() as usize, display.height() as usize)
    };

    log::info!("Display {} resolution: {}x{}", display_id, width, height);

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
            display_id,
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
    fn test_capture_and_save() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .init();

        println!("\n=== 测试截屏功能 ===\n");

        // 测试逻辑分辨率
        println!("1. 测试逻辑分辨率（use_native_resolution=false）:");
        match capture_screen(0, true, false) {
            Ok(capture) => {
                println!("  ✓ 截屏成功");
                println!("    分辨率: {}x{}", capture.width, capture.height);
                println!("    显示器 ID: {}", capture.display_id);
                println!("    是否显示鼠标: {}", capture.show_cursor);
                let output_path = "test_screenshot_logic_res.png";
                println!("    保存图片到: test_screenshot_logic_res.png");
                match save_capture_as_png(&capture, output_path) {
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

        // 测试物理分辨率
        println!("\n2. 测试物理分辨率（use_native_resolution=true）:");
        match capture_screen(0, true, true) {
            Ok(capture) => {
                println!("  ✓ 截屏成功");
                println!("    分辨率: {}x{}", capture.width, capture.height);
                println!("    显示器 ID: {}", capture.display_id);

                // 保存为 PNG
                let output_path = "test_screenshot_native_res.png";
                println!("\n3. 保存图片到: {}", output_path);
                match save_capture_as_png(&capture, output_path) {
                    Ok(_) => {
                        println!("  ✓ 图片已保存");

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
                panic!("  ✗ 截屏失败: {:?}", e);
            }
        }

        println!("\n=== 测试完成 ===\n");
    }
}
