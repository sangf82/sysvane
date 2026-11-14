//! Tells the core when Windows wakes from sleep, so modules can write their saved state to the hardware
//! again. The callback runs on a system thread: it only calls `Devices::resume`, which never blocks.

use crate::devices::Devices;
use std::ffi::c_void;
use std::sync::Arc;
use windows_sys::Win32::System::Power::{
    DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, RegisterSuspendResumeNotification,
};

const PBT_APMRESUMEAUTOMATIC: u32 = 0x12;
const DEVICE_NOTIFY_CALLBACK: u32 = 2;

unsafe extern "system" fn on_power(context: *const c_void, kind: u32, _: *const c_void) -> u32 {
    if kind == PBT_APMRESUMEAUTOMATIC {
        crate::log::info("resume from sleep: re-applying modules");
        // SAFETY: `context` is the `Devices` leaked by `watch`, which is never freed.
        unsafe { &*(context as *const Devices) }.resume();
    }
    0
}

pub fn watch(devices: Arc<Devices>) {
    // The registration lives as long as the process, so the parameters and the context are leaked on purpose.
    let params = Box::leak(Box::new(DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
        Callback: Some(on_power),
        Context: Arc::into_raw(devices) as *mut c_void,
    }));
    // SAFETY: `params` is valid for the process life; DEVICE_NOTIFY_CALLBACK says the handle is that struct.
    let handle = unsafe {
        RegisterSuspendResumeNotification(
            params as *mut DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS as _,
            DEVICE_NOTIFY_CALLBACK,
        )
    };
    if handle == 0 {
        crate::log::error("cannot watch for resume from sleep");
    }
}
