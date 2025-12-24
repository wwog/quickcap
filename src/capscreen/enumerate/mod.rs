mod structs;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub use structs::WindowInfo;
use tao::monitor::MonitorHandle;

/// 枚举所有窗口（不筛选显示器）
/// macOS和Windows都返回所有窗口，使用绝对坐标
pub fn enumerate_all_windows() -> Option<Vec<WindowInfo>> {
    #[cfg(target_os = "macos")]
    {
        macos::enumerate_all_windows()
    }
    #[cfg(not(target_os = "macos"))]
    {
        windows::enumerate_windows()
    }
}

#[cfg(target_os = "macos")]
/// macOS专用的按显示器筛选窗口
/// 输入：使用绝对坐标的窗口列表
/// 输出：使用相对于显示器坐标的窗口列表
pub fn filter_windows_by_display(
    all_windows: &[WindowInfo],
    display_id: u32,
) -> Option<Vec<WindowInfo>> {
    macos::filter_windows_by_display(all_windows, display_id)
}

#[allow(unused)]
pub fn enumerate_windows(handle: &MonitorHandle) -> Vec<WindowInfo> {
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::MonitorHandleExtMacOS;

        macos::enumerate_windows(handle.native_id()).unwrap_or_default()
    }
    #[cfg(not(target_os = "macos"))]
    {
        windows::enumerate_windows().unwrap_or_default()
    }
}