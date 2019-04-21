use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::shared::minwindef::{DWORD, UINT};
use winapi::shared::windef::HCURSOR;

use crate::CursorHandle;


// We have to encode text to wide format for Windows
#[cfg(windows)]
fn win32_string(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

#[cfg(windows)]
pub fn get_cursor(path: &str) -> CursorHandle {
    use winapi::um::winuser::{
        LoadImageW, IMAGE_CURSOR, LR_DEFAULTCOLOR, LR_LOADFROMFILE,
    };

    let wide: Vec<u16> = win32_string(path);

    // unsafe { LoadCursorFromFileW(wide.as_ptr()) }
    let c = unsafe {
        LoadImageW(
            null_mut(),
            wide.as_ptr(),
            IMAGE_CURSOR,
            0,
            0,
            LR_DEFAULTCOLOR | LR_LOADFROMFILE,
        ) as HCURSOR
    };

    CursorHandle(c)
}

/// Set all system cursors to a specific cursor.
///
/// See: https://stackoverflow.com/a/55098397/451726
#[cfg(windows)]
pub fn set_system_cursor(cursor: &CursorHandle) {
    use winapi::um::winuser::SetSystemCursor;

    // See: https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-setsystemcursor
    let cursor_ids: Vec<DWORD> = vec![
        32650, // OCR_APPSTARTING
        32512, // OCR_NORMAL
        32515, // OCR_CROSS
        32649, // OCR_HAND
        32651, // OCR_HELP
        32513, // OCR_IBEAM
        32648, // OCR_NO
        32646, // OCR_SIZEALL
        32643, // OCR_SIZENESW
        32645, // OCR_SIZENS
        32642, // OCR_SIZENWSE
        32644, // OCR_SIZEWE
        32516, // OCR_UP
        32514, // OCR_WAIT
    ];

    for cursor_id in cursor_ids {
        let copied = copy_cursor(cursor);
        unsafe { SetSystemCursor(copied.0, cursor_id) };
    }
}

/// Restore original system cursors
///
/// See: https://stackoverflow.com/a/55098397/451726
#[cfg(windows)]
pub fn restore_original_cursors() {
    use winapi::um::winuser::SystemParametersInfoW;

    const SPI_SETCURSORS: UINT = 0x0057;

    unsafe { SystemParametersInfoW(SPI_SETCURSORS, 0, null_mut(), 0) };
}

/// Copy a cursor
/// See: https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-copycursor
#[cfg(windows)]
fn copy_cursor(cursor: &CursorHandle) -> CursorHandle {
    use winapi::shared::windef::HICON;
    use winapi::um::winuser::CopyIcon;

    let copied = unsafe { CopyIcon(cursor.0 as HICON) };

    CursorHandle(copied)
}