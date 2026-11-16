//! Monitors over DDC/CI: brightness, contrast, colour preset and RGB gains, plus profiles. One module per
//! physical monitor, so a monitor that stops answering costs only itself. Safety rules live in `vcp`.
//! The monitor keeps its own settings across sleep, so re-apply only reads them again. Resolution and
//! refresh rate go through Windows (`mode`) and are reverted by the module unless the user keeps them.
//! Colour through the GPU gamma ramp (`gamma`) works without DDC and is written again whenever Windows
//! may have reset it.

pub mod ddc;
pub mod gamma;
pub mod mode;
pub mod profile;
pub mod vcp;

use crate::modules::Module;
use mode::Mode;
use profile::{Profile, Store};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use sysvane_wire::{Control, ControlId, ControlValue, DeviceId};
use vcp::Bus;

const CONTROLS: [(ControlId, u8); 6] = [
    (ControlId::Brightness, vcp::BRIGHTNESS),
    (ControlId::Contrast, vcp::CONTRAST),
    (ControlId::ColorPreset, vcp::PRESET),
    (ControlId::RedGain, vcp::RED),
    (ControlId::GreenGain, vcp::GREEN),
    (ControlId::BlueGain, vcp::BLUE),
];

const GPU: [ControlId; 6] = [
    ControlId::GpuBrightness,
    ControlId::GpuContrast,
    ControlId::GpuGamma,
    ControlId::GpuRed,
    ControlId::GpuGreen,
    ControlId::GpuBlue,
];

fn gpu_value(g: &mut gamma::Gpu, id: ControlId) -> Option<&mut u16> {
    Some(match id {
        ControlId::GpuBrightness => &mut g.brightness,
        ControlId::GpuContrast => &mut g.contrast,
        ControlId::GpuGamma => &mut g.gamma,
        ControlId::GpuRed => &mut g.red,
        ControlId::GpuGreen => &mut g.green,
        ControlId::GpuBlue => &mut g.blue,
        _ => return None,
    })
}

/// What a display trial goes back to: the mode and the orientation before it.
#[derive(Clone, Copy)]
struct Before {
    mode: Mode,
    turn: u8,
}

pub struct Monitor<B: Bus> {
    index: u8,
    display: String,
    caps: vcp::Caps,
    bus: B,
    store: Store,
    /// Only values read from the monitor, with its maximum; nothing is shown before it is read.
    values: BTreeMap<u8, (u16, u16)>,
    modes: Vec<Mode>,
    mode: Option<Mode>,
    turn: u8,
    /// A display mode or orientation on trial: what to go back to, and when.
    trial: Option<(Before, Instant)>,
    trial_for: Duration,
    /// GPU colour as last set; written to the display only once it is not neutral.
    gpu: gamma::Gpu,
}

impl<B: Bus> Monitor<B> {
    pub fn new(index: u8, display: String, bus: B, store: Store) -> Self {
        Self {
            index,
            display,
            caps: vcp::Caps::default(),
            bus,
            store,
            values: BTreeMap::new(),
            modes: Vec::new(),
            mode: None,
            turn: 0,
            trial: None,
            trial_for: mode::TRIAL,
            gpu: gamma::Gpu::default(),
        }
    }

    /// shortcut: keyed by the Windows display number, which can change when monitors are re-plugged; key by
    /// EDID serial once the module reads it.
    fn gpu_file(&self) -> std::path::PathBuf {
        self.store.file(&format!("gpu-{}.toml", self.display))
    }

    /// Writes the GPU colour again, as Windows drops it on a mode change, sleep or a full-screen game.
    fn apply_gpu(&mut self) -> Result<(), String> {
        if self.gpu.is_neutral() {
            return Ok(());
        }
        self.bus.set_gamma(&self.gpu.ramp())
    }

    fn read_mode(&mut self) {
        self.modes = self.bus.modes();
        self.mode = self.bus.mode();
        self.turn = self.bus.turn().unwrap_or(0);
    }

    /// Every change of mode or orientation is a trial. It keeps what was there before the first change, so
    /// trying several in a row still reverts to what the user had.
    fn trial_set(&mut self, m: Mode, turn: u8) -> Result<(), String> {
        let before = match self.trial {
            Some((before, _)) => before,
            None => Before {
                mode: self.mode.ok_or("the current display mode is unknown")?,
                turn: self.turn,
            },
        };
        self.bus.set_mode(m, turn, false)?;
        let back = m != before.mode || turn != before.turn;
        self.trial = back.then(|| (before, Instant::now() + self.trial_for));
        Ok(())
    }

    /// Only a mode the display lists, in the current orientation.
    fn try_mode(&mut self, value: ControlValue) -> Result<(), String> {
        let m: Mode = Self::text(value)?.parse()?;
        if !self.modes.contains(&m) {
            return Err(format!("{m} is not a mode this display lists"));
        }
        self.trial_set(m, self.turn)
    }

    /// The current mode turned: portrait and landscape swap width and height.
    fn try_turn(&mut self, value: ControlValue) -> Result<(), String> {
        let turn = match value {
            ControlValue::Int(n @ 0..=3) => n as u8,
            _ => return Err("orientation is 0 to 3 quarter turns".to_owned()),
        };
        let mut m = self.mode.ok_or("the current display mode is unknown")?;
        if turn % 2 != self.turn % 2 {
            (m.width, m.height) = (m.height, m.width);
        }
        self.trial_set(m, turn)
    }

    fn end_trial(&mut self, keep: bool) -> Result<(), String> {
        let Some((before, _)) = self.trial.take() else {
            return Ok(());
        };
        match (keep, self.mode) {
            (true, Some(now)) => self.bus.set_mode(now, self.turn, true),
            _ => self.bus.set_mode(before.mode, before.turn, false),
        }
    }

    /// A GPU colour value from the UI. The ramp is written first and saved only once Windows took it.
    fn set_gpu(&mut self, control: ControlId, value: ControlValue) -> Result<(), String> {
        let n = match value {
            ControlValue::Int(n @ 0..=100) => n as u16,
            _ => return Err("a GPU colour setting is 0 to 100".to_owned()),
        };
        let mut next = self.gpu;
        *gpu_value(&mut next, control).ok_or("not a GPU colour setting")? = n;
        self.bus.set_gamma(&next.ramp())?;
        self.gpu = next;
        crate::modules::config::save(&self.gpu_file(), &self.gpu)
    }

    /// The named controls, then every other code the monitor lists that is kept as a percentage, such as
    /// volume or 6-axis saturation. shortcut: 0x0C and picture mode are left out (a raw range, a model's own
    /// values); add them when the UI can show a raw range.
    fn list(&self) -> Vec<(ControlId, u8)> {
        let extra = self
            .caps
            .values
            .keys()
            .copied()
            .filter(|&c| vcp::scaled(c) && !CONTROLS.iter().any(|&(_, n)| n == c))
            .map(|c| (ControlId::Vcp(c), c));
        CONTROLS.into_iter().chain(extra).collect()
    }

    fn refresh(&mut self) -> Result<(), String> {
        self.values = self
            .list()
            .iter()
            .filter_map(|&(_, code)| Some((code, self.bus.read(code).ok()?)))
            .collect();
        if self.values.is_empty() {
            return Err("not answering over DDC/CI (is DDC/CI on in the monitor menu?)".to_owned());
        }
        Ok(())
    }

    fn text(value: ControlValue) -> Result<String, String> {
        match value {
            ControlValue::Text(name) => Ok(name),
            _ => Err("a profile is named by text".to_owned()),
        }
    }
}

impl Monitor<ddc::Physical> {
    /// Every monitor Windows lists, numbered in that order.
    pub fn detect(store: impl Fn() -> Store) -> Vec<Self> {
        ddc::detect()
            .into_iter()
            .zip(0..=u8::MAX)
            .map(|(f, i)| Self::new(i, f.display, f.bus, store()))
            .collect()
    }
}

impl<B: Bus> Module for Monitor<B> {
    fn id(&self) -> DeviceId {
        DeviceId::Monitor(self.index)
    }

    fn name(&self) -> String {
        match &self.caps.model {
            Some(m) => format!("{m} · {}", self.display),
            None => self.display.clone(),
        }
    }

    fn start(&mut self) -> Result<(), String> {
        self.read_mode();
        self.caps = self
            .bus
            .capabilities()
            .map(|c| vcp::Caps::parse(&c))
            .unwrap_or_default();
        self.gpu = crate::modules::config::load::<gamma::Gpu>(&self.gpu_file()).clamped();
        let gpu = self.apply_gpu();
        self.refresh().and(gpu)
    }

    fn controls(&self) -> Vec<Control> {
        let ddc = self.list().into_iter().filter_map(|(id, code)| {
            let (value, max) = self.values.get(&code).copied()?;
            let level = if vcp::scaled(code) {
                vcp::to_percent(value, max)
            } else {
                value
            };
            Some(Control {
                id,
                value: ControlValue::Int(level.into()),
            })
        });
        let mode = self.mode.map(|m| Control {
            id: ControlId::DisplayMode,
            value: ControlValue::Text(m.to_string()),
        });
        // Only for a display Windows shows: the orientation and the ramp belong to it.
        let turn = self.mode.map(|_| Control {
            id: ControlId::Orientation,
            value: ControlValue::Int(self.turn.into()),
        });
        let mut gpu = self.gpu;
        let gpu = GPU
            .map(|id| Control {
                id,
                value: ControlValue::Int((*gpu_value(&mut gpu, id).expect("a GPU control")).into()),
            })
            .into_iter()
            .filter(|_| self.mode.is_some());
        let left = self.trial.map(|(_, at)| Control {
            id: ControlId::KeepDisplayMode,
            value: ControlValue::Int(
                at.saturating_duration_since(Instant::now())
                    .as_secs_f64()
                    .ceil() as i64,
            ),
        });
        ddc.chain(mode).chain(turn).chain(gpu).chain(left).collect()
    }

    fn modes(&self) -> Vec<String> {
        self.modes.iter().map(Mode::to_string).collect()
    }

    /// The MCCS presets the capabilities list. A monitor that lists none gets every MCCS preset in the UI.
    fn presets(&self) -> Vec<u16> {
        self.caps
            .values
            .get(&vcp::PRESET)
            .map(|v| v.iter().copied().filter(|p| (1..=13).contains(p)).collect())
            .unwrap_or_default()
    }

    fn wake(&self) -> Option<Duration> {
        let (_, at) = self.trial?;
        Some(
            at.saturating_duration_since(Instant::now())
                .min(Duration::from_secs(1)),
        )
    }

    fn tick(&mut self) -> Result<(), String> {
        if self.trial.is_some_and(|(_, at)| Instant::now() >= at) {
            crate::system::log::info("display mode not kept: reverting");
            let result = self.end_trial(false);
            self.read_mode();
            return result.and(self.apply_gpu());
        }
        Ok(())
    }

    fn profiles(&self) -> Vec<String> {
        self.store.names()
    }

    fn set(&mut self, control: ControlId, value: ControlValue) -> Result<(), String> {
        if let ControlId::DisplayMode | ControlId::Orientation | ControlId::KeepDisplayMode =
            control
        {
            let result = match (control, value) {
                (ControlId::DisplayMode, v) => self.try_mode(v),
                (ControlId::Orientation, v) => self.try_turn(v),
                (_, ControlValue::Bool(keep)) => self.end_trial(keep),
                _ => Err("keep is yes or no".to_owned()),
            };
            self.read_mode();
            return result.and(self.apply_gpu());
        }
        if GPU.contains(&control) {
            return self.set_gpu(control, value);
        }
        let result = match control {
            ControlId::Profile => {
                let name = Self::text(value)?;
                let model = self.caps.model.as_deref();
                self.store.get(&name)?.apply(&mut self.bus, model)
            }
            ControlId::SaveProfile => {
                let name = Self::text(value)?;
                let model = self.caps.model.as_deref();
                return self
                    .store
                    .save(&Profile::capture(&name, &mut self.bus, model)?);
            }
            _ => {
                let (_, code) = self
                    .list()
                    .into_iter()
                    .find(|(id, _)| *id == control)
                    .ok_or_else(|| format!("a monitor has no {control:?}"))?;
                let ControlValue::Int(n) = value else {
                    return Err("a monitor setting is a whole number".to_owned());
                };
                let n = u16::try_from(n).map_err(|_| format!("{n} is out of range"))?;
                vcp::write(&mut self.bus, code, n).map_err(|e| e.to_string())
            }
        };
        // Show what the monitor holds now, also after a partial failure.
        let read = self.refresh();
        result.and(read)
    }

    fn reapply(&mut self) -> Result<(), String> {
        self.read_mode();
        let gpu = self.apply_gpu();
        self.refresh().and(gpu)
    }
}

/// A trial still running when the module goes (a scan, the core quitting) is reverted, never left in place.
impl<B: Bus> Drop for Monitor<B> {
    fn drop(&mut self) {
        if let Err(e) = self.end_trial(false) {
            crate::system::log::error(&format!("display mode revert: {e}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh folder per call, so one test's saved GPU colour never reaches another.
    fn dir() -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("sysvane-monitor-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn monitor() -> Monitor<vcp::Fake> {
        Monitor::new(
            1,
            "Display 2".into(),
            vcp::Fake::monitor(),
            Store::new(dir()),
        )
    }

    #[test]
    fn gpu_colour_is_written_saved_and_written_again() {
        let mut m = modes();
        assert_eq!(value(&m, ControlId::GpuBlue), Some(ControlValue::Int(100)));
        assert!(m.bus.ramps.is_empty(), "neutral: the ramp is left alone");
        m.set(ControlId::GpuBlue, ControlValue::Int(80)).unwrap();
        assert_eq!(m.bus.ramps.last().unwrap()[2][255], 52428);
        for bad in [
            ControlValue::Int(101),
            ControlValue::Int(-1),
            ControlValue::Bool(true),
        ] {
            assert!(m.set(ControlId::GpuRed, bad).is_err());
        }
        assert_eq!(m.bus.ramps.len(), 1);
        m.reapply().unwrap();
        assert_eq!(m.bus.ramps.len(), 2, "written again after sleep");

        let store = Store::new(m.store.file(""));
        let mut again = Monitor::new(1, "Display 2".into(), vcp::Fake::monitor(), store);
        again.bus.mode = m.bus.mode;
        again.start().unwrap();
        assert_eq!(
            value(&again, ControlId::GpuBlue),
            Some(ControlValue::Int(80))
        );
        assert_eq!(again.bus.ramps.len(), 1, "loaded and written at start");
        again
            .set(ControlId::GpuBlue, ControlValue::Int(100))
            .unwrap();
        assert_eq!(again.bus.ramps.last(), Some(&gamma::Gpu::default().ramp()));
    }

    #[test]
    fn an_orientation_is_a_trial_that_swaps_the_size() {
        let mut m = modes();
        m.trial_for = Duration::ZERO;
        m.set(ControlId::GpuRed, ControlValue::Int(90)).unwrap();
        m.set(ControlId::Orientation, ControlValue::Int(1)).unwrap();
        let fast: Mode = "2560x1440@165".parse().unwrap();
        let portrait: Mode = "1440x2560@165".parse().unwrap();
        assert_eq!((m.bus.mode, m.bus.turn), (Some(portrait), 1));
        assert_eq!(
            value(&m, ControlId::Orientation),
            Some(ControlValue::Int(1))
        );
        assert!(m.set(ControlId::Orientation, ControlValue::Int(4)).is_err());
        let ramps = m.bus.ramps.len();
        m.tick().unwrap();
        assert_eq!((m.bus.mode, m.bus.turn), (Some(fast), 0), "reverted");
        assert_eq!(
            m.bus.ramps.len(),
            ramps + 1,
            "colour written again after the change"
        );
    }

    fn value(m: &Monitor<vcp::Fake>, id: ControlId) -> Option<ControlValue> {
        m.controls()
            .into_iter()
            .find(|c| c.id == id)
            .map(|c| c.value)
    }

    #[test]
    fn nothing_is_shown_before_it_is_read() {
        let mut m = monitor();
        assert!(m.controls().is_empty());
        m.start().unwrap();
        assert_eq!(m.name(), "Display 2", "no model reported");
        assert_eq!(
            value(&m, ControlId::Brightness),
            Some(ControlValue::Int(70))
        );
        m.bus.values.clear();
        assert!(
            m.reapply().is_err(),
            "a monitor that stops answering is failed"
        );
    }

    #[test]
    fn a_setting_is_written_through_the_guard_and_read_back() {
        let mut m = monitor();
        m.start().unwrap();
        m.set(ControlId::Contrast, ControlValue::Int(40)).unwrap();
        assert_eq!(value(&m, ControlId::Contrast), Some(ControlValue::Int(40)));
        for bad in [
            (ControlId::Contrast, ControlValue::Int(101)),
            (ControlId::Contrast, ControlValue::Int(-1)),
            (ControlId::Contrast, ControlValue::Bool(true)),
            (ControlId::Level, ControlValue::Int(1)),
            (ControlId::Profile, ControlValue::Int(1)),
            (ControlId::Profile, ControlValue::Text("missing".into())),
        ] {
            assert!(m.set(bad.0, bad.1.clone()).is_err(), "{bad:?}");
        }
        assert_eq!(m.bus.writes, [(vcp::CONTRAST, 40)]);
    }

    #[test]
    fn a_calibrated_profile_reaches_the_monitor() {
        let mut m = monitor();
        m.bus.caps = Some("(model(CVTE)vcp(10 12 14(05 06 08 0B 0E) 16 18 1A B6))".into());
        m.start().unwrap();
        assert_eq!(m.name(), "CVTE · Display 2");
        assert_eq!(m.presets(), [5, 6, 8, 11], "14 is not an MCCS preset");
        m.set(
            ControlId::Profile,
            ControlValue::Text("Calibrated-Right".into()),
        )
        .unwrap();
        assert_eq!(
            m.bus.writes[..2],
            [(vcp::PICTURE_MODE, 3), (vcp::PRESET, 5)],
            "same model: picture mode too"
        );
        assert_eq!(value(&m, ControlId::RedGain), Some(ControlValue::Int(47)));
        assert_eq!(value(&m, ControlId::BlueGain), Some(ControlValue::Int(51)));
    }

    #[test]
    fn listed_codes_beyond_the_named_ones_become_controls() {
        let mut m = monitor();
        m.bus.caps = Some("(vcp(0C 10 12 14(05) 16 18 1A 59 60(0F) 62 B6 D6(01))".into());
        m.bus.values.insert(0x62, (25, 50));
        m.start().unwrap();
        let vcp: Vec<_> = m
            .controls()
            .into_iter()
            .filter(|c| matches!(c.id, ControlId::Vcp(_)))
            .collect();
        assert_eq!(
            vcp,
            [Control {
                id: ControlId::Vcp(0x62),
                value: ControlValue::Int(50)
            }],
            "0x59 listed but not answering, 0x0C/0x60/0xB6/0xD6 never offered"
        );
        m.set(ControlId::Vcp(0x62), ControlValue::Int(100)).unwrap();
        assert_eq!(m.bus.values[&0x62].0, 50);
        m.bus.values.insert(0x60, (15, 255));
        m.bus.values.insert(0x59, (50, 100));
        for code in [0x60, 0x0C, 0x6C] {
            assert!(
                m.set(ControlId::Vcp(code), ControlValue::Int(1)).is_err(),
                "0x{code:02X}"
            );
        }
        assert_eq!(m.bus.writes, [(0x62, 50)]);
    }

    #[test]
    fn controls_are_shown_as_a_percentage_of_the_maximum() {
        let mut m = monitor();
        m.bus.values.insert(vcp::BRIGHTNESS, (128, 255));
        m.start().unwrap();
        assert!(m.presets().is_empty(), "no capabilities");
        assert_eq!(
            value(&m, ControlId::Brightness),
            Some(ControlValue::Int(50))
        );
        m.set(ControlId::Brightness, ControlValue::Int(100))
            .unwrap();
        assert_eq!(m.bus.values[&vcp::BRIGHTNESS].0, 255);
    }

    fn modes() -> Monitor<vcp::Fake> {
        let mut m = monitor();
        let (fast, slow): (Mode, Mode) = (
            "2560x1440@165".parse().unwrap(),
            "1920x1080@60".parse().unwrap(),
        );
        m.bus.modes = vec![fast, slow];
        m.bus.mode = Some(fast);
        m.start().unwrap();
        m
    }

    #[test]
    fn a_display_mode_is_a_trial_until_kept() {
        let mut m = modes();
        assert_eq!(m.modes(), ["2560x1440@165", "1920x1080@60"]);
        assert!(
            m.set(
                ControlId::DisplayMode,
                ControlValue::Text("3840x2160@60".into())
            )
            .is_err(),
            "not listed"
        );
        assert!(
            m.set(ControlId::DisplayMode, ControlValue::Text("junk".into()))
                .is_err()
        );
        assert!(m.bus.mode_writes.is_empty());
        m.set(
            ControlId::DisplayMode,
            ControlValue::Text("1920x1080@60".into()),
        )
        .unwrap();
        assert!(m.wake().is_some(), "a trial asks for a tick");
        assert_eq!(
            value(&m, ControlId::KeepDisplayMode),
            Some(ControlValue::Int(15))
        );
        m.set(ControlId::KeepDisplayMode, ControlValue::Bool(true))
            .unwrap();
        let slow = "1920x1080@60".parse().unwrap();
        assert_eq!(
            m.bus.mode_writes,
            [(slow, false), (slow, true)],
            "saved only once kept"
        );
        assert!(m.wake().is_none() && value(&m, ControlId::KeepDisplayMode).is_none());
    }

    #[test]
    fn a_display_mode_not_kept_reverts_by_itself() {
        let mut m = modes();
        m.trial_for = Duration::ZERO;
        let fast: Mode = "2560x1440@165".parse().unwrap();
        m.set(
            ControlId::DisplayMode,
            ControlValue::Text("1920x1080@60".into()),
        )
        .unwrap();
        m.tick().unwrap();
        assert_eq!(m.bus.mode, Some(fast));
        assert_eq!(
            m.bus.mode_writes.last(),
            Some(&(fast, false)),
            "the saved mode is never touched"
        );
        assert!(m.wake().is_none());
    }
}
