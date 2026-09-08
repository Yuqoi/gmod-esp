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
struct EntityBox {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug)]
struct EntityCoords {
    x: f32,
    y: f32,
    z: f32,
    head: f32,
}

fn get_esp_coords(viewmatrix: &[f32; 16], entity_coords: &EntityCoords, window_size: &(u32, u32)) -> Option<(f32, f32, f32, f32)> {
    let feet_coords = match to_world_screen(*viewmatrix, [entity_coords.x, entity_coords.y, entity_coords.z], *window_size){
        Some(coords) => coords,
        None => return None
    };
    let head_coords = to_world_screen(*viewmatrix, [entity_coords.x, entity_coords.y, entity_coords.head], *window_size);

    let (feet_x, feet_y) = feet_coords;
    let (_head_x, head_y) = head_coords.unwrap();

    let height = feet_y - head_y ;
    if height <= 0. {
        return None
    }

    let width = height / 2.5;
    let x = feet_x - (width / 2.);
    let y = head_y;

    Some((x, y, width, height))
}

#[warn(unused_variables)]
fn main() -> procmod_overlay::Result<()> {

    unsafe{
        let gmod_pid = get_gmod_process_id().unwrap_or_else(|| panic!("Unable to get process id from gmod open the game first"));

        let game_process =  OpenProcess(PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION, false, gmod_pid).unwrap();

        println!("Starting game process: {}", gmod_pid);
        let client_dll = get_module_base_address("client.dll", gmod_pid).ok_or("couldnt find client.dll :(").unwrap();
        let engine_dll = get_module_base_address("engine.dll", gmod_pid).ok_or("couldnt find engine.dll :(").unwrap();

        let mut entities:Vec<EntityBox> = Vec::new();
        let mut overlay = Overlay::new(OverlayTarget::Pid(gmod_pid))?;

        loop{
            overlay.begin_frame()?;

            for i in 1 ..100 {
                entities.clear();

                let entity_ptr = client_dll + OFFSETS.lock().unwrap().get("PLAYER_OFFSET").unwrap() + 0x0004 * i;
                let read = read_i32_bytes_from_memory(game_process, entity_ptr as *const c_void);

                if read.is_none(){
                    continue;
                }

                let health_point_address = read.unwrap() as usize + OFFSETS.lock().unwrap().get("PLAYER_HEALTH_ADDRESS").unwrap();
                let entity_health = match read_i32_bytes_from_memory(game_process, health_point_address as *const c_void){
                    Some(val) => if val as f32 <= 0.0 {
                        continue;
                    }else{
                        val
                    },
                    None => continue,
                };


                let x = match read_f32_bytes_from_memory(game_process, (read.unwrap() as usize + 0x026C) as *const c_void) {
                    Some(val) => val,
                    None => continue
                };
                let y = match read_f32_bytes_from_memory(game_process, (read.unwrap() + 0x0270) as *const c_void) {
                    Some(val) => val,
                    None => continue
                };
                let z = match read_f32_bytes_from_memory(game_process, (read.unwrap() + 0x0274) as *const c_void) {
                    Some(val) => val,
                    None => continue
                };
                let head = match read_f32_bytes_from_memory(game_process, (read.unwrap() + 0x0274) as *const c_void) {
                    Some(val) => val + 64.,
                    None => continue
                };


                let plr_coords = EntityCoords { x,y,z,head };


                let viewmatrix = match read_matrix(game_process, (engine_dll + OFFSETS.lock().unwrap().get("PLAYER_VIEWMATRIX").unwrap()) as *const c_void) {
                    Some(val) => val,
                    None => break,
                };

                match get_esp_coords(&viewmatrix, &plr_coords, &overlay.size()){
                    Some(coords) => entities.push(
                        EntityBox{
                            x: coords.0,
                            y: coords.1,
                            w: coords.2,
                            h: coords.3,
                        }
                    ),
                    None => continue,
                };

                for ent in entities.iter(){
                    overlay.rect(ent.x, ent.y, ent.w, ent.h, Color::RED);
                }

            }
            overlay.end_frame()?;

        }

    }



}

