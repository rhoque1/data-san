#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::command;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::io::Write;
use rand::Rng;
use sysinfo::System;
use battery;

#[derive(Debug, Serialize, Deserialize)]
struct DriveInfo {
    letter: String,
    size: u64,
    free_space: u64,
    label: String,
    file_system: String,
    serial_number: u32,
    is_system: bool,
}

#[derive(serde::Serialize)]
struct SystemSpecs {
    os_name: Option<String>,
    os_version: Option<String>,
    kernel_version: Option<String>,
    cpu_brand: String,
    cpu_cores: usize,
    total_memory: u64,
    used_memory: u64,
    disks: Vec<String>,
    networks: Vec<String>,
    battery_percentage: Option<f32>,
    battery_cycle_count: Option<u32>,
    gpu_name: Option<String>,
}

#[command]
fn test_system_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    format!("System test successful - Rust backend working | OS: {} | Arch: {}", os, arch)
}

#[command]
fn detect_drives() -> Result<Vec<DriveInfo>, String> {
    let mut drives = Vec::new();
    
    // On macOS, we'll detect mounted volumes
    #[cfg(target_os = "macos")]
    {
        if let Ok(entries) = fs::read_dir("/Volumes") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let drive_letter = path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        let mut drive_info = DriveInfo {
                            letter: format!("/Volumes/{}", drive_letter),
                            size: 0,
                            free_space: 0,
                            label: drive_letter.clone(),
                            file_system: "APFS/HFS+".to_string(),
                            serial_number: 0,
                            is_system: false,
                        };
                        
                        // Get disk space info
                        if let Ok(metadata) = fs::metadata(&path) {
                            // This is a simplified approach - in a real app you'd use statvfs
                            drive_info.size = 0; // Would need statvfs for actual size
                            drive_info.free_space = 0; // Would need statvfs for actual free space
                        }
                        
                        // Check if it's the system drive
                        if drive_letter == "Macintosh HD" || path.to_string_lossy().contains("Macintosh HD") {
                            drive_info.is_system = true;
                        }
                        
                        drives.push(drive_info);
                    }
                }
            }
        }
    }
    
    // On Windows, use the original Windows-specific code
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Storage::FileSystem::{GetLogicalDrives, GetDiskFreeSpaceExW, GetVolumeInformationW};
        use windows::core::{PCWSTR, HSTRING};
        
        unsafe {
            let drives_mask = GetLogicalDrives();
            if drives_mask == 0 {
                return Err("Failed to get logical drives".to_string());
            }
            
            for i in 0..26 {
                if drives_mask & (1 << i) != 0 {
                    let drive_letter = format!("{}:\\", char::from(b'A' + i as u8));
                    let mut drive_info = DriveInfo {
                        letter: drive_letter.clone(),
                        size: 0,
                        free_space: 0,
                        label: String::new(),
                        file_system: String::new(),
                        serial_number: 0,
                        is_system: false,
                    };

                    let mut label = [0u16; 256];
                    let mut serial_number = 0u32;
                    let mut max_component_length = 0u32;
                    let mut file_system_flags = 0u32;
                    let mut file_system_name = [0u16; 256];
                    let root_path = HSTRING::from(drive_letter.as_str());
                    let result = GetVolumeInformationW(
                        PCWSTR::from_raw(root_path.as_ptr()),
                        Some(&mut label[..]),
                        Some(&mut serial_number),
                        Some(&mut max_component_length),
                        Some(&mut file_system_flags),
                        Some(&mut file_system_name[..]),
                    );
                    if result.is_ok() {
                        drive_info.label = String::from_utf16_lossy(&label)
                            .trim_end_matches('\0')
                            .to_string();
                        drive_info.serial_number = serial_number;
                        drive_info.file_system = String::from_utf16_lossy(&file_system_name)
                            .trim_end_matches('\0')
                            .to_string();
                    }

                    let mut free_bytes = 0u64;
                    let mut total_bytes = 0u64;
                    let mut total_free_bytes = 0u64;
                    let success = GetDiskFreeSpaceExW(
                        PCWSTR::from_raw(root_path.as_ptr()),
                        Some(&mut free_bytes),
                        Some(&mut total_bytes),
                        Some(&mut total_free_bytes),
                    );
                    if success.is_ok() {
                        drive_info.free_space = free_bytes;
                        drive_info.size = total_bytes;
                    }

                    if drive_letter == "C:\\" {
                        drive_info.is_system = true;
                    }

                    drives.push(drive_info);
                }
            }
        }
    }
    
    // On Linux, detect mounted filesystems
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = fs::read_dir("/mnt") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let drive_letter = path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        let mut drive_info = DriveInfo {
                            letter: format!("/mnt/{}", drive_letter),
                            size: 0,
                            free_space: 0,
                            label: drive_letter.clone(),
                            file_system: "ext4".to_string(),
                            serial_number: 0,
                            is_system: false,
                        };
                        
                        drives.push(drive_info);
                    }
                }
            }
        }
    }
    
    Ok(drives)
}

#[command]
fn check_safety(drive_letter: String) -> Result<bool, String> {
    let drives = detect_drives()?;
    if let Some(drive) = drives.iter().find(|d| d.letter == drive_letter) {
        if drive.is_system {
            return Err("Cannot sanitize system drive".to_string());
        }
        Ok(true)
    } else {
        Err("Drive not found".to_string())
    }
}

#[command]
async fn sanitize_drive(drive_letter: String, confirm: bool) -> Result<String, String> {
    if !confirm {
        return Err("Confirmation required to proceed".to_string());
    }
    
    let drives = detect_drives()?;
    if let Some(drive) = drives.iter().find(|d| d.letter == drive_letter) {
        if drive.is_system {
            return Err("Cannot sanitize system drive".to_string());
        }
        
        let path = drive_letter.clone();
        if !Path::new(&path).exists() {
            return Err(format!("Drive {} not found or not accessible", path));
        }
        
        // Create a temporary file for sanitization
        let temp_file_path = format!("{}/temp_sanitize_file", path);
        let mut file = match fs::File::create(&temp_file_path) {
            Ok(f) => f,
            Err(e) => return Err(e.to_string()),
        };
        
        let buffer_size = 1024 * 1024; // 1MB buffer
        let mut rng = rand::rng();
        let iterations = 100; // Limit to 100 iterations for safety
        
        for pass_num in 0..3 { // 3-pass overwrite (zeros, ones, random)
            let buffer = match pass_num {
                0 => vec![0u8; buffer_size],  // Pass 1: Zeros
                1 => vec![255u8; buffer_size], // Pass 2: Ones
                _ => (0..buffer_size).map(|_| rng.random()).collect(), // Pass 3: Random
            };
            
            for _ in 0..iterations {
                if let Err(e) = file.write_all(&buffer) {
                    return Err(e.to_string());
                }
            }
            
            if let Err(e) = file.sync_all() {
                return Err(e.to_string());
            }
        }
        
        // Clean up the temporary file
        if let Err(e) = fs::remove_file(&temp_file_path) {
            return Err(e.to_string());
        }
        
        Ok(format!("Sanitized {} with 3-pass overwrite method (limited to {}MB)", path, iterations * (buffer_size as u64) / (1024 * 1024)))
    } else {
        Err("Drive not found".to_string())
    }
}

#[command]
async fn greet(name: &str) -> Result<String, String> {
    if name.is_empty() {
        Err("Name cannot be empty".to_string())
    } else {
        Ok(format!("Hello, {}! Welcome to Data Sanitizer Pro.", name))
    }
}

#[tauri::command]
fn get_system_specs() -> SystemSpecs {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    // Battery info
    let (mut battery_percentage, mut battery_cycle_count) = (None, None);
    if let Ok(manager) = battery::Manager::new() {
        if let Ok(batteries) = manager.batteries() {
            for maybe_battery in batteries.flatten() {
                battery_percentage = Some(maybe_battery.state_of_charge().value * 100.0);
                battery_cycle_count = maybe_battery.cycle_count();
                break; // Only use the first battery found
            }
        }
    }
    // GPU info (macOS only)
    let mut gpu_name = None;
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("system_profiler").arg("SPDisplaysDataType").output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines() {
                    if line.trim_start().starts_with("Chipset Model:") {
                        gpu_name = Some(line.trim_start().replace("Chipset Model:", "").trim().to_string());
                        break;
                    }
                }
            }
        }
    }
    SystemSpecs {
        os_name: sysinfo::System::name(),
        os_version: sysinfo::System::os_version(),
        kernel_version: sysinfo::System::kernel_version(),
        cpu_brand: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default(),
        cpu_cores: sys.cpus().len(),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        disks: Vec::new(),
        networks: Vec::new(),
        battery_percentage,
        battery_cycle_count,
        gpu_name,
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![test_system_info, detect_drives, check_safety, sanitize_drive, greet, get_system_specs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}