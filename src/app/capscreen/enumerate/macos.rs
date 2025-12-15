use objc2_core_foundation::{
    CFArray, CFDictionary, CFNumber, CFRetained, CFShow, CFString, CFType,
};
use objc2_core_graphics::{
    CGWindowListCopyWindowInfo, CGWindowListOption, kCGNullWindowID, kCGWindowBounds, kCGWindowName,
};
use screencapturekit::prelude::SCShareableContent;
use crate::app::capscreen::enumerate::structs::Rect;
use super::structs::WindowInfo;

pub fn enumerate_windows(display_id: u32) -> Option<Vec<WindowInfo>> {
    let content = SCShareableContent::with_options()
        .on_screen_windows_only(true)
        .get()
        .ok()?;

    let display_info = content
        .displays()
        .into_iter()
        .find(|display| display.display_id() == display_id)?;
    let frame = display_info.frame();
    let display_left = frame.origin().x;
    let display_right = frame.origin().x + frame.size().width;
    let display_top = frame.origin().y;
    let display_bottom = frame.origin().y + frame.size().height;

    let windows = content.windows();
    let mut window_infos = vec![];
    for window in windows {
        if window.window_layer() != 0 {
            continue;
        }
        if window.frame().is_empty() || window.frame().is_null() {
            continue;
        }

        //因为窗口可能会溢出当前显示器，所以不能用桌面frame包含来判断是否在当前显示器上
        //这里应该是只要有交集就认为在当前显示器上
        let window_frame = window.frame();
        let window_origin = window_frame.origin();
        let window_size = window_frame.size();
        let window_left = window_origin.x;
        let window_right = window_origin.x + window_size.width;
        let window_top = window_origin.y;
        let window_bottom = window_origin.y + window_size.height;
        
        if window_right < display_left || window_left > display_right || window_bottom < display_top || window_top > display_bottom {
            continue;
        }

        window_infos.push(WindowInfo {
            name: window.title().unwrap_or_default(),
            bounds: Rect {
                x: window_left,
                y: window_top,
                width: window_size.width,
                height: window_size.height,
            },
        });
    }
    Some(window_infos)
}

/// 如果后续兼容12.3以上，可以考虑使用这个函数，暂时不考虑使用
#[allow(dead_code)]
pub fn enumerate_windows_cg(display_id: u32) -> Vec<WindowInfo> {
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_windows() {
        let window_infos = enumerate_windows(1);
        println!("window_infos: {:#?}", window_infos);
    }

    // #[test]
    // fn test_enumerate_windows_cg() {
    //     let window_infos = enumerate_windows_cg(1);
    //     // println!("window_infos: {:#?}", window_infos);
    // }
}
