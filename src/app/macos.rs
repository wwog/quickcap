use screencapturekit::{
    prelude::{CGDisplay, PixelFormat, SCContentFilter, SCShareableContent, SCStreamConfiguration},
    screenshot_manager::{CGImage, capture_image_with_stream},
};

pub enum CaptureError {
    FailedToGetShareableContent,
    FailedToFindDisplay,
    FailedToGetDisplayMode,
    FailedToCaptureImage,
}

pub fn capscreen(display_id: u32) -> Result<CGImage, CaptureError> {
    let content = SCShareableContent::get().map_err(|_| {
        log::error!("Failed to get shareable content");
        CaptureError::FailedToGetShareableContent
    })?;

    let displays = content.displays();
    let Some(sc_display) = displays
        .into_iter()
        .find(|display| display.display_id() == display_id)
    else {
        log::error!("Failed to find display with id: {}", display_id);
        return Err(CaptureError::FailedToFindDisplay);
    };

    let display_mode = CGDisplay::new(display_id).display_mode().ok_or_else(|| {
        log::error!("Failed to get display mode for id: {}", display_id);
        CaptureError::FailedToGetDisplayMode
    })?;
    let height = display_mode.logical_height();
    let width = display_mode.logical_width();

    let filter = SCContentFilter::builder().display(&sc_display).build();
    let config = SCStreamConfiguration::new()
        .with_width(width as u32)
        .with_height(height as u32)
        .with_pixel_format(PixelFormat::BGRA)
        .with_shows_cursor(false);
    let image = capture_image_with_stream(&filter, &config).map_err(|_| {
        log::error!("Failed to capture image with stream");
        CaptureError::FailedToCaptureImage
    })?;
    Ok(image)
}
