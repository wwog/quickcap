// 公共类型定义（跨平台通用）
mod error;
mod result;

// 导出公共类型
pub use error::CaptureError;
pub use result::CaptureResult;

// 平台特定实现
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

// 统一的公共 API
/// 截取指定显示器的屏幕
///
/// # 参数
/// * `display_id` - 显示器索引（从 0 开始）
/// * `show_cursor` - 是否在截图中显示鼠标光标
///
/// # 返回
/// 返回包含图像数据的 `CaptureResult`，或者截屏失败时的 `CaptureError`
pub fn capture_screen(display_id: usize, show_cursor: bool) -> Result<CaptureResult, CaptureError> {
    platform::capture_screen(display_id, show_cursor)
}

