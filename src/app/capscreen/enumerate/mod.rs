mod structs;

#[cfg(target_os = "macos")]
mod macos;

pub use structs::WindowInfo;

pub fn enumerate_windows(display_id: u32) -> Vec<WindowInfo> {
    #[cfg(target_os = "macos")]
    {
        macos::enumerate_windows_cg(display_id)
    }
    #[cfg(not(target_os = "macos"))]
    {
        vec![]
    }
}