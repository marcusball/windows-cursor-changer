
use crate::Result;
use std::mem;
use std::ptr::null_mut;
use winapi::shared::minwindef::DWORD;
use winapi::shared::windef::POINT;

/// Wrapper around the winapi POINT type.
pub struct CursorPosition(POINT);

impl CursorPosition {
    /// Try to read the current position of the user's cursor.
    pub fn try_read() -> Option<Self> {
        use winapi::um::winuser::GetCursorPos;
        unsafe {
            let mut point: POINT = mem::uninitialized();

            if GetCursorPos(&mut point) == 0 {
                return None;
            }

            Some(CursorPosition(point))
        }
    }
}

/// Represents the Prcoess under the user's cursor.
/// Wrapper around the winapi process_id of the application under the cursor.
pub struct Process {
    process_id: DWORD,
}

impl Process {
    /// Find the Process of the window at the `CursorPostion`.
    pub fn from_position(position: CursorPosition) -> Option<Self> {
        use winapi::um::winuser::{GetWindowThreadProcessId, WindowFromPoint};

        unsafe {
            // Get the window identifier that lies under this point.
            let window = WindowFromPoint(position.0);

            if window.is_null() {
                return None;
            }

            // Get the ID of the process from the window under the cursor.
            let mut process_id: DWORD = mem::uninitialized();
            GetWindowThreadProcessId(window, &mut process_id);

            Some(Process { process_id })
        }
    }

    /// Get the full path of the executable corresponding to this Process.
    pub fn executable_path(&self) -> Result<String> {
        use winapi::shared::minwindef::MAX_PATH;
        use winapi::shared::ntdef::HANDLE;

        use winapi::um::handleapi::CloseHandle;
        use winapi::um::processthreadsapi::OpenProcess;
        use winapi::um::psapi::GetModuleFileNameExW;
        use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

        // We will read the executable path beneath the cursor into this vec.
        let mut executable_name = Vec::with_capacity(MAX_PATH);

        unsafe {
            // In order to use `GetModuleFileNameExW` We need to get a handle to the process with these access rights.
            // See: https://docs.microsoft.com/en-us/windows/desktop/api/psapi/nf-psapi-getmodulefilenameexw
            let process_handle: HANDLE = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                0,
                self.process_id,
            );

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


        Ok(String::from_utf16(&executable_name)?)
    }
}
