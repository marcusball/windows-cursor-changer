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
mod info;
mod window;

use error::Error;

use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

use std::sync::{Arc, Mutex};
use std::thread;

use winapi::shared::minwindef::{DWORD, UINT};
use winapi::shared::windef::HCURSOR;

use std::collections::HashMap;
use std::path::Path;

pub type Result<T> = std::result::Result<T, error::Error>;

/// Wrapper around the HCURSOR winapi type
#[derive(Debug)]
pub struct CursorHandle(HCURSOR);

type CursorId = u32;

/// Cursor struct
#[derive(Debug)]
pub struct Cursor {
    /// A unique integer identifer for this Cursor
    id: CursorId,

    /// Unique identifer for this Cursor
    name: String,

    /// Path to this Cursor's .cur or .ani file.
    path: String,

    /// Handle to the Cursor loaded by Windows.
    handle: CursorHandle,
}

impl Cursor {
    /// Create a cursor and load it to acquire a usable handle to it.
    pub fn new(id: CursorId, name: String, path: String) -> Cursor {
        Cursor {
            id: id,
            name: name,
            handle: get_cursor(&path),
            path: path,
        }
    }

    /// Get the Path to this Cursor's image file.
    pub fn path(&self) -> &Path {
        Path::new(&self.path)
    }
}

#[derive(Debug)]
pub struct Application {
    /// The ID of the Cursor to use when the mouse is over this Application.
    cursor_id: CursorId,

    /// The path (or partial path) that will be used to identify this Application.
    /// Comparison will be done by checking if the full path of the executable
    /// under the cursor `ends_with` this `path`, so this may be a full absolute path,
    /// or just the exe name or partial path.
    path: String,
}

impl Application {
    pub fn new(cursor: CursorId, path: String) -> Self {
        Application {
            cursor_id: cursor,
            path: path,
        }
    }
}

#[derive(Debug)]
pub struct CursorChanger {
    /// Lookup map to associate the cursor `name` with a unique numerical CursorId
    cursor_ids: HashMap<String, CursorId>,

    /// Map that associates a cursor's unique `name` with the cursor itself.
    cursors: HashMap<CursorId, Cursor>,

    /// Monitored applications
    applications: Vec<Application>,

    /// Run-time state: Which custom cursor is currently active, or is it the Windows system cursor (`None`).
    active_cursor: Option<CursorId>,
}


impl CursorChanger {
    fn from_config(config: config::Config) -> Result<CursorChanger> {
        let mut changer = CursorChanger::new();
        changer.add_cursors(config.cursor)?;
        changer.add_applications(config.application)?;

        Ok(changer)
    }

    fn new() -> CursorChanger {
        CursorChanger {
            cursor_ids: HashMap::new(),
            cursors: HashMap::new(),
            applications: Vec::new(),
            active_cursor: None,
        }
    }

    pub fn is_custom_cursor_active(&self) -> bool {
        self.active_cursor.is_some()
    }

    /// Copy configuration details for Cursors into the configuration `cursors` map.
    fn add_cursors(&mut self, cursors: Vec<config::Cursor>) -> Result<()> {
        // Find the max existing ID, or default to zero if there are no existing IDs.
        let max_id = self.cursor_ids.values().max().unwrap_or(&0);

        // This will keep track of the next ID to assign to each new cursor.
        let mut next_id = max_id + 1;

        for config_cursor in cursors.into_iter() {
            let cursor = Cursor::new(next_id, config_cursor.name, config_cursor.path);

            // Check to make sure there isn't already a cursor using this unique `name`.
            if self.cursor_ids.contains_key(&cursor.name) {
                return Err(error::Error::DuplicateCursorName {
                    name: cursor.name.clone(),
                });
            }

            // Check to make sure the file specified by the `path` exists.
            if !cursor.path().exists() {
                return Err(error::Error::MissingCursorFileError {
                    name: cursor.name.clone(),
                    path: cursor.path.clone(),
                });
            }

            let _existing = self.cursor_ids.insert(cursor.name.clone(), cursor.id);

            // insert returns the value that was replaced if the key already exists
            assert_eq!(None, _existing);

            // Insert it into the map for easy lookup by `name`.
            self.cursors.insert(cursor.id, cursor);

            // Increment the new Cursor ID.
            next_id += 1;
        }

        Ok(())
    }

    /// Insert tracked applications into the Config `applications` map.
    /// This will check to make sure that there exists a Cursor identified
    /// by the Application's `cursor` name.
    fn add_applications(&mut self, applications: Vec<config::Application>) -> Result<()> {
        for config_application in applications.into_iter() {
            // Try to find the ID of the cursor, given the cursor's name.
            let cursor_id = match self.cursor_ids.get(&config_application.cursor) {
                // If we found it, use that ID.
                Some(id) => id,
                // If the name did not return an ID, quit with an error.
                None => {
                    return Err(error::Error::MissingCursorNameError {
                        name: config_application.cursor.clone(),
                    });
                }
            };

            let application = Application::new(*cursor_id, config_application.path);

            self.applications.push(application);
        }

        Ok(())
    }

    pub fn tick(&mut self) {
        // Temp: just always use the first cursor
        let cursor = self.cursors.iter().nth(0).unwrap().1;

        match get_process_under_cursor() {
            Ok(Some(name)) => {
                if name.ends_with("powershell.exe") {
                    if !self.is_custom_cursor_active() {
                        println!("setting cursor");
                        set_system_cursor(&cursor.handle);
                        self.active_cursor = Some(cursor.id);
                    }
                } else {
                    if self.is_custom_cursor_active() {
                        println!("restoring cursor");
                        restore_original_cursors();
                        self.active_cursor = None;
                    }
                }
            }
            Ok(None) => {}
            Err(e) => println!("ERROR: {}", e),
        }
    }
}

// ----------------------------------------------------

// We have to encode text to wide format for Windows
#[cfg(windows)]
fn win32_string(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

#[cfg(windows)]
fn get_cursor(path: &str) -> CursorHandle {
    use winapi::um::winuser::{
        LoadCursorFromFileW, LoadImageW, IMAGE_CURSOR, LR_DEFAULTCOLOR, LR_LOADFROMFILE,
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
fn set_system_cursor(cursor: &CursorHandle) {
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
fn copy_cursor(cursor: &CursorHandle) -> CursorHandle {
    use winapi::shared::windef::HICON;
    use winapi::um::winuser::CopyIcon;

    let copied = unsafe { CopyIcon(cursor.0 as HICON) };

    CursorHandle(copied)
}

#[cfg(windows)]
fn get_process_under_cursor() -> Result<Option<String>> {
    use info::{CursorPosition, Process};

    // Read the position of the cursor
    CursorPosition::try_read()
        // Get the process that is under the cursor at that position
        .and_then(Process::from_position)
        // Get the full path to that process's executable
        .map(|p| p.executable_path())
        // Convert the Option<Result<_>> type to Result<Option<_>>
        .transpose()
}

#[cfg(windows)]
fn main() {
    let config = config::Config::from_file("cursor.toml").unwrap();


    let exit = Arc::new(Mutex::new(false));

    let thread_exit = Arc::clone(&exit);
    let child = thread::spawn(move || {
        let mut cursor_changer = CursorChanger::from_config(config).unwrap();
        println!("{:?}", cursor_changer);

        let mut should_exit = false;

        while !should_exit {
            cursor_changer.tick();

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