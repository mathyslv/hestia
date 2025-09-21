use wdk::{nt_success, println};
use wdk_sys::{
    ntddk::PsSetCreateProcessNotifyRoutineEx, FALSE, HANDLE, PEPROCESS, PPS_CREATE_NOTIFY_INFO,
    TRUE,
};

use crate::Result;

pub fn install_create_process_callback() -> Result<()> {
    let ntstatus =
        unsafe { PsSetCreateProcessNotifyRoutineEx(Some(create_process_callback), FALSE as _) };
    if !nt_success(ntstatus) {
        return Err(ntstatus);
    }
    Ok(())
}

pub fn uninstall_create_process_callback() -> Result<()> {
    let ntstatus =
        unsafe { PsSetCreateProcessNotifyRoutineEx(Some(create_process_callback), TRUE as _) };
    if !nt_success(ntstatus) {
        return Err(ntstatus);
    }
    Ok(())
}

unsafe extern "C" fn create_process_callback(
    process: PEPROCESS,
    process_id: HANDLE,
    create_info: PPS_CREATE_NOTIFY_INFO,
) {
    if !create_info.is_null() {
        println!(
            "[create_process_callback] Process {} is creating",
            process_id as u32
        );
    } else {
        println!(
            "[create_process_callback] Process {} is exiting",
            process_id as u32
        );
    }
    // unsafe { *create_info }.CreationStatus
}
