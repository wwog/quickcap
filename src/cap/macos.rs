use screencapturekit::prelude::SCContentFilter;

use super::{error::CaptureError, result::CaptureResult, ScreenCapture};
use crate::app::AppWindow;

impl ScreenCapture for AppWindow {
    fn capture_screen(&self, _show_cursor: bool) -> Result<CaptureResult, CaptureError> {
        use screencapturekit::prelude::SCShareableContent;

        let content = SCShareableContent::get()
            .map_err(|e| CaptureError::ContentNotAvailable(e.to_string()))?;

        let displays = content.displays();
        if displays.is_empty() {
            return Err(CaptureError::ContentNotAvailable(
                "No displays found".to_string(),
            ));
        }
        let display = displays
            .get(self.display_id)
            .ok_or(CaptureError::DisplayNotFound(self.display_id))?;

        let _filter = SCContentFilter::builder().display(display).build();

        // TODO: 实现实际的截屏逻辑
        todo!()
    }
}
