use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use screencapturekit::{
    CMSampleBuffer,
    prelude::{PixelFormat, SCContentFilter, SCStreamConfiguration, SCStreamOutputType},
    stream::{SCStream, SCStreamOutput},
};

use super::{error::CaptureError, result::CaptureResult};

/// # 参数
/// * `display_id` - 显示器索引
/// * `show_cursor` - 是否显示光标
pub fn capture_screen(display_id: usize, show_cursor: bool) -> Result<CaptureResult, CaptureError> {
    use screencapturekit::prelude::SCShareableContent;

    let start_time = Instant::now();
    log::info!("开始为显示器 {} 初始化截屏...", display_id);

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

    let width = display.width();
    let height = display.height();

    let filter = SCContentFilter::builder().display(display).build();

    let stream_config = SCStreamConfiguration::new()
        .with_shows_cursor(show_cursor)
        .with_width(width)
        .with_height(height)
        .with_pixel_format(PixelFormat::BGRA);

    let handler = SingleFrameHandler::new();
    let captured_data = handler.get_captured();

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
        if let Ok(data) = captured_data.lock() {
            if data.is_some() {
                frame_received = true;
                break;
            }
        }
    }
    // 停止流
    let _ = stream.stop_capture();
    if !frame_received {
        return Err(CaptureError::Timeout);
    }
    let mut data = captured_data.lock().unwrap();
    if let Some((data, width, height)) = data.take() {
        log::info!("Capture screen success in {}ms", start_time.elapsed().as_millis());
        Ok(CaptureResult {
            data,
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

/// 单帧捕获处理器
struct SingleFrameHandler {
    captured_data: Arc<Mutex<Option<(Vec<u8>, usize, usize)>>>,
}

impl SingleFrameHandler {
    fn new() -> Self {
        Self {
            captured_data: Arc::new(Mutex::new(None)),
        }
    }

    fn get_captured(&self) -> Arc<Mutex<Option<(Vec<u8>, usize, usize)>>> {
        Arc::clone(&self.captured_data)
    }
}

impl SCStreamOutput for SingleFrameHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        if of_type == SCStreamOutputType::Screen {
            if let Ok(mut data) = self.captured_data.lock() {
                // 只捕获第一帧
                if data.is_none() {
                    if let Some(buffer) = sample.image_buffer() {
                        // 锁定基地址（只读模式）
                        if let Ok(_lock) = buffer.lock_base_address(false) {
                            let width = buffer.width();
                            let height = buffer.height();
                            let bytes_per_row = buffer.bytes_per_row();

                            if let Some(base_address) = buffer.base_address() {
                                // 复制图像数据
                                let length = height * bytes_per_row;
                                let mut rgb_data = vec![0u8; length];
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        base_address as *const u8,
                                        rgb_data.as_mut_ptr(),
                                        length,
                                    );
                                }
                                *data = Some((rgb_data, width, height));
                            }
                            // _lock 在这里自动解锁
                        }
                    }
                }
            }
        }
    }
}
