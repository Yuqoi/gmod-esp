mod offsets;
mod memory;
use crate::memory::lib_memory::{get_module_base_address, read_bytes_from_memory};

use std::process;
use std::ffi::c_void;
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32First, PROCESSENTRY32, TH32CS_SNAPPROCESS, Process32Next, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, Module32FirstW, Module32NextW};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};
use crate::offsets::OFFSETS;


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
        let base_pointer_address = client_dll + OFFSETS.lock().unwrap().get("PLAYER_HEALTH").unwrap();

        let first_pointer = read_bytes_from_memory(game_process, base_pointer_address as *const c_void)
            .ok_or("Failed to read base pointer").unwrap();

        let target_value_address = first_pointer + 0x0098;
        let final_value = read_bytes_from_memory(game_process, target_value_address as *const c_void)
            .ok_or("Failed to read final byte value").unwrap();

        println!("health val {}", final_value as u8);

    }
}

