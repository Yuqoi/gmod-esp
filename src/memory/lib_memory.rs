use std::ffi::c_void;
use std::io::Error;
use std::{f32, process};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Module32NextW, MODULEENTRY32W, TH32CS_SNAPMODULE, Module32FirstW, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS, PROCESSENTRY32, Process32First, Process32Next};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};
use crate::offsets::OFFSETS;

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

pub fn read_i32_bytes_from_memory(process: HANDLE, address: *const c_void) -> Option<i32> {
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

pub fn read_f32_bytes_from_memory(process: HANDLE, address: *const c_void) -> Option<f32> {
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
        Some(f32::from_le_bytes(buffer))
    } else {
        None
    }
}

pub fn get_gmod_process_id() -> Option<u32>{
    unsafe {
        let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap_or_else(|e| {
            println!("CreateToolhelp32Snapshot Error: {}", e);
            process::exit(1);
        });

        let mut pe32: PROCESSENTRY32 = PROCESSENTRY32::default();
        pe32.dwSize = size_of_val(&pe32) as u32;

        Process32First(handle, &mut pe32).unwrap_or_else(|e| {
            println!("Process32First Error: {}", e);
            process::exit(1);
        });

        loop {
            let name = String::from_utf8(pe32.szExeFile.to_vec().iter().map(|x| *x as u8).collect()).unwrap_or_else(|e| {
                process::exit(1);
            });

            if name.contains("gmod.exe"){
                break Some(pe32.th32ProcessID);
            }

            if let Err(_) = Process32Next(handle, &mut pe32) {
                break None
            }
        }

    }
}