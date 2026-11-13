//! NVIDIA GPU readings, read in a worker process (`worker.rs`).

use nvml_wrapper::{
    Nvml,
    enum_wrappers::device::{Clock, TemperatureSensor},
};
use sysvane_wire::Gpu;

fn read(nvml: &Nvml) -> Option<Gpu> {
    let d = nvml.device_by_index(0).ok()?;
    let mem = d.memory_info().ok()?;
    Some(Gpu {
        name: d.name().ok()?,
        driver: nvml.sys_driver_version().ok(),
        util: d.utilization_rates().ok()?.gpu,
        mem_used: mem.used,
        mem_total: mem.total,
        temp_c: d.temperature(TemperatureSensor::Gpu).ok(),
        power_w: d.power_usage().ok().map(|mw| mw as f32 / 1000.0),
        power_limit_w: d.enforced_power_limit().ok().map(|mw| mw as f32 / 1000.0),
        clock_gr_mhz: d.clock_info(Clock::Graphics).ok(),
        clock_mem_mhz: d.clock_info(Clock::Memory).ok(),
        fan_pct: d.fan_speed(0).ok(),
        enc_util: d.encoder_utilization().ok().map(|u| u.utilization),
        dec_util: d.decoder_utilization().ok().map(|u| u.utilization),
        pcie_gen: d.current_pcie_link_gen().ok(),
        pcie_width: d.current_pcie_link_width().ok(),
    })
}

/// Worker entry point: see `worker::run`.
pub fn worker_main() {
    let nvml = Nvml::init().ok();
    crate::worker::run(|| nvml.as_ref().and_then(read));
}
