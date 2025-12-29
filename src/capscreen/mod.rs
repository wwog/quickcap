pub mod enumerate;

mod error;
mod frame;

pub use error::CaptureError;
pub use frame::Frame;
use tao::{monitor::MonitorHandle, window::Window};

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[allow(unused)]
pub fn capscreen(handle: &MonitorHandle) -> Result<Frame, CaptureError> {
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::MonitorHandleExtMacOS;

        return macos::capscreen(handle.native_id());
    }
    #[cfg(not(target_os = "macos"))]
    {
        return windows::capscreen();
    }
}

#[allow(unused)]
pub fn configure_overlay_window(window: &Window) {
    log::error!("configure_overlay_window");
    #[cfg(target_os = "macos")]
    {
        macos::configure_overlay_window(window);
    }
    window.set_always_on_top(true);
}
