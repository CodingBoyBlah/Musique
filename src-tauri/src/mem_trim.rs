#[cfg(target_os = "windows")]
pub fn start_memory_trimmer() {
    std::thread::Builder::new()
        .name("memory-trimmer".into())
        .spawn(|| {
            // Initial wait for app launch & UI hydration
            std::thread::sleep(std::time::Duration::from_secs(4));
            trim_all();
            std::thread::sleep(std::time::Duration::from_secs(10));
            trim_all();

            loop {
                std::thread::sleep(std::time::Duration::from_secs(45));
                trim_all();
            }
        })
        .ok();
}

#[cfg(target_os = "windows")]
pub fn trim_all() {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, GetCurrentProcessId, OpenProcess, SetProcessWorkingSetSize,
        PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
    };

    unsafe {
        let current_h = GetCurrentProcess();
        SetProcessWorkingSetSize(current_h, usize::MAX, usize::MAX);

        let current_pid = GetCurrentProcessId();

        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == -1 as isize as _ {
            return;
        }

        let mut entry: PROCESSENTRY32 = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let mut all_entries = Vec::with_capacity(256);

        if Process32First(snap, &mut entry) != 0 {
            loop {
                all_entries.push((entry.th32ProcessID, entry.th32ParentProcessID));
                if Process32Next(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);

        let mut target_pids = vec![current_pid];
        let mut added = true;
        while added {
            added = false;
            for &(pid, ppid) in &all_entries {
                if target_pids.contains(&ppid) && !target_pids.contains(&pid) {
                    target_pids.push(pid);
                    added = true;
                }
            }
        }

        for &pid in &target_pids {
            let h = OpenProcess(
                PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION,
                0,
                pid,
            );
            if h != 0 as _ {
                SetProcessWorkingSetSize(h, usize::MAX, usize::MAX);
                CloseHandle(h);
            }
        }
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn trim_memory() {
    trim_all();
}

#[cfg(not(target_os = "windows"))]
pub fn start_memory_trimmer() {}

#[cfg(not(target_os = "windows"))]
pub fn trim_all() {}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn trim_memory() {}
