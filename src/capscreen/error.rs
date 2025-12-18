#[allow(dead_code)]
#[derive(Debug)]
pub enum CaptureError {
    UnsupportedPlatform,
    FailedToGetShareableContent,
    FailedToFindDisplay,
    FailedToGetDisplayMode,
    FailedToCaptureImage,
    FailedToGetRGBAData,
    FailedToGetBuffer,
}