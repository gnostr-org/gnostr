#![cfg(target_os = "windows")]

// Import necessary items from the winapi crate
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::shared::ntdef::NULL;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
use winapi::um::winnt::{HANDLE, PROCESS_TERMINATE};

// The function is public so it can be called from the main function.
pub fn kill_process_by_pid(process_id: DWORD) -> Result<(), String> {
    // Operations that call FFI functions are inherently unsafe in Rust.
    unsafe {
        // 1. Open the process to get a handle with termination rights.
        let process_handle: HANDLE = OpenProcess(PROCESS_TERMINATE, FALSE, process_id);

        // 2. Check if OpenProcess failed.
        if process_handle.is_null() {
            let error_code = GetLastError();
            return Err(format!(
                "Failed to open process (PID: {}). Win32 Error Code: {}",
                process_id, error_code
            ));
        }

        // 3. Terminate the process.
        let terminated: i32 = TerminateProcess(process_handle, 1);

        // 4. Close the handle.
        CloseHandle(process_handle);

        // 5. Check if TerminateProcess succeeded.
        if terminated != 0 {
            println!(
                "✅ Process with PID {} successfully terminated.",
                process_id
            );
            Ok(())
        } else {
            let error_code = GetLastError();
            Err(format!(
                "Failed to terminate process (PID: {}). Win32 Error Code: {}",
                process_id, error_code
            ))
        }
    }
}
