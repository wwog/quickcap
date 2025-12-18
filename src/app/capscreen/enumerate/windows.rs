use crate::app::capscreen::enumerate::{WindowInfo, structs::Rect};
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, RECT},
        Graphics::Dwm::{DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute},
        UI::WindowsAndMessaging::{
            self, GetSystemMetrics, GetWindowInfo, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, WINDOWINFO, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW
        },
    },
    core::BOOL,
};

pub fn enumerate_windows() -> Option<Vec<WindowInfo>> {
    let mut window_infos: Vec<WindowInfo> = vec![];
    // 修正 EnumWindows 的 LPARAM 构造方式，确保传递的是 isize
    let window_infos_ptr = &mut window_infos as *mut _ as isize;
    _ = unsafe {
        WindowsAndMessaging::EnumWindows(Some(enum_window_callback), LPARAM(window_infos_ptr))
    };

    //截屏的全尺寸窗口画布是基于虚拟桌面原点,所以需要将实际坐标转换为基于虚拟桌面原点的坐标
    let (v_x,v_y) = unsafe {
        (GetSystemMetrics(SM_XVIRTUALSCREEN), GetSystemMetrics(SM_YVIRTUALSCREEN))
    };

    if v_x != 0 || v_y != 0 {
        for window_info in window_infos.iter_mut() {
            window_info.bounds.x -= v_x as f64;
            window_info.bounds.y -= v_y as f64;
        }
    }
    Some(window_infos)
}
extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let is_visible: bool = WindowsAndMessaging::IsWindowVisible(hwnd).into();
        if !is_visible {
            return true.into();
        }
        let mut window_info = WINDOWINFO {
            cbSize: core::mem::size_of::<WINDOWINFO>() as u32,
            ..Default::default()
        };

        if let Err(e) = GetWindowInfo(hwnd, &mut window_info) {
            println!("GetWindowInfo failed,HWND is {hwnd:?} error: {:?}", e);
            return true.into();
        }

        if window_info.rcWindow.left >= window_info.rcWindow.right
            || window_info.rcWindow.top >= window_info.rcWindow.bottom
        {
            return true.into();
        }

        let mut cloaked: u32 = 0;
        if let Err(e) = DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut _, 4) {
            println!("DwmGetWindowAttribute failed,HWND is {hwnd:?} error: {:?}", e);
            return true.into();
        }
        if cloaked != 0 {
            return true.into();
        }
        if (window_info.dwExStyle.0 & WS_EX_TOOLWINDOW.0 != 0)
            || (window_info.dwExStyle.0 & WS_EX_NOACTIVATE.0 != 0)
        {
            return true.into();
        }
        let mut window_text_buf = [0u16; 512];
        let len = WindowsAndMessaging::GetWindowTextW(hwnd, &mut window_text_buf);
        let window_text = String::from_utf16_lossy(&window_text_buf[..len as usize]);

        if window_text == "Program Manager" {
            return true.into();
        }
        let mut visual_rect = RECT::default();
        if let Err(e) = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut visual_rect as *mut _ as *mut _,
            core::mem::size_of::<RECT>() as u32,
        ) {
            println!("DwmGetWindowAttribute failed,HWND is {hwnd:?} error: {:?}", e);
            return true.into();
        }

        let window_infos = lparam.0 as *mut Vec<WindowInfo>;
        if let Some(window_infos) = window_infos.as_mut() {
            window_infos.push(WindowInfo {
                name: window_text,
                //todo:转换逻辑分辨率
                bounds: Rect {
                    x: visual_rect.left as f64,
                    y: visual_rect.top as f64,
                    width: (visual_rect.right - visual_rect.left) as f64,
                    height: (visual_rect.bottom - visual_rect.top) as f64,
                },
            });
        }
        true.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_windows() {
        let windows = enumerate_windows();
        println!("windows: {:#?}", windows);
    }
}
