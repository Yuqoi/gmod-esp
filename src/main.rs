mod offsets;
mod memory;
use crate::memory::lib_memory::{get_gmod_process_id, get_module_base_address, read_f32_bytes_from_memory, read_i32_bytes_from_memory};

use std::process;
use std::ffi::c_void;
use std::ops::Add;
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32First, PROCESSENTRY32, TH32CS_SNAPPROCESS, Process32Next, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, Module32FirstW, Module32NextW};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};
use crate::offsets::OFFSETS;



#[warn(unused_variables)]
fn main() {

    unsafe{
        let gmod_pid = get_gmod_process_id().unwrap_or_else(|| panic!("Unable to get process id from gmod open the game first"));

        let game_process =  OpenProcess(PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION, false, gmod_pid).unwrap();

        println!("Starting game process {}", gmod_pid);
        let client_dll = get_module_base_address("client.dll", gmod_pid).ok_or("couldnt find client.dll :(").unwrap();
        let engine_dll = get_module_base_address("engine.dll", gmod_pid).ok_or("couldnt find engine.dll :(").unwrap();

        //
        // let player_health_value = read_bytes_from_memory(game_process, player_health_address as *const c_void)
        //     .ok_or("Failed to read final byte value")
        //     .unwrap();

        for i in 0..30 {
            let entity_ptr = client_dll + OFFSETS.lock().unwrap().get("PLAYER_OFFSET").unwrap() + 0x0004 * i;
            let read = read_i32_bytes_from_memory(game_process, entity_ptr as *const c_void);

            if read.is_none(){
                continue;
            }

            let health_point_address = read.unwrap() as usize + OFFSETS.lock().unwrap().get("PLAYER_HEALTH_ADDRESS").unwrap();
            let entity_health = read_i32_bytes_from_memory(game_process, health_point_address as *const c_void);

            if entity_health.is_none() || entity_health.unwrap() <= 0{
                continue;
            }

            let red: Vec<u8> = vec![255,0,0];
            // 026C x
            // 0270 y
            // 0274 z
            let plr_x = read_f32_bytes_from_memory(game_process, (read.unwrap() as usize + 0x026C) as *const c_void).unwrap_or_else(|| panic!("couldnt find plr_x."));
            let plr_y = read_f32_bytes_from_memory(game_process, (read.unwrap() + 0x0270) as *const c_void).unwrap_or_else(|| panic!("couldnt find plr_y."));
            let plr_z = read_f32_bytes_from_memory(game_process, (read.unwrap() + 0x0274) as *const c_void).unwrap_or_else(|| panic!("couldnt find plr_z"));
            let head_z = read_f32_bytes_from_memory(game_process, (read.unwrap() + 0x0274) as *const c_void).unwrap_or_else(|| panic!("couldnt find plr_head_z")) + 64.0;

            println!("Starting plr game process {}, {}, {}", plr_x, plr_y, plr_z );
        }

    }



}

