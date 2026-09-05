#[tauri::command]
pub async fn trim_memory() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };
        use windows_sys::Win32::System::ProcessStatus::K32EmptyWorkingSet;
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcess, GetCurrentProcessId, OpenProcess, SetProcessWorkingSetSize,
            PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
        };

        unsafe {
            // Empty host process working set
            let current = GetCurrentProcess();
            let _ = SetProcessWorkingSetSize(current, usize::MAX, usize::MAX);
            let _ = K32EmptyWorkingSet(current);

            // Empty all child processes (WebView2 browser, GPU, renderer, utility)
            let my_pid = GetCurrentProcessId();
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot != INVALID_HANDLE_VALUE && !snapshot.is_null() {
                let mut entry: PROCESSENTRY32W = std::mem::zeroed();
                entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

                if Process32FirstW(snapshot, &mut entry) != 0 {
                    loop {
                        if entry.th32ParentProcessID == my_pid {
                            let child_pid = entry.th32ProcessID;
                            let child_handle = OpenProcess(
                                PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA,
                                0,
                                child_pid,
                            );
                            if !child_handle.is_null() && child_handle != INVALID_HANDLE_VALUE {
                                let _ = SetProcessWorkingSetSize(
                                    child_handle,
                                    usize::MAX,
                                    usize::MAX,
                                );
                                let _ = K32EmptyWorkingSet(child_handle);
                                CloseHandle(child_handle);
                            }
                        }
                        if Process32NextW(snapshot, &mut entry) == 0 {
                            break;
                        }
                    }
                }
                CloseHandle(snapshot);
            }
        }
    }
    Ok(())
}
