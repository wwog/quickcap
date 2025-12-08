#[allow(dead_code)]
#[derive(Debug)]
pub enum CaptureError {
    /// 无法获取可共享内容
    ContentNotAvailable(String),
    /// 指定的显示器不存在
    DisplayNotFound(usize),
    /// 流创建或启动失败
    StreamError(String),
    /// 超时未收到帧数据
    Timeout,
    /// 图像数据处理失败
    ImageProcessingError(String),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::ContentNotAvailable(msg) => {
                write!(f, "无法获取可共享内容: {}", msg)
            }
            CaptureError::DisplayNotFound(id) => write!(f, "显示器 {} 不存在", id),
            CaptureError::StreamError(msg) => write!(f, "流错误: {}", msg),
            CaptureError::Timeout => write!(f, "截屏超时"),
            CaptureError::ImageProcessingError(msg) => write!(f, "图像处理错误: {}", msg),
        }
    }
}

impl std::error::Error for CaptureError {}