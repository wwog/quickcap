use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use screencapturekit::{
    cm::CVPixelBuffer,
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
    log::info!("Start to initialize capture screen for display {}", display_id);

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
        
        log::info!("Capture screen success in {}ms (zero-copy)", start_time.elapsed().as_millis());
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
