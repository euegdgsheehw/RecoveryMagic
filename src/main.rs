#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod drives;
mod fs;
mod gui_bridge;
mod indexer;
mod logging;
mod scan;
mod util;

use gui_bridge::{
    build_filelist_cmd, copy_to_desktop_cmd, eject_cmd, list_drives_cmd, open_path_cmd,
    reveal_in_explorer_cmd, start_mount_cmd, AppState,
};

#[cfg(windows)]
fn ensure_admin_or_relaunch_early() {
    use std::{mem, os::windows::ffi::OsStrExt, ptr::null_mut};
    use winapi::um::{
        handleapi::CloseHandle,
        processthreadsapi::GetCurrentProcess,
        securitybaseapi::GetTokenInformation,
        shellapi::ShellExecuteW,
        winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY},
        winuser::SW_SHOW,
    };

    unsafe {
        let mut tok: HANDLE = null_mut();
        if winapi::um::processthreadsapi::OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY,
            &mut tok,
        ) != 0
        {
            let mut elev: TOKEN_ELEVATION = mem::zeroed();
            let mut len: u32 = 0;
            let ok = GetTokenInformation(
                tok,
                TokenElevation,
                &mut elev as *mut _ as _,
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut len,
            );
            CloseHandle(tok);
            if ok != 0 && elev.TokenIsElevated == 0 {
                if let Ok(exe) = std::env::current_exe() {
                    let exe_w: Vec<u16> = exe.as_os_str().encode_wide().chain(Some(0)).collect();
                    let op: Vec<u16> = "runas\0".encode_utf16().collect();
                    let ret = ShellExecuteW(
                        null_mut(),
                        op.as_ptr(),
                        exe_w.as_ptr(),
                        std::ptr::null(),
                        std::ptr::null(),
                        SW_SHOW,
                    );
                    if (ret as usize) > 32 {
                        std::process::exit(0);
                    }
                }
            }
        }
    }
}

fn main() {
    #[cfg(windows)]
    ensure_admin_or_relaunch_early();

    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            list_drives_cmd,
            start_mount_cmd,
            eject_cmd,
            build_filelist_cmd,
            open_path_cmd,
            copy_to_desktop_cmd,
            reveal_in_explorer_cmd
        ])
        .setup(|app| {
            let handle = logging::init_tracing_and_gui_emitter(app.handle());
            let _ = logging::raise_to_warn_if_release(&handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
