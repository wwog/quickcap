mod structs;

#[cfg(target_os = "macos")]
mod macos;

pub use structs::WindowInfo;
use tao::monitor::MonitorHandle;

pub fn enumerate_windows(handle: &MonitorHandle) -> Vec<WindowInfo> {
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::MonitorHandleExtMacOS;

        macos::enumerate_windows(handle.native_id()).unwrap_or_default()
    }
    #[cfg(not(target_os = "macos"))]
    {
        vec![]
    }
}