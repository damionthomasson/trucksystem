use serde::Serialize;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::thread;
use std::time::Duration;
use tauri::Emitter; // Removed 'Manager' to fix the warning
use enigo::{Enigo, Key, KeyboardControllable};

// Official Windows Bindings
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{MapViewOfFile, OpenFileMappingW, FILE_MAP_READ};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow};

#[derive(Clone, Serialize)]
struct TelemetryPayload {
    sdkActive: bool,
    speed: f32,
    gear: i32,
    rpm: f32,
    maxRpm: f32, 
    odometer: f32,
    parkBrake: bool,
    fuel: f32,
    fuelRange: f32,
    fuelAvgCons: f32,
}

fn to_wstring(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect()
}

#[tauri::command]
fn start_telemetry(app_handle: tauri::AppHandle) {
    // --- THREAD 1: THE AUTO-BYPASS ---
    thread::spawn(|| {
        let mut enigo = Enigo::new();
        let target_title = to_wstring("Request to use advanced SDK features");
        loop {
            unsafe {
                let hwnd = FindWindowW(None, windows::core::PCWSTR(target_title.as_ptr()));
                if hwnd.0 != 0 {
                    let _ = SetForegroundWindow(hwnd);
                    thread::sleep(Duration::from_millis(500));
                    enigo.key_click(Key::Return);
                    println!("✅ SDK Bypass: Clicked OK.");
                    break; 
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    });

    // --- THREAD 2: NATIVE MEMORY READER ---
    thread::spawn(move || {
        let map_name = to_wstring("Local\\SCSTelemetry");

        unsafe {
            let mut handle: HANDLE;
            loop {
                handle = OpenFileMappingW(
                    FILE_MAP_READ.0, 
                    false,
                    windows::core::PCWSTR(map_name.as_ptr()),
                ).unwrap_or(HANDLE::default());

                if handle.0 != 0 { 
                    println!("✅ SUCCESS! Connected to SCS Memory Map.");
                    break; 
                }
                thread::sleep(Duration::from_secs(2));
            }

            let map_ptr = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, 0);
            if map_ptr.Value.is_null() { return; }

            let base_ptr = map_ptr.Value as *const u8;

            loop {
                let sdk_active = ptr::read_unaligned(base_ptr.add(0) as *const u8) != 0;
                let gear = ptr::read_unaligned(base_ptr.add(504) as *const i32);
                let fuel_cap = ptr::read_unaligned(base_ptr.add(704) as *const f32).max(1.0);
                
                // FIXED: Changed variable name to maxRpm to match the struct field exactly
                let maxRpm = ptr::read_unaligned(base_ptr.add(708) as *const f32).max(1.0);
                
                let speed_mps = ptr::read_unaligned(base_ptr.add(948) as *const f32);
                let rpm = ptr::read_unaligned(base_ptr.add(952) as *const f32);
                let fuel_liters = ptr::read_unaligned(base_ptr.add(1000) as *const f32);
                let fuel_cons = ptr::read_unaligned(base_ptr.add(1004) as *const f32);
                let fuel_range = ptr::read_unaligned(base_ptr.add(1008) as *const f32);
                let odometer = ptr::read_unaligned(base_ptr.add(1056) as *const f32);
                let park_brake = ptr::read_unaligned(base_ptr.add(1566) as *const u8) != 0;

                let payload = TelemetryPayload {
                    sdkActive: sdk_active,
                    speed: (speed_mps * 3.6).abs(),
                    gear,
                    rpm,
                    maxRpm, // This now matches the variable above
                    odometer,
                    parkBrake: park_brake,
                    fuel: (fuel_liters / fuel_cap) * 100.0,
                    fuelRange: fuel_range, 
                    fuelAvgCons: fuel_cons * 100.0, 
                };

                if let Err(_) = app_handle.emit("telemetry-update", payload) {
                    break; 
                }
                
                thread::sleep(Duration::from_millis(33)); 
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_telemetry])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}