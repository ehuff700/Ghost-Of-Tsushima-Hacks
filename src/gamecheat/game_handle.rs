#![allow(non_snake_case)]

#[macro_export]
macro_rules! map_win_ptr {
    ($func:ident($($arg:expr),*)) => {{
        let ret = $func($($arg),*);
        let r = if ret.is_null() || ret == $crate::api::constants::INVALID_HANDLE_VALUE {
            Err($crate::errors::GamecheatError::OperationError(stringify!($func), std::io::Error::last_os_error()))
        } else {
            Ok(ret)
        };
        r
    }};
}

#[macro_export]
macro_rules! map_win_bool {
    ($func:ident($($arg:expr),*)) => {{
        let ret = $func($($arg),*) as bool;
        if ret == false {
            Err($crate::errors::GamecheatError::OperationError(stringify!($func), std::io::Error::last_os_error()))
        } else {
            Ok(ret)
        }
    }};
}

#[macro_export]
macro_rules! map_win_int {
    ($func:ident($($arg:expr),*)) => {{
        let ret = $func($($arg),*);
        if ret == 0 {
            Err($crate::errors::GamecheatError::OperationError(stringify!($func), std::io::Error::last_os_error()))
        } else {
            Ok(ret)
        }
    }};
}
use std::{ffi::OsString, os::windows::ffi::OsStringExt};

use crate::{
    api::{
        constants::{
            MAX_PATH, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
            TH32CS_SNAPPROCESS,
        },
        prototypes::*,
        structs::PROCESSENTRY32W,
        wintypes::{HANDLE, HMODULE},
    },
    errors::GamecheatError,
    GamecheatResult,
};

#[derive(Debug, Clone)]
/// A handle to a running game process.
///
/// This struct contains the raw handle from OpenHandle and image base.
pub struct GameHandle {
    pub handle: HANDLE,
    pub image_base: u64,
}

impl GameHandle {
    /// Creates a new GameHandle for the given game process name.
    pub fn new(game_name: &'static str) -> GamecheatResult<Self> {
        let process_list = GetProcessList()?;
        let game_info = process_list
            .into_iter()
            .find_map(|(pid, name)| name.eq_ignore_ascii_case(game_name).then_some((pid, name)));

        if let Some((process_id, _)) = game_info {
            let handle = unsafe {
                map_win_ptr!(OpenProcess(
                    PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_QUERY_INFORMATION,
                    false,
                    process_id
                ))?
            };

            let image_base = GetImageBase(handle, game_name)?;
            Ok(GameHandle { handle, image_base })
        } else {
            Err(GamecheatError::GameProcessNotFound(game_name))
        }
    }

    /// Reads the memory at a given offset and returns the result
    fn read_memory<const N: usize>(&self, offset: u64) -> GamecheatResult<[u8; N]> {
        let mut buffer = [0u8; N];
        let final_address = self.image_base + offset;
        unsafe {
            map_win_bool!(ReadProcessMemory(
                self.handle,
                final_address as *const _,
                buffer.as_mut_ptr() as *mut _,
                N,
                std::ptr::null_mut()
            ))?
        };
        Ok(buffer)
    }

    /// Writes the given value to the memory at the given offset
    fn write_memory(&self, offset: u64, value: &[u8]) -> GamecheatResult<()> {
        let final_address = self.image_base + offset;
        unsafe {
            map_win_bool!(WriteProcessMemory(
                self.handle,
                final_address as *const _,
                value.as_ptr() as *const _,
                value.len(),
                std::ptr::null_mut()
            ))?
        };
        Ok(())
    }

    /// Writes a u8 value to the memory at the given offset
    pub fn write_u8(&self, offset: u64, value: u8) -> GamecheatResult<()> {
        self.write_memory(offset, &[value])
    }

    /// Reads a u8 value from the memory at the given offset
    pub fn read_u8(&self, offset: u64) -> GamecheatResult<u8> {
        let byte: [u8; 1] = self.read_memory(offset)?;
        Ok(byte[0])
    }

    /// Writes a u16 value to the memory at the given offset
    pub fn write_u16(&self, offset: u64, value: u16) -> GamecheatResult<()> {
        self.write_memory(offset, &value.to_le_bytes())
    }

    /// Reads a u16 value from the memory at the given offset
    pub fn read_u16(&self, offset: u64) -> GamecheatResult<u16> {
        Ok(u16::from_le_bytes(self.read_memory(offset)?))
    }

    /// Writes a u32 value to the memory at the given offset
    pub fn write_u32(&self, offset: u64, value: u32) -> GamecheatResult<()> {
        self.write_memory(offset, &value.to_le_bytes())
    }

    /// Reads a u32 value from the memory at the given offset
    pub fn read_u32(&self, offset: u64) -> GamecheatResult<u32> {
        Ok(u32::from_le_bytes(self.read_memory(offset)?))
    }

    /* Getters */
    pub fn handle(&self) -> HANDLE {
        self.handle
    }

    pub fn image_base(&self) -> u64 {
        self.image_base
    }
}

impl Drop for GameHandle {
    fn drop(&mut self) {
        if !unsafe { CloseHandle(self.handle) } {
            error!(
                "failed to close process handle: {}",
                std::io::Error::last_os_error()
            );
        }
    }
}

/// Retrieves a list of all running processes on the system.
fn GetProcessList() -> GamecheatResult<Vec<(u32, OsString)>> {
    let h_process_snap = unsafe { map_win_ptr!(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0))? };

    let mut pe32 = unsafe { std::mem::zeroed::<PROCESSENTRY32W>() };
    pe32.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    unsafe {
        if !Process32FirstW(h_process_snap, &mut pe32) {
            CloseHandle(h_process_snap);
            return Err(GamecheatError::ProcessEnumError);
        }
    }

    let mut process_list = Vec::new();
    while unsafe { Process32NextW(h_process_snap, &mut pe32) } {
        let name = std::ffi::OsString::from_wide(
            &pe32.szExeFile[..pe32.szExeFile.iter().position(|c| *c == 0).unwrap()],
        );
        process_list.push((pe32.th32ProcessID, name));
    }
    unsafe { CloseHandle(h_process_snap) };
    Ok(process_list)
}

/// Retrieves the base address of the game's executable module.
fn GetImageBase(h_process: HANDLE, game_name: &'static str) -> GamecheatResult<u64> {
    let mut cb_needed = 0;
    unsafe {
        EnumProcessModules(h_process, std::ptr::null_mut(), 0, &mut cb_needed);
    }
    let module_count = cb_needed / std::mem::size_of::<HMODULE>() as u32;
    let mut h_modules = vec![0 as HMODULE; module_count as usize];
    unsafe {
        map_win_bool!(EnumProcessModules(
            h_process,
            h_modules.as_mut_ptr(),
            cb_needed,
            &mut cb_needed
        ))?
    };

    for h_module in h_modules {
        let mut buffer = [0u16; MAX_PATH];
        let file_len = unsafe {
            map_win_int!(GetModuleBaseNameW(
                h_process,
                h_module,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32
            ))
        }?;
        let file_name = OsString::from_wide(&buffer[..file_len as usize]);
        if file_name.eq_ignore_ascii_case(game_name) {
            return Ok(h_module as u64);
        }
    }

    Err(GamecheatError::GameModuleNotFound)
}
