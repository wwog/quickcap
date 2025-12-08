use screencapturekit::prelude::SCContentFilter;

use super::error::CaptureError;
use crate::cap::result::CaptureResult;

pub fn capture_screen(display_id: usize, show_cursor: bool) -> Result<CaptureResult, CaptureError> {
    use screencapturekit::prelude::SCShareableContent;

    let content =
        SCShareableContent::get().map_err(|e| CaptureError::ContentNotAvailable(e.to_string()))?;

    let displays = content.displays();
    if displays.is_empty() {
        return Err(CaptureError::ContentNotAvailable(
            "No displays found".to_string(),
        ));
    }
    let display = displays
        .get(display_id)
        .ok_or(CaptureError::DisplayNotFound(display_id))?;

    let filter = SCContentFilter::builder().display(display).build();


    todo!()
}
