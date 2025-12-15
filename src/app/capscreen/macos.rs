use crate::app::capscreen::{error::CaptureError, frame::Frame};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSScreenSaverWindowLevel, NSWindow, NSWindowCollectionBehavior};
use screencapturekit::{
    prelude::{CGDisplay, PixelFormat, SCContentFilter, SCShareableContent, SCStreamConfiguration},
    screenshot_manager::capture_image_with_stream,
};
use tao::platform::macos::WindowExtMacOS;
use tao::window::Window;

pub fn capscreen(display_id: u32) -> Result<Frame, CaptureError> {
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
    let height = display_mode.pixel_height();
    let width = display_mode.pixel_width();

    let filter = SCContentFilter::builder().display(&sc_display).build();
    let config = SCStreamConfiguration::new()
        .with_width(width as u32)
        .with_height(height as u32)
        .with_pixel_format(PixelFormat::BGRA)
        .with_shows_cursor(false);
    let cg_image = capture_image_with_stream(&filter, &config).map_err(|_| {
        log::error!("Failed to capture image with stream");
        CaptureError::FailedToCaptureImage
    })?;
    let data = cg_image.rgba_data().map_err(|_| {
        log::error!("Failed to get rgba data");
        CaptureError::FailedToGetRGBAData
    })?;
    Ok(Frame {
        data: std::sync::Arc::new(data),
        width: width as u32,
        height: height as u32,
    })
}

pub fn configure_overlay_window(window: &Window) {
    unsafe {
        let ns_window_ptr = window.ns_window() as *mut AnyObject;
        let ns_window: Retained<NSWindow> =
            Retained::retain(ns_window_ptr as *mut NSWindow).unwrap();
        //设置窗口级别为窗口保护程序
        ns_window.setLevel(NSScreenSaverWindowLevel);

        let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::IgnoresCycle;
        ns_window.setCollectionBehavior(behavior);

        ns_window.setHidesOnDeactivate(false);
        ns_window.setIgnoresMouseEvents(false);

        log::info!("Configured window as overlay with NSScreenSaverWindowLevel");
    }
}
