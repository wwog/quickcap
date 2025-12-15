pub mod enumerate;

mod error;
mod frame;

pub use error::CaptureError;
pub use frame::Frame;
use tao::window::Window;

#[cfg(target_os = "macos")]
mod macos;

pub fn capscreen(display_id: u32) -> Result<Frame, CaptureError> {
    #[cfg(target_os = "macos")]
    {
        return macos::capscreen(display_id);
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(error::CaptureError::UnsupportedPlatform)
    }
}
#[allow(dead_code)]
pub fn configure_overlay_window(window: &Window) {
    #[cfg(target_os = "macos")]
    {
        macos::configure_overlay_window(window);
    }
}
