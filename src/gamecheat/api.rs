#![allow(clippy::upper_case_acronyms, non_snake_case, non_camel_case_types)]
pub mod wintypes {
    use std::ffi::c_void;
    pub type HANDLE = *mut c_void;
    pub type HMODULE = *mut c_void;
    pub type DWORD = u32;
    pub type LONG = i32;
    pub type BOOL = bool;
    pub type ULONG_PTR = usize;
    pub type WCHAR = u16;
    pub type LPWSTR = *mut WCHAR;
    pub type SIZE_T = usize;
    pub type LPVOID = *const c_void;
}

pub mod constants {
    use super::*;
    use wintypes::*;
    pub const MAX_PATH: usize = 260;
    pub const TH32CS_SNAPPROCESS: DWORD = 0x00000002;
    pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
    pub const PROCESS_VM_READ: DWORD = 0x0010;
    pub const PROCESS_VM_WRITE: DWORD = 0x0020;
    pub const PROCESS_QUERY_INFORMATION: DWORD = 0x0400;
    pub const PAGE_READONLY: DWORD = 0x02;
}

pub mod structs {
    use super::*;
    use constants::*;
    use wintypes::*;

    #[repr(C)]
    pub struct PROCESSENTRY32W {
        pub dwSize: DWORD,
        pub cntUsage: DWORD,
        pub th32ProcessID: DWORD,
        pub th32DefaultHeapID: ULONG_PTR,
        pub th32ModuleID: DWORD,
        pub cntThreads: DWORD,
        pub th32ParentProcessID: DWORD,
        pub pcPriClassBase: LONG,
        pub dwFlags: DWORD,
        pub szExeFile: [WCHAR; MAX_PATH],
    }
}

pub mod prototypes {
    use super::*;
    use structs::*;
    use wintypes::*;

    extern "C" {
        /* Process Manipulation */
        pub fn OpenProcess(
            dwDesiredAccess: DWORD,
            bInheritHandle: bool,
            dwProcessId: DWORD,
        ) -> HANDLE;
        pub fn CloseHandle(hObject: HANDLE) -> BOOL;
        pub fn ReadProcessMemory(
            hProcess: HANDLE,
            lpBaseAddress: LPVOID,
            lpBuffer: *mut std::ffi::c_void,
            nSize: SIZE_T,
            lpNumberOfBytesRead: *mut SIZE_T,
        ) -> BOOL;
        pub fn WriteProcessMemory(
            hProcess: HANDLE,
            lpBaseAddress: LPVOID,
            lpBuffer: *const std::ffi::c_void,
            nSize: SIZE_T,
            lpNumberOfBytesRead: *mut SIZE_T,
        ) -> BOOL;
        pub fn VirtualProtectEx(
            hProcess: HANDLE,
            lpAddress: LPVOID,
            dwSize: SIZE_T,
            flNewProtect: DWORD,
            lpflOldProtect: *mut DWORD,
        ) -> BOOL;
        /* Process enumeration */
        pub fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD) -> HANDLE;
        pub fn Process32FirstW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
        pub fn Process32NextW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
        pub fn EnumProcessModules(
            hProcess: HANDLE,
            lphModule: *mut HMODULE,
            cb: DWORD,
            lpcbNeeded: *mut DWORD,
        ) -> BOOL;
        pub fn GetModuleBaseNameW(
            hProcess: HANDLE,
            hModule: HMODULE,
            lpBaseName: LPWSTR,
            nSize: DWORD,
        ) -> DWORD;

    }
}
