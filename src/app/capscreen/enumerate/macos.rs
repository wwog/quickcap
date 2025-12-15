use objc2_core_foundation::{CFArray, CFDictionary, CFRetained, CFShow};
use objc2_core_graphics::{CGWindowListCopyWindowInfo, CGWindowListOption, kCGNullWindowID};

use super::structs::WindowInfo;

pub fn enumerate_windows(display_id: u32) -> Vec<WindowInfo> {
    unsafe {
        let options =
            CGWindowListOption::OptionOnScreenOnly | CGWindowListOption::ExcludeDesktopElements;
        let Some(raw_windows) = CGWindowListCopyWindowInfo(options, kCGNullWindowID) else {
            log::warn!(
                "CGWindowListCopyWindowInfo returned None for display {}",
                display_id
            );
            return vec![];
        };

        let windows: CFRetained<CFArray<CFDictionary>> = CFRetained::cast_unchecked(raw_windows);

        log::info!(
            "枚举显示器 {} 的窗口，总计 {} 个",
            display_id,
            windows.len()
        );

        for window in windows.iter() {
            let dict: &CFDictionary = window.as_opaque();
            CFShow(Some(dict));
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_windows() {
        enumerate_windows(1);
    }
}
