use objc2_core_foundation::{
    CFArray, CFDictionary, CFNumber, CFRetained, CFShow, CFString, CFType,
};
use objc2_core_graphics::{
    CGWindowListCopyWindowInfo, CGWindowListOption, kCGNullWindowID, kCGWindowBounds, kCGWindowName,
};

use crate::app::capscreen::enumerate::structs::Rect;

use super::structs::WindowInfo;

unsafe fn get_number(dict: &CFDictionary<CFString, CFType>, key: &'static str) -> Option<f64> {
    let key = CFString::from_static_str(key);

    let value = dict.get(&key)?;
    let number = value.downcast_ref::<CFNumber>()?;
    number.as_f64()
}

unsafe fn extract_bounds(window: &CFDictionary<CFString, CFType>) -> Option<Rect> {
    unsafe {
        let bounds_value = window.get(&kCGWindowBounds)?;
        let raw_dict = bounds_value.downcast_ref::<CFDictionary>()?;

        let bounds_dict: &CFDictionary<CFString, CFType> = raw_dict.cast_unchecked();

        let x = get_number(bounds_dict, "X")?;
        let y = get_number(bounds_dict, "Y")?;
        let width = get_number(bounds_dict, "Width")?;
        let height = get_number(bounds_dict, "Height")?;

        Some(Rect {
            x,
            y,
            width,
            height,
        })
    }
}

pub fn enumerate_windows(display_id: u32) -> Vec<WindowInfo> {
    let mut window_infos: Vec<WindowInfo> = vec![];
    unsafe {
        let options =
            CGWindowListOption::OptionOnScreenOnly | CGWindowListOption::ExcludeDesktopElements;
        let Some(raw_windows) = CGWindowListCopyWindowInfo(options, kCGNullWindowID) else {
            log::warn!(
                "CGWindowListCopyWindowInfo returned None for display {}",
                display_id
            );
            return window_infos;
        };

        let windows: CFRetained<CFArray<CFDictionary<CFString, CFType>>> =
            CFRetained::cast_unchecked(raw_windows);

        for window in windows.iter() {
            let name: String = if let Some(value) = window.get(&kCGWindowName) {
                if let Some(cf_str) = value.downcast_ref::<CFString>() {
                    cf_str.to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            let bounds = extract_bounds(&window);
            if let Some(bounds) = bounds {
                window_infos.push(WindowInfo { name, bounds });
            } else {
                CFShow(Some(&window));
                log::warn!("Failed to extract bounds for window: {}", name);
                continue;
            }
        }

        window_infos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_windows() {
        let window_infos = enumerate_windows(1);
        println!("window_infos: {:#?}", window_infos);
    }
}
