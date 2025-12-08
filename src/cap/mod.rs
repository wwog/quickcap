// 公共类型定义（跨平台通用）
mod error;
mod result;

// 导出公共类型
pub use error::CaptureError;
pub use result::CaptureResult;

// 平台特定实现
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

/// 屏幕截取 trait
/// 
/// 实现此 trait 的类型可以进行屏幕截取
pub trait ScreenCapture {
    /// 截取屏幕
    ///
    /// # 参数
    /// * `show_cursor` - 是否在截图中显示鼠标光标
    ///
    /// # 返回
    /// 返回包含图像数据的 `CaptureResult`，或者截屏失败时的 `CaptureError`
    fn capture_screen(&self, show_cursor: bool) -> Result<CaptureResult, CaptureError>;
}

