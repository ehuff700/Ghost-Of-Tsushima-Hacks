#![allow(non_snake_case)]

#[macro_export]
macro_rules! map_win_ptr {
    ($result:expr) => {{
        let r = $result;
        if r.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        r
    }};
    ($result:expr, $failure_value:expr) => {{
        let r = $result;
        if r == $failure_value {
            return Err(std::io::Error::last_os_error().into());
        }
        r
    }};
}

#[macro_export]
macro_rules! map_win_bool {
    ($result:expr) => {{
        let r = $result;
        if r == false {
            return Err(std::io::Error::last_os_error().into());
        }
        r
    }};
}

use std::{ffi::OsString, os::windows::ffi::OsStringExt};

use crate::{
    api::{
        constants::{
            INVALID_HANDLE_VALUE, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ,
            PROCESS_VM_WRITE, TH32CS_SNAPPROCESS,
        },
        prototypes::*,
        structs::PROCESSENTRY32W,
        wintypes::{HANDLE, HMODULE},
    },
    cli::Material,
};

pub struct ProcessInfo {
    pub(crate) pid: u32,
    pub(crate) name: OsString,
}

impl ProcessInfo {
    pub fn compare_by_name(&self, other: impl Into<OsString>) -> bool {
        let other = other.into();
        self.name.eq_ignore_ascii_case(other)
    }
}

#[derive(Debug, Clone)]
pub struct GameHandle {
    pub handle: HANDLE,
    pub process_id: u32,
    pub image_base: u64,
}

impl GameHandle {
    /// Reads the memory at a given offset and returns the result
    pub fn read_memory<const N: usize>(
        &self,
        offset: u64,
    ) -> Result<[u8; N], Box<dyn std::error::Error>> {
        let mut buffer = [0u8; N];
        let final_address = self.image_base + offset;
        map_win_bool!(unsafe {
            ReadProcessMemory(
                self.handle,
                final_address as *mut _,
                buffer.as_mut_ptr() as *mut _,
                N,
                std::ptr::null_mut(),
            )
        });
        Ok(buffer)
    }

    /// Writes the given value to the memory at the given offset
    pub fn write_memory(
        &self,
        offset: u64,
        value: &mut [u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let final_address = self.image_base + offset;
        map_win_bool!(unsafe {
            WriteProcessMemory(
                self.handle,
                final_address as *mut _,
                value.as_mut_ptr() as *mut _,
                value.len(),
                std::ptr::null_mut(),
            )
        });
        Ok(())
    }
}

impl Drop for GameHandle {
    fn drop(&mut self) {
        if !unsafe { CloseHandle(self.handle) } {
            eprintln!(
                "failed to close process handle: {}",
                std::io::Error::last_os_error()
            );
        }
    }
}

/// Retrieves a list of all running processes on the system.
fn GetProcessList() -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
    let h_process_snap = map_win_ptr!(
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) },
        INVALID_HANDLE_VALUE
    );

    let mut pe32 = unsafe { std::mem::zeroed::<PROCESSENTRY32W>() };
    pe32.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    unsafe {
        if !Process32FirstW(h_process_snap, &mut pe32) {
            CloseHandle(h_process_snap);
            return Err("failed to retrieve first process entry".into());
        }
    }

    let mut process_list = Vec::new();
    while unsafe { Process32NextW(h_process_snap, &mut pe32) } {
        let name = std::ffi::OsString::from_wide(
            &pe32.szExeFile[..pe32.szExeFile.iter().position(|c| *c == 0).unwrap()],
        );
        let process_info = ProcessInfo {
            pid: pe32.th32ProcessID,
            name,
        };
        process_list.push(process_info);
    }
    unsafe { CloseHandle(h_process_snap) };
    Ok(process_list)
}

/// Retrieves the base address of the game's executable module.
fn GetImageBase(h_process: HANDLE) -> Result<u64, Box<dyn std::error::Error>> {
    let mut cb_needed = 0;
    unsafe {
        EnumProcessModules(h_process, std::ptr::null_mut(), 0, &mut cb_needed);
    }
    let module_count = cb_needed / std::mem::size_of::<HMODULE>() as u32;
    let mut h_modules = vec![0 as HMODULE; module_count as usize];
    map_win_bool!(unsafe {
        EnumProcessModules(h_process, h_modules.as_mut_ptr(), cb_needed, &mut cb_needed)
    });
    Ok(h_modules[0] as u64)
}

/// Retrieves a valid handle to the specified game process.
pub fn GetGameHandle(game_name: &str) -> Result<Option<GameHandle>, Box<dyn std::error::Error>> {
    let process_list = GetProcessList()?;
    let game_info = process_list
        .into_iter()
        .find_map(|p| p.compare_by_name(game_name).then_some(p));

    if let Some(game_info) = game_info {
        let handle = unsafe {
            map_win_ptr!(OpenProcess(
                PROCESS_VM_READ
                    | PROCESS_VM_WRITE
                    | PROCESS_VM_OPERATION
                    | PROCESS_QUERY_INFORMATION,
                false,
                game_info.pid
            ))
        };

        let image_base = GetImageBase(handle)?;
        Ok(Some(GameHandle {
            handle,
            process_id: game_info.pid,
            image_base,
        }))
    } else {
        Ok(None)
    }
}

/// Sets a given material to the provided value.
///
/// Returns the new value on success
pub fn SetMaterial(
    game_handle: &GameHandle,
    material: Material,
    value: u32,
) -> Result<u32, Box<dyn std::error::Error>> {
    game_handle.write_memory(material.offset(), &mut value.to_le_bytes())?;
    Ok(value)
}

/// Adds a given material amount to the provided value.
///
/// Returns the new value on success
pub fn AddMaterial(
    game_handle: &GameHandle,
    material: Material,
    value: u32,
) -> Result<u32, Box<dyn std::error::Error>> {
    // Get the current material amount and increment it by the given value.
    let material_amount: [u8; 4] = game_handle.read_memory(material.offset())?;
    let mut material_amount = u32::from_le_bytes(material_amount);
    material_amount += value;

    // Write the modified value back to the game process.
    game_handle.write_memory(material.offset(), &mut material_amount.to_le_bytes())?;
    Ok(material_amount)
}

/// Subtracts a given material amount from the provided value.
///
/// Returns the new value on success
pub fn SubtractMaterial(
    game_handle: &GameHandle,
    material: Material,
    value: u32,
) -> Result<u32, Box<dyn std::error::Error>> {
    // Get the current material amount and decrement it by the given value.
    let material_amount: [u8; 4] = game_handle.read_memory(material.offset())?;
    let mut material_amount = u32::from_le_bytes(material_amount);
    material_amount -= value;

    // Write the modified value back to the game process.
    game_handle.write_memory(material.offset(), &mut material_amount.to_le_bytes())?;
    Ok(material_amount)
}
