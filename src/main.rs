// Let's put this so that it won't open console
// #![windows_subsystem = "windows"]

extern crate serde;
extern crate toml;
#[cfg(windows)]
extern crate winapi;
#[macro_use]
extern crate serde_derive;


// https://docs.rs/winapi/*/x86_64-pc-windows-msvc/winapi/um/libloaderapi/index.html?search=winuser

mod config;
mod error;
mod window;

use error::Error;

use std::ffi::OsStr;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

use std::sync::{Arc, Mutex};
use std::thread;

use winapi::shared::minwindef::{DWORD, UINT};
use winapi::shared::windef::HCURSOR;


/// Wrapper around the HCURSOR winapi type
pub struct Cursor(HCURSOR);


// ----------------------------------------------------

// We have to encode text to wide format for Windows
#[cfg(windows)]
fn win32_string(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

#[cfg(windows)]
fn get_cursor() -> Cursor {
    use winapi::um::winuser::{
        LoadCursorFromFileW, LoadImageW, IMAGE_CURSOR, LR_DEFAULTCOLOR, LR_LOADFROMFILE,
    };

    let path = "D:\\Users\\Marcus\\Source\\SmallProjects\\windows-cursor-changer\\big.cur";

    let wide: Vec<u16> = win32_string(&path);

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

    Cursor(c)
}

/// Set all system cursors to a specific cursor.
///
/// See: https://stackoverflow.com/a/55098397/451726
#[cfg(windows)]
fn set_system_cursor(cursor: &Cursor) {
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
fn restore_original_cursors() {
    use winapi::um::winuser::SystemParametersInfoW;

    const SPI_SETCURSORS: UINT = 0x0057;

    unsafe { SystemParametersInfoW(SPI_SETCURSORS, 0, null_mut(), 0) };
}

/// Copy a cursor
/// See: https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-copycursor
#[cfg(windows)]
fn copy_cursor(cursor: &Cursor) -> Cursor {
    use winapi::shared::windef::HICON;
    use winapi::um::winuser::CopyIcon;

    let copied = unsafe { CopyIcon(cursor.0 as HICON) };

    Cursor(copied)
}

#[cfg(windows)]
fn get_cursor_pos() -> Result<Option<String>, Error> {
    use winapi::shared::minwindef::MAX_PATH;
    use winapi::shared::ntdef::HANDLE;
    use winapi::shared::windef::POINT;
    #[rustfmt::skip]
    use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    #[rustfmt::skip]
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::handleapi::CloseHandle;
    #[rustfmt::skip]
    use winapi::um::psapi::GetModuleFileNameExW;
    use winapi::um::winuser::{GetCursorPos, GetWindowThreadProcessId, WindowFromPoint};

    // We will read the executable path beneath the cursor into this vec.
    let mut executable_name = Vec::with_capacity(MAX_PATH);

    unsafe {
        let mut point: POINT = mem::uninitialized();

        if GetCursorPos(&mut point) == 0 {
            return Ok(None);
        }

        // Get the window identifier that lies under this point.
        let window = WindowFromPoint(point);

        // Get the ID of the process from the window under the cursor.
        let mut process_id: DWORD = mem::uninitialized();
        GetWindowThreadProcessId(window, &mut process_id);

        // In order to use `GetModuleFileNameExW` We need to get a handle to the process with these access rights.
        // See: https://docs.microsoft.com/en-us/windows/desktop/api/psapi/nf-psapi-getmodulefilenameexw
        let process_handle: HANDLE =
            OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id);

        // Read the file name of the executable from the process, stored into the `executable_name` vec.
        // Returns the length of the returned value.
        let length = GetModuleFileNameExW(
            process_handle,
            null_mut(),
            executable_name.as_mut_ptr(),
            MAX_PATH as u32,
        );

        // Close the process handle.
        CloseHandle(process_handle);

        // Update the length of the vec.
        executable_name.set_len(length as usize);
    }


    Ok(Some(String::from_utf16(&executable_name)?))
}

#[cfg(windows)]
fn main() {
    let config = config::Config::from_file("cursor.toml").unwrap();

    println!("{:?}", config);


    let exit = Arc::new(Mutex::new(false));

    let thread_exit = Arc::clone(&exit);
    let child = thread::spawn(move || {
        let cursor = get_cursor();

        let mut is_custom_cursor = false;

        let mut should_exit = false;

        while !should_exit {
            match get_cursor_pos() {
                Ok(Some(name)) => {
                    if name.ends_with("powershell.exe") {
                        if !is_custom_cursor {
                            println!("setting cursor");
                            set_system_cursor(&cursor);
                            is_custom_cursor = true;
                        }
                    } else {
                        if is_custom_cursor {
                            println!("restoring cursor");
                            restore_original_cursors();
                            is_custom_cursor = false;
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => println!("ERROR: {}", e),
            }

            // read the mutex to see if the thread should quit
            should_exit = *thread_exit.lock().unwrap();
        }

        println!("Exiting gracefully...");
        // some work here

        restore_original_cursors();
    });

    // Create a window
    window::create_window_and_block();

    println!("Notifying thread to exit");

    {
        let mut signal_exit = exit.lock().unwrap();
        *signal_exit = true;
        // Drop the lock so that the thread can read the signal
    }

    // some work here
    let res = child.join();

    //restore_original_cursors();
}