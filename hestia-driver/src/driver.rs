use wdk::{nt_success, println};
use wdk_sys::{
    call_unsafe_wdf_function_binding, ntddk::RtlInitUnicodeString, DRIVER_OBJECT, GUID, NTSTATUS,
    PCUNICODE_STRING, STATUS_SUCCESS, TRUE, UNICODE_STRING, WDFDEVICE, WDFDEVICE_INIT, WDFDRIVER,
    WDFQUEUE, WDFREQUEST, WDF_IO_QUEUE_CONFIG, WDF_NO_HANDLE, WDF_NO_OBJECT_ATTRIBUTES,
    _WDF_IO_QUEUE_DISPATCH_TYPE::WdfIoQueueDispatchManual,
};
use windows_strings::w;

use crate::{
    callbacks::{install_callbacks, uninstall_callbacks},
    make_unicode_string,
    wdf::{create_driver, WDFDriverConfig},
    DEFINE_GUID,
};

// {509969B5-B0E0-452C-A372-A9A754D0423C}
DEFINE_GUID!(
    HESTIA_GUID,
    0x509969b5,
    0xb0e0,
    0x452c,
    0xa3,
    0x72,
    0xa9,
    0xa7,
    0x54,
    0xd0,
    0x42,
    0x3c
);

pub unsafe fn initialize_driver(
    driver_object: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    println!("[initialize_driver] Creating WDF Driver");
    let config = WDFDriverConfig::new().driver_unload(Some(evt_driver_unload));
    let driver = match create_driver(driver_object, registry_path, config) {
        Ok(driver) => driver,
        Err(ntstatus) => {
            println!(
                "[initialize_driver] Failed to create WDF Driver: 0x{:08X}",
                ntstatus
            );
            return ntstatus;
        }
    };
    let status = create_device(driver);
    if !nt_success(status) {
        return status;
    }
    if let Err(ntstatus) = install_callbacks() {
        println!(
            "[initialize_driver] Failed to install callbacks: 0x{:08X}",
            ntstatus
        );
        return ntstatus;
    }
    STATUS_SUCCESS
}

fn create_device(driver: WDFDRIVER) -> NTSTATUS {
    println!("[evt_driver_device_add]");

    println!("[evt_driver_device_add] Initializing SDDL UNICODE_STRING");
    make_unicode_string!(
        sddl_unicode_string,
        "D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;WD)"
    );

    println!("[evt_driver_device_add] Allocating DeviceInit structure");
    let mut pwdfdevice_init = unsafe {
        call_unsafe_wdf_function_binding!(
            WdfControlDeviceInitAllocate,
            driver,
            &sddl_unicode_string
        )
    };

    let attributes = WDF_NO_OBJECT_ATTRIBUTES;

    println!("[evt_driver_device_add] Creating WDF Device");
    let mut device = WDF_NO_HANDLE as WDFDEVICE;
    let mut ntstatus = unsafe {
        call_unsafe_wdf_function_binding!(
            WdfDeviceCreate,
            &mut pwdfdevice_init,
            WDF_NO_OBJECT_ATTRIBUTES,
            &mut device
        )
    };
    if !nt_success(ntstatus) {
        println!(
            "[initialize_driver] Failed to created WDF Device: 0x{:08X}",
            ntstatus
        );
        unsafe { call_unsafe_wdf_function_binding!(WdfDeviceInitFree, pwdfdevice_init) };
        return ntstatus;
    }

    // plante au chargement .. unicode en cause ? investiguer la macro, ou méthode de construction unicode

    // println!("[evt_driver_device_add] Creating Symlink");
    // make_unicode_string!(symlink_name, "\\\\??\\\\Hestia");
    // let status = unsafe {
    //     call_unsafe_wdf_function_binding!(WdfDeviceCreateSymbolicLink, device, &symlink_name)
    // };
    // if !nt_success(status) {
    //     println!(
    //         "[initialize_driver] Failed to creat symlink: 0x{:08X}",
    //         status
    //     );
    //     unsafe { call_unsafe_wdf_function_binding!(WdfDeviceInitFree, pwdfdevice_init) };
    //     return status;
    // }

    // println!("[evt_driver_device_add] Creating I/O Queue");
    // let wdfqueue_handle = WDF_NO_HANDLE.cast::<WDFQUEUE>();
    // let mut queue_config = WDF_IO_QUEUE_CONFIG {
    //     Size: core::mem::size_of::<WDF_IO_QUEUE_CONFIG>() as _,
    //     DispatchType: WdfIoQueueDispatchManual,
    //     DefaultQueue: TRUE as _,
    //     EvtIoDeviceControl: Some(evt_io_device_control),
    //     ..Default::default()
    // };

    // ntstatus = unsafe {
    //     call_unsafe_wdf_function_binding!(
    //         WdfIoQueueCreate,
    //         device,
    //         &mut queue_config,
    //         WDF_NO_OBJECT_ATTRIBUTES,
    //         wdfqueue_handle
    //     )
    // };

    ntstatus
}

unsafe extern "C" fn evt_io_device_control(
    _queue: WDFQUEUE,
    _request: WDFREQUEST,
    _x: usize,
    _y: usize,
    _z: u32,
) {
    println!("[evt_io_device_control]");
}

unsafe extern "C" fn evt_driver_unload(_driver: WDFDRIVER) {
    // if let Err(ntstatus) = uninstall_callbacks() {
    //     println!(
    //         "[initialize_driver] Failed to uninstall callbacks: 0x{:08X}",
    //         ntstatus
    //     );
    // }
    println!("[evt_driver_unload]");
}
