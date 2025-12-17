use log::error;
use std::{os::raw::c_void, sync::Arc};
use tao::{monitor::MonitorHandle, platform::windows::MonitorHandleExtWindows};
use windows_capture::{dxgi_duplication_api::DxgiDuplicationApi, monitor::Monitor};

use crate::app::capscreen::{CaptureError, Frame};

pub fn capscreen_windows(handle: &MonitorHandle) -> Result<Frame, CaptureError> {
    let h_monitor = handle.hmonitor();
    let monitor = Monitor::from_raw_hmonitor(h_monitor as *mut c_void);

    let mut dup = DxgiDuplicationApi::new(monitor).map_err(|e| {
        error!("init DxgiDuplicationApi failed: {:?}", e);
        CaptureError::FailedToInitDuplication
    })?;

    let mut n_frame = dup.acquire_next_frame(100).map_err(|e| {
        error!("acquire_next_frame failed: {:?}", e);
        CaptureError::FailedToAcquireFrame
    })?;
    let mut buffer = n_frame.buffer().map_err(|e| {
        error!("get frame buffer failed: {:?}", e);
        CaptureError::FailedToGetBuffer
    })?;
    log::info!("buffer format: {:?}", buffer.format());
    let data = buffer.as_raw_buffer().to_vec();
    let frame = Frame {
        data: Arc::new(data),
        width: n_frame.width(),
        height: n_frame.height(),
    };
    Ok(frame)
}
