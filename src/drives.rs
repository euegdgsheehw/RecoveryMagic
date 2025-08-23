use serde::Serialize;
use widestring::U16CStr;
use winapi::um::fileapi::GetDriveTypeW;
use winapi::um::fileapi::GetLogicalDrives;
use winapi::um::fileapi::{GetDiskFreeSpaceExW, GetVolumeInformationW};
use winapi::um::winbase::{DRIVE_FIXED, DRIVE_REMOVABLE};
use winapi::um::winnt::WCHAR;

#[derive(Debug, Clone, Serialize)]
pub struct SourceDrive {
    pub letter: char,
    pub fs_name: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

// ドライブを取得する関数
// NTFSのみに絞り込む
pub fn enum_ntfs_drives() -> Vec<SourceDrive> {
    let mut out = Vec::<SourceDrive>::new();
    unsafe {
        let mask = GetLogicalDrives();
        for i in 0..26u32 {
            if (mask & (1 << i)) == 0 {
                continue;
            }
            let letter = (b'A' + (i as u8)) as char;
            let root = format!("{}:\\", letter);
            let root_w: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();

            let dtype = GetDriveTypeW(root_w.as_ptr());
            if dtype != DRIVE_FIXED && dtype != DRIVE_REMOVABLE {
                continue;
            }

            let mut fs_name_buf: [WCHAR; 64] = [0; 64];
            let mut vol_name_buf: [WCHAR; 64] = [0; 64];
            let mut serial: u32 = 0;
            let mut max_comp_len: u32 = 0;
            let mut fs_flags: u32 = 0;
            let ok = GetVolumeInformationW(
                root_w.as_ptr(),
                vol_name_buf.as_mut_ptr(),
                vol_name_buf.len() as u32,
                &mut serial,
                &mut max_comp_len,
                &mut fs_flags,
                fs_name_buf.as_mut_ptr(),
                fs_name_buf.len() as u32,
            );
            if ok == 0 {
                continue;
            }
            let fs_name = U16CStr::from_ptr_str(fs_name_buf.as_ptr())
                .to_string_lossy()
                .to_string();
            if fs_name.to_uppercase() != "NTFS" {
                continue;
            }

            let mut avail: u64 = 0;
            let mut total: u64 = 0;
            let mut free: u64 = 0;
            let ok2 = GetDiskFreeSpaceExW(
                root_w.as_ptr(),
                &mut avail as *mut u64 as *mut _,
                &mut total as *mut u64 as *mut _,
                &mut free as *mut u64 as *mut _,
            );
            if ok2 == 0 {
                continue;
            }

            out.push(SourceDrive {
                letter,
                fs_name,
                total_bytes: total,
                free_bytes: free,
            });
        }
    }
    out
}
