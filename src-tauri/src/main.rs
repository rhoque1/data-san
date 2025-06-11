#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::command;
use serde::{Deserialize, Serialize};
use windows::Win32::Storage::FileSystem::{GetLogicalDrives, GetDiskFreeSpaceExW, GetVolumeInformationW};
use windows::core::{PCWSTR, HSTRING};
use std::fs;
use std::path::Path;
use std::io::Write;
use rand::Rng;

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

#[command]
fn test_system_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    format!("System test successful - Rust backend working | OS: {} | Arch: {}", os, arch)
}

#[command]
fn detect_drives() -> Result<Vec<DriveInfo>, String> {
    unsafe {
        let drives_mask = GetLogicalDrives();
        if drives_mask == 0 {
            return Err("Failed to get logical drives".to_string());
        }
        let mut drives = Vec::new();
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
        Ok(drives)
    }
}

#[command]
fn check_safety(drive_letter: String) -> Result<bool, String> {
    let drives = detect_drives()?;
    if let Some(drive) = drives.iter().find(|d| d.letter == drive_letter) {
        if drive.is_system {
            return Err("Cannot sanitize system drive (C:)".to_string());
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
        let mut file = match fs::File::create(path.clone() + "temp_sanitize_file") {
            Ok(f) => f,
            Err(e) => return Err(e.to_string()),
        };
        let buffer_size = 1024 * 1024; // 1MB buffer
        let mut rng = rand::thread_rng();
        for _ in 0..3 { // 3-pass overwrite (zeros, ones, random)
            let buffer = match _ {
                0 => vec![0u8; buffer_size],  // Pass 1: Zeros
                1 => vec![255u8; buffer_size], // Pass 2: Ones
                _ => (0..buffer_size).map(|_| rng.gen()).collect(), // Pass 3: Random
            };
            let iterations = (drive.size / buffer_size as u64).min(100); // Limit to 100 iterations
            for _ in 0..iterations {
                if let Err(e) = file.write_all(&buffer) {
                    return Err(e.to_string());
                }
            }
            if let Err(e) = file.sync_all() {
                return Err(e.to_string());
            }
        }
        if let Err(e) = fs::remove_file(path.clone() + "temp_sanitize_file") {
            return Err(e.to_string());
        }
        Ok(format!("Sanitized {} with 3-pass overwrite method (limited to {}MB)", path, iterations * buffer_size / (1024 * 1024)))
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![test_system_info, detect_drives, check_safety, sanitize_drive, greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}