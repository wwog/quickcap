mod structs;

#[cfg(target_os = "macos")]
mod macos;


pub fn enumerate_windows(display_id: u32) {
    #[cfg(target_os = "macos")]
    {
        macos::enumerate_windows(display_id);
    }
    #[cfg(not(target_os = "macos"))]
    {
        vec![]
    }
}