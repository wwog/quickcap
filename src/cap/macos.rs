use crate::cap::result::CaptureResult;
use super::error::CaptureError;

pub fn capture_screen(display_id: usize, show_cursor: bool) -> Result<CaptureResult,CaptureError>  {
    use screencapturekit::prelude::SCShareableContent;

    let content = SCShareableContent::get().unwrap();


    todo!()
}