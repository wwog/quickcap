use super::error::CaptureError;
use screencapturekit::cm::CVPixelBuffer;

#[allow(dead_code)]
#[derive(Debug)]
pub struct CaptureResult {
    /// 像素缓冲区（零拷贝）
    pub pixel_buffer: CVPixelBuffer,
    /// 图像宽度
    pub width: usize,
    /// 图像高度
    pub height: usize,
    /// 显示器 ID
    pub native_id: u32,
    /// 是否显示鼠标
    pub show_cursor: bool,
}
#[allow(dead_code)]
impl CaptureResult {
    /// 零拷贝锁定数据（用于读取/渲染）
    pub fn lock_for_read(&self) -> Result<CaptureDataGuard<'_>, CaptureError> {
        let lock = self
            .pixel_buffer
            .lock_base_address(true) // 只读锁定
            .map_err(|e| CaptureError::ImageProcessingError(format!("Lock failed: {}", e)))?;

        let bytes_per_row = self.pixel_buffer.bytes_per_row();

        Ok(CaptureDataGuard {
            lock,
            width: self.width,
            height: self.height,
            bytes_per_row,
        })
    }

    /// 获取字节/行（用于 wgpu 等）
    pub fn bytes_per_row(&self) -> usize {
        self.pixel_buffer.bytes_per_row()
    }

    /// 如果需要拥有数据副本，可以手动拷贝
    pub fn to_vec(&self) -> Result<Vec<u8>, CaptureError> {
        let lock = self.lock_for_read()?;
        Ok(lock.as_slice().to_vec())
    }
}

/// RAII 守卫：自动管理锁的生命周期
#[allow(dead_code)]
pub struct CaptureDataGuard<'a> {
    lock: screencapturekit::cm::CVPixelBufferLockGuard<'a>,
    width: usize,
    height: usize,
    bytes_per_row: usize,
}

impl<'a> CaptureDataGuard<'a> {
    /// 获取原始字节切片（用于 wgpu 等）
    pub fn as_slice(&self) -> &[u8] {
        let ptr = self.lock.base_address();
        let len = self.height * self.bytes_per_row;
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
    #[allow(dead_code)]
    pub fn width(&self) -> usize {
        self.width
    }
    #[allow(dead_code)]
    pub fn height(&self) -> usize {
        self.height
    }
    #[allow(dead_code)]
    pub fn bytes_per_row(&self) -> usize {
        self.bytes_per_row
    }
}
