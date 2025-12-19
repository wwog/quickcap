use rayon::prelude::*;
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
        DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HGDIOBJ, ReleaseDC, SRCCOPY,
        SelectObject,
    },
    UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    },
};

use crate::capscreen::{CaptureError, Frame};

pub fn capscreen() -> Result<Frame, CaptureError> {
    unsafe {
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        let h_screen_dc = GetDC(Some(HWND(std::ptr::null_mut())));
        let h_memory_dc = CreateCompatibleDC(Some(h_screen_dc));

        let h_bitmap = CreateCompatibleBitmap(h_screen_dc, width, height);

        let old_obj = SelectObject(h_memory_dc, HGDIOBJ(h_bitmap.0));

        if let Err(e) = BitBlt(
            h_memory_dc,
            0,
            0,
            width,
            height,
            Some(h_screen_dc),
            x,
            y,
            SRCCOPY,
        ) {
            SelectObject(h_memory_dc, old_obj);
            _ = DeleteObject(HGDIOBJ(h_bitmap.0));
            _ = DeleteDC(h_memory_dc);
            ReleaseDC(None, h_screen_dc);
            println!("BitBlt failed, error: {:?}", e);
            return Err(CaptureError::FailedToCaptureImage);
        }

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32, // 4 字节每像素 (BGRA)
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut data = vec![0u8; (width * height * 4) as usize];
        let lines_copied = GetDIBits(
            h_memory_dc,
            h_bitmap,
            0,
            height as u32,
            Some(data.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        data.par_chunks_exact_mut(4).for_each(|pixel| {
            pixel.swap(0, 2);
            pixel[3] = 255;
        });

        SelectObject(h_memory_dc, old_obj);
        _ = DeleteObject(HGDIOBJ(h_bitmap.0));
        _ = DeleteDC(h_memory_dc);
        ReleaseDC(None, h_screen_dc);

        if lines_copied == 0 {
            println!("GetDIBits failed, lines_copied: {}", lines_copied);
            return Err(CaptureError::FailedToCaptureImage);
        }
        Ok(Frame {
            data,
            width: width as u32,
            height: height as u32,
        })
    }
}
