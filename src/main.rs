mod helpers;

use std::collections::HashMap;
use std::process;
use std::ffi::c_void;
use std::ops::{Add, Deref};
use ndarray::Array;
use procmod_overlay::{Color, Overlay, OverlayTarget};
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32First, PROCESSENTRY32, TH32CS_SNAPPROCESS, Process32Next, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, Module32FirstW, Module32NextW};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};
use crate::helpers::lib_memory::{get_gmod_process_id, get_module_base_address, read_f32_bytes_from_memory, read_i32_bytes_from_memory, read_matrix};
use crate::helpers::math::to_world_screen;
use crate::helpers::offsets::OFFSETS;


#[derive(Debug)]
struct EntityBot {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[warn(unused_variables)]
fn main() -> procmod_overlay::Result<()> {

    unsafe{
        let gmod_pid = get_gmod_process_id().unwrap_or_else(|| panic!("Unable to get process id from gmod open the game first"));

        let game_process =  OpenProcess(PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION, false, gmod_pid).unwrap();

        println!("Starting game process {}", gmod_pid);
        let client_dll = get_module_base_address("client.dll", gmod_pid).ok_or("couldnt find client.dll :(").unwrap();
        let engine_dll = get_module_base_address("engine.dll", gmod_pid).ok_or("couldnt find engine.dll :(").unwrap();

        let mut entities = Vec::new();

        let mut overlay = Overlay::new(OverlayTarget::Pid(gmod_pid))?;

        // let viewmatrix = read_matrix(game_process, (engine_dll + OFFSETS.lock().unwrap().get("PLAYER_VIEWMATRIX").unwrap()) as *const c_void).unwrap();
        // dbg!(viewmatrix);
        loop{
            overlay.begin_frame()?;

            for i in 1..30 {
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
                let plr_x = read_f32_bytes_from_memory(game_process, (read.unwrap() as usize + 0x026C) as *const c_void).unwrap_or_else(|| panic!("couldnt find plr_x."));
                let plr_y = read_f32_bytes_from_memory(game_process, (read.unwrap() + 0x0270) as *const c_void).unwrap_or_else(|| panic!("couldnt find plr_y."));
                let plr_z = read_f32_bytes_from_memory(game_process, (read.unwrap() + 0x0274) as *const c_void).unwrap_or_else(|| panic!("couldnt find plr_z"));
                let head_z = read_f32_bytes_from_memory(game_process, (read.unwrap() + 0x0274) as *const c_void).unwrap_or_else(|| panic!("couldnt find plr_head_z")) + 64.0;

                // println!("Plr x -> {}\ny -> {}\nz -> {}\n", plr_x, plr_y, plr_z);

                let viewmatrix = read_matrix(game_process, (engine_dll + OFFSETS.lock().unwrap().get("PLAYER_VIEWMATRIX").unwrap()) as *const c_void).unwrap();

                let feet_coords = to_world_screen(viewmatrix, [plr_x, plr_y, plr_z], overlay.size());
                let head_coords = to_world_screen(viewmatrix, [plr_x, plr_y, head_z ], overlay.size());
                //
                //
                let height = feet_coords.1 - head_coords.1;
                let width = height / 2.5;
                if feet_coords != (0.0,0.0){
                    entities.push(EntityBot {
                        x: head_coords.0 - width / 2.0,
                        y: head_coords.1,
                        w: width,
                        h: height,
                    });
                }
                //
                for ent in entities.iter(){
                    println!("{:?}", ent);
                    // overlay.rect(ent.x, ent.y,100.0, 100.0, Color::RED);
                    overlay.rect(ent.x, ent.y, ent.w, ent.h, Color::RED);
                    // overlay.text(ent.x - 20.0, ent.y + 20.0, "instrumentation active", 16.0, Color::WHITE);
                }

                entities.clear();
                overlay.end_frame()?;
            }



        }

    }



}

