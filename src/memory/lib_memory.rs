use std::ffi::c_void;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Module32NextW, MODULEENTRY32W, TH32CS_SNAPMODULE, Module32FirstW, TH32CS_SNAPMODULE32};

pub fn get_module_base_address(module_name: &str, pid: u32) -> Option<usize> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid).unwrap();

        let mut entry = MODULEENTRY32W::default();
        entry.dwSize = size_of::<MODULEENTRY32W>() as u32;

        let mut res: Option<usize> = None;

        if Module32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &entry.szModule[..entry.szModule.iter().position(|&c| c == 0).unwrap_or(0)]
                );
                println!("ModuleBase Address: {}", name);
                if name.eq_ignore_ascii_case(module_name) {
                    res = Some(entry.modBaseAddr as usize);
                    break;
                }

                if Module32NextW(snapshot, &mut entry).is_err() {
                    res = None;
                    break;
                }
            }
        };

        let _ = CloseHandle(snapshot);
        res
    }
}

pub fn read_bytes_from_memory(process: HANDLE, address: *const c_void) -> Option<i32> {
    let mut buffer = [0u8; 4];
    let mut bytes_read= 0;

    let res = unsafe {
        ReadProcessMemory(
            process,
            address,
            buffer.as_mut_ptr() as *mut c_void,
            buffer.len(),
            Some(&mut bytes_read),
        )
    };

    if res.is_ok() {
        Some(i32::from_le_bytes(buffer))
    } else {
        None
    }
}