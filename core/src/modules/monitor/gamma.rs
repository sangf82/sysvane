//! Colour through the GPU's gamma ramp, for any monitor and any GPU: brightness, contrast, gamma and a gain
//! per channel, as the GPU control panels offer them. Windows keeps a ramp only until a mode change, sleep or
//! another program sets its own, so the monitor module writes it again then. A ramp is written only once the
//! user has changed something, so Night Light and calibration tools keep the ramp while SysVane is unused.

use serde::{Deserialize, Serialize};

/// Each value 0 to 100, as the UI's sliders send it.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(default)]
pub struct Gpu {
    /// 50 is unchanged; each step adds or takes 1% of full scale.
    pub brightness: u16,
    /// 50 is unchanged; 0 halves the slope, 100 doubles it.
    pub contrast: u16,
    /// 50 is gamma 1.0; 0 is 0.5 (darker mid-tones), 100 is 2.0 (lighter).
    pub gamma: u16,
    /// 100 is unchanged; lower takes that channel down, as a white-point preset does.
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl Default for Gpu {
    fn default() -> Self {
        Self {
            brightness: 50,
            contrast: 50,
            gamma: 50,
            red: 100,
            green: 100,
            blue: 100,
        }
    }
}

pub type Ramp = [[u16; 256]; 3];

impl Gpu {
    pub fn is_neutral(&self) -> bool {
        *self == Self::default()
    }

    /// Values from disk or the UI above 100 are brought down to 100.
    pub fn clamped(mut self) -> Self {
        for v in [
            &mut self.brightness,
            &mut self.contrast,
            &mut self.gamma,
            &mut self.red,
            &mut self.green,
            &mut self.blue,
        ] {
            *v = (*v).min(100);
        }
        self
    }

    pub fn ramp(&self) -> Ramp {
        let step = |v: u16| 2f64.powf((f64::from(v) - 50.0) / 50.0);
        let (gamma, contrast) = (step(self.gamma), step(self.contrast));
        let brightness = (f64::from(self.brightness) - 50.0) / 100.0;
        let mut ramp = [[0; 256]; 3];
        for (channel, gain) in ramp.iter_mut().zip([self.red, self.green, self.blue]) {
            for (i, out) in channel.iter_mut().enumerate() {
                let x = (i as f64 / 255.0).powf(1.0 / gamma);
                let y = ((x - 0.5) * contrast + 0.5 + brightness) * f64::from(gain) / 100.0;
                *out = (y.clamp(0.0, 1.0) * 65535.0).round() as u16;
            }
        }
        ramp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_ramp_is_the_identity() {
        let ramp = Gpu::default().ramp();
        for channel in ramp {
            for (i, v) in channel.into_iter().enumerate() {
                assert_eq!(v, i as u16 * 257);
            }
        }
        assert!(Gpu::default().is_neutral());
    }

    #[test]
    fn each_setting_moves_the_ramp_the_way_it_says() {
        let mid = |g: Gpu, c: usize| g.ramp()[c][128];
        let base = mid(Gpu::default(), 0);
        let with = |f: fn(&mut Gpu)| {
            let mut g = Gpu::default();
            f(&mut g);
            g
        };
        assert!(mid(with(|g| g.brightness = 70), 0) > base);
        assert!(
            mid(with(|g| g.gamma = 80), 0) > base,
            "higher gamma lifts mid-tones"
        );
        assert_eq!(
            with(|g| g.contrast = 100).ramp()[0][0],
            0,
            "dark end clipped"
        );
        let warm = with(|g| g.blue = 80);
        assert_eq!(warm.ramp()[0][255], 65535);
        assert_eq!(warm.ramp()[2][255], 52428);
        assert!(!warm.is_neutral());
    }

    #[test]
    fn values_from_disk_are_clamped_and_missing_ones_are_neutral() {
        let g: Gpu = toml::from_str("red = 900\ngamma = 40").unwrap();
        let g = g.clamped();
        assert_eq!((g.red, g.gamma, g.blue, g.contrast), (100, 40, 100, 50));
    }
}
