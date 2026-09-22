// Release memory back to the OS - but only when the app is genuinely idle.
//
// This used to empty the working set 4s after launch, again at 14s, and then
// every 45 seconds forever, plus on every focus loss. `SetProcessWorkingSetSize`
// with (-1, -1) does not "free" anything: it evicts every resident page of this
// process AND of every WebView2 child (renderer, GPU, network). The pages are
// still needed, so the moment the user scrolls, clicks or hits play, the CPU
// takes thousands of hard faults pulling them back off disk. On a 45s timer that
// meant the app was re-faulting its entire working set several times a minute
// while in active use, and a plain alt-tab away and back guaranteed a stutter.
// It bought a smaller number in Task Manager and paid for it in latency
// everywhere.
//
// Now the trim happens only when the window has been hidden or minimized
// continuously for a while - i.e. the app is sitting in the tray and nobody is
// looking at it - and only once per idle stretch. Active use is never touched,
// so nothing the user can see or feel has to be paged back in.
#[cfg(target_os = "windows")]
pub fn start_memory_trimmer(app: tauri::AppHandle) {
    use tauri::Manager;

    const POLL: std::time::Duration = std::time::Duration::from_secs(15);
    // how long the window must stay out of sight before we bother
    const IDLE_BEFORE_TRIM: std::time::Duration = std::time::Duration::from_secs(90);

    std::thread::Builder::new()
        .name("memory-trimmer".into())
        .spawn(move || {
            let mut hidden_since: Option<std::time::Instant> = None;
            let mut trimmed_this_idle = false;

            loop {
                std::thread::sleep(POLL);

                let out_of_sight = match app.get_webview_window("main") {
                    Some(w) => {
                        !w.is_visible().unwrap_or(true) || w.is_minimized().unwrap_or(false)
                    }
                    // no window yet (or already torn down): nothing to do
                    None => false,
                };

                if !out_of_sight {
                    hidden_since = None;
                    trimmed_this_idle = false;
                    continue;
                }

                let since = *hidden_since.get_or_insert_with(std::time::Instant::now);
                if !trimmed_this_idle && since.elapsed() >= IDLE_BEFORE_TRIM {
                    trim_all();
                    trimmed_this_idle = true;
                }
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
pub fn start_memory_trimmer(_app: tauri::AppHandle) {}

#[cfg(not(target_os = "windows"))]
pub fn trim_all() {}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn trim_memory() {}
