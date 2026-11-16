//! Program icons for the process list. Read from the exe file only, never from a running process,
//! so the UI still touches no hardware and needs no extra rights.

use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits,
    GetObjectW, HBITMAP, HDC, ReleaseDC,
};
use windows_sys::Win32::UI::Shell::ExtractIconExW;
use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

/// The exe's first large icon (32×32 on most systems) as RGBA, rows top first. Empty when the file has
/// no icon or cannot be read; the UI then draws a placeholder.
#[tauri::command]
pub async fn exe_icon(path: String) -> tauri::ipc::Response {
    // Reading PE resources blocks; a page asks for many at once, so keep them off the async workers.
    let bytes = tauri::async_runtime::spawn_blocking(move || rgba(&path).unwrap_or_default())
        .await
        .unwrap_or_default();
    tauri::ipc::Response::new(bytes)
}

fn rgba(path: &str) -> Option<Vec<u8>> {
    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    let mut icon: HICON = std::ptr::null_mut();
    // SAFETY: `wide` is NUL-terminated; one large icon is asked for and `icon` receives it.
    if unsafe { ExtractIconExW(wide.as_ptr(), 0, &mut icon, std::ptr::null_mut(), 1) } == 0
        || icon.is_null()
    {
        return None;
    }
    let out = pixels(icon);
    // SAFETY: `icon` came from ExtractIconExW and is destroyed once.
    unsafe { DestroyIcon(icon) };
    out
}

fn pixels(icon: HICON) -> Option<Vec<u8>> {
    // SAFETY: ICONINFO is plain data; GetIconInfo fills it and hands over two bitmaps we delete below.
    let mut info: ICONINFO = unsafe { std::mem::zeroed() };
    if unsafe { GetIconInfo(icon, &mut info) } == 0 {
        return None;
    }
    // SAFETY: a null window gives the screen DC, released below.
    let dc = unsafe { GetDC(std::ptr::null_mut()) };
    let out = (|| {
        if info.hbmColor.is_null() {
            return None; // A monochrome icon: rare for programs, shown as the placeholder.
        }
        let (w, h) = size(info.hbmColor)?;
        let mut px = bits(dc, info.hbmColor, w, h)?;
        // Old icons carry no alpha: the mask says which pixels are see-through (white = transparent).
        if px.chunks(4).all(|p| p[3] == 0) {
            let mask = bits(dc, info.hbmMask, w, h)?;
            for (p, m) in px.chunks_mut(4).zip(mask.chunks(4)) {
                p[3] = if m[0] == 0 { 255 } else { 0 };
            }
        }
        for p in px.chunks_mut(4) {
            p.swap(0, 2); // BGRA to RGBA
        }
        Some(px)
    })();
    // SAFETY: each handle is released once.
    unsafe {
        ReleaseDC(std::ptr::null_mut(), dc);
        DeleteObject(info.hbmColor);
        DeleteObject(info.hbmMask);
    }
    out
}

fn size(bmp: HBITMAP) -> Option<(i32, i32)> {
    // SAFETY: BITMAP is plain data and the size passed matches it.
    let mut b: BITMAP = unsafe { std::mem::zeroed() };
    let n = unsafe { GetObjectW(bmp, size_of::<BITMAP>() as i32, &mut b as *mut _ as *mut _) };
    (n != 0 && b.bmWidth > 0 && b.bmWidth <= 256 && b.bmWidth == b.bmHeight)
        .then_some((b.bmWidth, b.bmHeight))
}

/// The bitmap as 32-bit BGRA, top row first.
fn bits(dc: HDC, bmp: HBITMAP, w: i32, h: i32) -> Option<Vec<u8>> {
    // SAFETY: BITMAPINFO is plain data; a negative height asks for top-down rows.
    let mut bi: BITMAPINFO = unsafe { std::mem::zeroed() };
    bi.bmiHeader = BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: w,
        biHeight: -h,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        ..unsafe { std::mem::zeroed() }
    };
    let mut px = vec![0u8; (w * h * 4) as usize];
    // SAFETY: `px` holds w*h 32-bit pixels, as `bi` describes.
    let lines = unsafe {
        GetDIBits(
            dc,
            bmp,
            0,
            h as u32,
            px.as_mut_ptr().cast(),
            &mut bi,
            DIB_RGB_COLORS,
        )
    };
    (lines == h).then_some(px)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_system_exe_gives_a_square_rgba_icon_and_a_missing_file_gives_none() {
        let px = super::rgba(r"C:\Windows\explorer.exe").expect("explorer.exe has an icon");
        let side = ((px.len() / 4) as f64).sqrt() as usize;
        assert_eq!(side * side * 4, px.len());
        assert!(px.chunks(4).any(|p| p[3] > 0), "some pixels are visible");
        assert!(super::rgba(r"C:\no\such\file.exe").is_none());
    }
}
