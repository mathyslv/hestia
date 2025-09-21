use wdk::{nt_success, println};
use wdk_sys::{
    call_unsafe_wdf_function_binding, DRIVER_OBJECT, PCUNICODE_STRING, PFN_WDF_DRIVER_DEVICE_ADD,
    PFN_WDF_DRIVER_UNLOAD, STATUS_INVALID_PARAMETER, ULONG, WDFDRIVER, WDF_DRIVER_CONFIG,
    WDF_NO_HANDLE, WDF_NO_OBJECT_ATTRIBUTES,
};

use crate::Result;

#[derive(Default)]
pub struct WDFDriverConfig {
    pub evt_driver_device_add: PFN_WDF_DRIVER_DEVICE_ADD,
    pub evt_driver_unload: PFN_WDF_DRIVER_UNLOAD,
    pub driver_init_flags: ULONG,
    pub driver_pool_tag: ULONG,
}

impl WDFDriverConfig {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn driver_device_add(mut self, evt_driver_device_add: PFN_WDF_DRIVER_DEVICE_ADD) -> Self {
        self.evt_driver_device_add = evt_driver_device_add;
        self
    }

    pub fn driver_unload(mut self, evt_driver_unload: PFN_WDF_DRIVER_UNLOAD) -> Self {
        self.evt_driver_unload = evt_driver_unload;
        self
    }

    pub fn int_flags(mut self, driver_init_flags: ULONG) -> Self {
        self.driver_init_flags = driver_init_flags;
        self
    }

    pub fn pool_tags(mut self, driver_pool_tag: ULONG) -> Self {
        self.driver_pool_tag = driver_pool_tag;
        self
    }
}

impl From<WDFDriverConfig> for WDF_DRIVER_CONFIG {
    fn from(val: WDFDriverConfig) -> Self {
        WDF_DRIVER_CONFIG {
            Size: core::mem::size_of::<WDF_DRIVER_CONFIG>() as _,
            EvtDriverDeviceAdd: val.evt_driver_device_add,
            EvtDriverUnload: val.evt_driver_unload,
            DriverInitFlags: val.driver_init_flags,
            DriverPoolTag: val.driver_pool_tag,
        }
    }
}

#[allow(non_snake_case)]
pub fn create_driver(
    driver_object: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
    config: WDFDriverConfig,
) -> Result<WDFDRIVER> {
    let mut driver = 0 as WDFDRIVER;
    let ntstatus = unsafe {
        call_unsafe_wdf_function_binding!(
            WdfDriverCreate,
            driver_object,
            registry_path,
            WDF_NO_OBJECT_ATTRIBUTES,
            &mut config.into(),
            &mut driver,
        )
    };
    if !nt_success(ntstatus) {
        println!(
            "[create_driver] Error: call to WdfDriverCreate failed (0x{:08X})",
            ntstatus
        );
        return Err(ntstatus);
    } else if driver.is_null() {
        println!(
            "[create_driver] Error: driver pointer is null (0x{:08X?})",
            driver
        );
        return Err(STATUS_INVALID_PARAMETER);
    }
    println!("[create_driver] Driver pointer is 0x{:08X?}", driver);
    Ok(driver)
}
