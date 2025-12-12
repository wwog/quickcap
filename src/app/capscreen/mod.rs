mod error;
mod frame;

pub use error::CaptureError;
pub use frame::Frame;

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
