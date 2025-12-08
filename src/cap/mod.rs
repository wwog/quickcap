// 公共类型定义（跨平台通用）
mod error;
mod result;

// 平台特定实现
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "macos")]
pub use macos::capture_screen;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::capture_screen;