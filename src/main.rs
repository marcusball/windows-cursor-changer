// Let's put this so that it won't open console
#![windows_subsystem = "windows"]

#[cfg(windows)]
extern crate winapi;
// https://docs.rs/winapi/*/x86_64-pc-windows-msvc/winapi/um/libloaderapi/index.html?search=winuser

use std::ffi::OsStr;
use std::io::Error;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;


use self::winapi::shared::windef::HWND;
use self::winapi::um::libloaderapi::GetModuleHandleW;
use self::winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
    TranslateMessage,

};
use self::winapi::um::winuser::{
    CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, MSG, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
};
use winapi::shared::minwindef::{DWORD, UINT};
use winapi::shared::windef::HCURSOR;

// ----------------------------------------------------

// We have to encode text to wide format for Windows
#[cfg(windows)]
fn win32_string(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

// Window struct
#[cfg(windows)]
struct Window {
    handle: HWND,
}

#[cfg(windows)]
fn get_cursor() -> HCURSOR {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;

    use winapi::um::winuser::{LoadCursorFromFileW, SetCursor};

    let path = "D:\\Users\\Marcus\\Source\\SmallProjects\\windows-cursor-changer\\sissy.ani";

    let wide: Vec<u16> = OsStr::new(&path).encode_wide().chain(once(0)).collect();

    unsafe { LoadCursorFromFileW(wide.as_ptr()) }
}

/// Set all system cursors to a specific cursor.
///
/// See: https://stackoverflow.com/a/55098397/451726
#[cfg(windows)]
fn set_system_cursor(cursor: HCURSOR) {
    use winapi::um::winuser::SetSystemCursor;

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
        unsafe { SetSystemCursor(copied, cursor_id) };
    }
}

/// Restore original system cursors
///
/// See: https://stackoverflow.com/a/55098397/451726
#[cfg(windows)]
fn restore_original_cursors() {
    use winapi::um::winuser::SystemParametersInfoW;

    const SPI_SETCURSORS: UINT = 0x0057;

    unsafe { SystemParametersInfoW(SPI_SETCURSORS, 0, null_mut(), 0) };
}

/// Copy a cursor
/// See: https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-copycursor
#[cfg(windows)]
fn copy_cursor(cursor: HCURSOR) -> HCURSOR {
    use winapi::shared::windef::HICON;
    use winapi::um::winuser::CopyIcon;

    unsafe { CopyIcon(cursor as HICON) }
}

// Create window function
#[cfg(windows)]
fn create_window(name: &str, title: &str) -> Result<Window, Error> {
    let name = win32_string(name);
    let title = win32_string(title);

    let cursor = get_cursor();

    set_system_cursor(cursor);

    unsafe {
        // Create handle instance that will call GetModuleHandleW, which grabs the instance handle of WNDCLASSW (check third parameter)
        let hinstance = GetModuleHandleW(null_mut());

        // Create "class" for window, using WNDCLASSW struct (different from Window our struct)
        let wnd_class = WNDCLASSW {
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW, // Style
            lpfnWndProc: Some(DefWindowProcW), // The callbackfunction for any window event that can occur in our window!!! Here you could react to events like WM_SIZE or WM_QUIT.
            hInstance: hinstance, // The instance handle for our application which we can retrieve by calling GetModuleHandleW.
            lpszClassName: name.as_ptr(), // Our class name which needs to be a UTF-16 string (defined earlier before unsafe). as_ptr() (Rust's own function) returns a raw pointer to the slice's buffer
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: null_mut(),
            hCursor: cursor,
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
        };

        // We have to register this class for Windows to use
        RegisterClassW(&wnd_class);

        // More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms632680(v=vs.85).aspx
        // Create a window based on registered class
        let handle = CreateWindowExW(
            0,                                // dwExStyle
            name.as_ptr(), // lpClassName, name of the class that we want to use for this window, which will be the same that we have registered before.
            title.as_ptr(), // lpWindowName
            WS_OVERLAPPEDWINDOW | WS_VISIBLE, // dwStyle
            CW_USEDEFAULT, // Int x
            CW_USEDEFAULT, // Int y
            CW_USEDEFAULT, // Int nWidth
            CW_USEDEFAULT, // Int nHeight
            null_mut(),    // hWndParent
            null_mut(),    // hMenu
            hinstance,     // hInstance
            null_mut(),
        ); // lpParam

        if handle.is_null() {
            Err(Error::last_os_error())
        } else {
            Ok(Window { handle })
        }
    }
}

#[cfg(windows)]
// Create message handling function with which to link to hook window to Windows messaging system
// More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms644927(v=vs.85).aspx
fn handle_message(window: &mut Window) -> bool {
    unsafe {
        let mut message: MSG = mem::uninitialized();

        // Get message from message queue with GetMessageW
        if GetMessageW(&mut message as *mut MSG, window.handle, 0, 0) > 0 {
            println!("{}", message.message);

            TranslateMessage(&message as *const MSG); // Translate message into something meaningful with TranslateMessage
            DispatchMessageW(&message as *const MSG); // Dispatch message with DispatchMessageW

            true
        } else {
            false
        }
    }
}
#[cfg(windows)]
fn main() {
    let mut window = create_window("my_window", "Portfolio manager pro").unwrap();

    loop {
        if !handle_message(&mut window) {

            restore_original_cursors();
            break;
        }
    }
}