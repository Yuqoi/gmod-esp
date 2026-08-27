use std::{mem, process};
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32First, PROCESSENTRY32, TH32CS_SNAPPROCESS, Process32Next};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};

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
                println!("Process32Next Error");
                break;
            }
        }

        println!("{:?}", game_pid);

        let process = OpenProcess( PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION, false, game_pid.unwrap());

    }
}
