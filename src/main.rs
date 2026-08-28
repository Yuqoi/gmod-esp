use std::{mem, process};
use std::arch::x86_64::_mm512_add_round_ph;
use std::ffi::c_void;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32First, PROCESSENTRY32, TH32CS_SNAPPROCESS, Process32Next, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, Module32FirstW, Module32NextW};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};
use windows::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32W;


// 0080DC74 – 40 = 80DC34
//
//
// 71886314B4

#[warn(unused_variables)]
fn main() {
    unsafe {
        let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap_or_else(|e| {
            println!("CreateToolhelp32Snapshot Error: {}", e);
            process::exit(1);
        });

        let mut pe32: PROCESSENTRY32 = PROCESSENTRY32::default();
        pe32.dwSize = size_of_val(&pe32) as u32;

        println!("{:?}", size_of_val(&pe32) as u32);

        Process32First(handle, &mut pe32).unwrap_or_else(|e| {
            println!("Process32First Error: {}", e);
            process::exit(1);
        });

        let mut game_pid: Option<u32> = None;

        loop {
            let name = String::from_utf8(pe32.szExeFile.to_vec().iter().map(|x| *x as u8).collect()).unwrap_or_else(|e| {
                println!("String::from_utf8 Error: {}", e);
                process::exit(1);
            });

            if name.contains("gmod.exe"){
                break game_pid = Some(pe32.th32ProcessID);
            }

            if let Err(_) = Process32Next(handle, &mut pe32) {
                break;
            }
        }

        let game_process = OpenProcess(PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION, false, game_pid.unwrap()).unwrap();

        let client_dll = get_module_base_address("client.dll", game_pid.unwrap()).ok_or("Not found :(").unwrap();
        let base_pointer_address = client_dll + 0x0080DC74;

        let first_pointer = read_bytes_from_memory(game_process, base_pointer_address as *const c_void)
            .ok_or("Failed to read base pointer").unwrap();

        let target_value_address = first_pointer + 0x0098;
        let final_value = read_bytes_from_memory(game_process, target_value_address as *const c_void)
            .ok_or("Failed to read final byte value").unwrap();

        println!("health val {}", final_value as u8);

    }
}

fn get_module_base_address(module_name: &str, pid: u32) -> Option<usize> {
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

fn read_bytes_from_memory(process: HANDLE, address: *const c_void) -> Option<i32> {
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