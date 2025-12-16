pub mod enumerate;

mod error;
mod frame;

pub use error::CaptureError;
pub use frame::Frame;
use tao::{monitor::MonitorHandle, window::Window};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub fn capscreen(handle: &MonitorHandle) -> Result<Frame, CaptureError> {
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::MonitorHandleExtMacOS;

        return macos::capscreen(handle.native_id());
    }
    #[cfg(not(target_os = "macos"))]
    {
        return windows::capscreen_windows(handle);
    }
}
#[allow(dead_code)]
pub fn configure_overlay_window(window: &Window) {
    #[cfg(target_os = "macos")]
    {
        macos::configure_overlay_window(window);
    }
}
