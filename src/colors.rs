//! Color palette and brightness adjustment.
//!
//! The palette is a hexad: six hues 60 degrees apart in HSL, at 0, 60, 120,
//! 180, 240 and 300 degrees. See ADR-001 and ADR-004.
//!
//! The six colors share the same perceptual lightness and chroma. Each color
//! has OKLCH lightness 0.80 and chroma 0.104. The chroma is the largest that
//! sRGB can show for every hue at that lightness; blue is the limit. See
//! ADR-003 and ADR-005.
//! red #F9A2A2 (0), yellow #C3C372 (60), green #94D094 (120),
//! cyan #5FD3D3 (180), blue #B5B5FF (240), magenta #E1A6E1 (300)

/// RGB color. Alpha is not handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Black, the usual color of a signature.
    pub const BLACK: Color = Color::rgb(0, 0, 0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Returns the color as `#RRGGBB`.
    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Returns the color with its lightness multiplied by `brightness_factor`.
    /// A factor of exactly 1.0 returns the color unchanged.
    pub fn with_brightness(self, brightness_factor: f64) -> Self {
        // The HSL round trip truncates and may shift a channel by one. A
        // factor of 1.0 must not alter the color, so skip the round trip.
        if brightness_factor == 1.0 {
            return self;
        }
        let (hue, saturation, brightness) = self.to_hsl();
        let brightness = (brightness * brightness_factor).clamp(0.0, 1.0);
        Self::from_hsl(hue, saturation, brightness)
    }

    /// Returns the color with its OKLCH lightness set to `lightness`.
    ///
    /// The hue stays the same. The chroma stays the same when sRGB can show
    /// it at the new lightness. Otherwise it is reduced until the color fits.
    /// `lightness` is clamped to 0.0..=1.0.
    pub fn with_oklch_lightness(self, lightness: f64) -> Self {
        let lightness = lightness.clamp(0.0, 1.0);
        let (_, a, b) = self.to_oklab();
        let chroma = a.hypot(b);
        let (unit_a, unit_b) = if chroma == 0.0 {
            (0.0, 0.0)
        } else {
            (a / chroma, b / chroma)
        };
        let fits = |c: f64| {
            Self::oklab_to_linear(lightness, c * unit_a, c * unit_b)
                .iter()
                .all(|&v| (-1e-6..=1.0 + 1e-6).contains(&v))
        };

        let chroma = if fits(chroma) {
            chroma
        } else {
            // The largest chroma that fits is found by bisection. A gray
            // (chroma 0) always fits, so the search has a valid lower bound.
            let (mut low, mut high) = (0.0, chroma);
            for _ in 0..40 {
                let mid = (low + high) / 2.0;
                if fits(mid) {
                    low = mid;
                } else {
                    high = mid;
                }
            }
            low
        };
        let linear = Self::oklab_to_linear(lightness, chroma * unit_a, chroma * unit_b);
        let to_byte = |v: f64| (Self::linear_to_srgb(v.clamp(0.0, 1.0)) * 255.0).round() as u8;
        Self::rgb(to_byte(linear[0]), to_byte(linear[1]), to_byte(linear[2]))
    }

    /// Returns the OKLab coordinates (L, a, b). OKLCH lightness is L and
    /// chroma is the length of (a, b).
    pub fn to_oklab(self) -> (f64, f64, f64) {
        let r = Self::srgb_to_linear(self.r as f64 / 255.0);
        let g = Self::srgb_to_linear(self.g as f64 / 255.0);
        let b = Self::srgb_to_linear(self.b as f64 / 255.0);
        let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
        let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
        let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
        (
            0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
            1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
            0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
        )
    }

    /// Converts OKLab to linear sRGB. Values outside 0.0..=1.0 mean the
    /// color is outside the sRGB gamut.
    fn oklab_to_linear(lightness: f64, a: f64, b: f64) -> [f64; 3] {
        let l = (lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
        let m = (lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
        let s = (lightness - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);
        [
            4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
            -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
            -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
        ]
    }

    fn srgb_to_linear(v: f64) -> f64 {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    fn linear_to_srgb(v: f64) -> f64 {
        if v <= 0.003_130_8 {
            12.92 * v
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    }

    /// Converts RGB to HSL. Returns (hue, saturation, lightness).
    fn to_hsl(self) -> (f64, f64, f64) {
        // The original divides in single precision before widening to double.
        // Keep the same precision so the results match.
        let r = (self.r as f32 / 255.0) as f64;
        let g = (self.g as f32 / 255.0) as f64;
        let b = (self.b as f32 / 255.0) as f64;

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;

        let brightness = (max + min) / 2.0;

        let (mut hue, saturation) = if delta == 0.0 {
            (0.0, 0.0)
        } else {
            let saturation = delta / (1.0 - (2.0 * brightness - 1.0).abs());
            let hue = if max == r {
                60.0 * (((g - b) / delta) % 6.0)
            } else if max == g {
                60.0 * (((b - r) / delta) + 2.0)
            } else {
                60.0 * (((r - g) / delta) + 4.0)
            };
            (hue, saturation)
        };

        if hue < 0.0 {
            hue += 360.0;
        }
        (hue, saturation, brightness)
    }

    fn from_hsl(hue: f64, saturation: f64, brightness: f64) -> Self {
        let c = (1.0 - (2.0 * brightness - 1.0).abs()) * saturation;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = brightness - c / 2.0;

        let (r_prime, g_prime, b_prime) = if (0.0..60.0).contains(&hue) {
            (c, x, 0.0)
        } else if (60.0..120.0).contains(&hue) {
            (x, c, 0.0)
        } else if (120.0..180.0).contains(&hue) {
            (0.0, c, x)
        } else if (180.0..240.0).contains(&hue) {
            (0.0, x, c)
        } else if (240.0..300.0).contains(&hue) {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        // The original truncates with a (byte) cast, so do not round here.
        Self::rgb(
            ((r_prime + m) * 255.0) as u8,
            ((g_prime + m) * 255.0) as u8,
            ((b_prime + m) * 255.0) as u8,
        )
    }
}

/// Named palette color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorName {
    Red,
    Blue,
    Green,
    Magenta,
    Cyan,
    Yellow,
}

impl ColorName {
    pub const ALL: [ColorName; 6] = [
        ColorName::Red,
        ColorName::Blue,
        ColorName::Green,
        ColorName::Magenta,
        ColorName::Cyan,
        ColorName::Yellow,
    ];

    pub const COUNT: u32 = Self::ALL.len() as u32;

    /// Picks a color by index. Out-of-range indexes wrap around by `COUNT`.
    pub fn from_index(index: u32) -> Self {
        Self::ALL[(index % Self::COUNT) as usize]
    }

    pub fn color(self) -> Color {
        match self {
            ColorName::Red => Color::rgb(0xF9, 0xA2, 0xA2),
            ColorName::Blue => Color::rgb(0xB5, 0xB5, 0xFF),
            ColorName::Green => Color::rgb(0x94, 0xD0, 0x94),
            ColorName::Magenta => Color::rgb(0xE1, 0xA6, 0xE1),
            ColorName::Cyan => Color::rgb(0x5F, 0xD3, 0xD3),
            ColorName::Yellow => Color::rgb(0xC3, 0xC3, 0x72),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_hex_is_uppercase_with_hash() {
        assert_eq!(ColorName::Red.color().to_hex(), "#F9A2A2");
    }

    #[test]
    fn with_brightness_with_factor_one_keeps_color() {
        for name in ColorName::ALL {
            let original = name.color();
            assert_eq!(original.with_brightness(1.0), original);
        }
    }

    #[test]
    fn with_brightness_darkens_and_lightens() {
        let original = ColorName::Green.color();
        let darker = original.with_brightness(0.9);
        let lighter = original.with_brightness(1.1);
        assert!(darker.g < original.g);
        assert!(lighter.r > original.r);
    }

    #[test]
    fn palette_shares_oklch_lightness() {
        for name in ColorName::ALL {
            let (lightness, _, _) = name.color().to_oklab();
            assert!((lightness - 0.80).abs() < 0.003, "{name:?} {lightness}");
        }
    }

    #[test]
    fn with_oklch_lightness_keeps_hue_and_reaches_the_lightness() {
        for name in ColorName::ALL {
            let original = name.color();
            let lighter = original.with_oklch_lightness(0.86);
            let (l0, a0, b0) = original.to_oklab();
            let (l1, a1, b1) = lighter.to_oklab();
            assert!(l1 > l0);
            assert!((l1 - 0.86).abs() < 0.005, "{name:?} {l1}");
            let hue_diff = (b1.atan2(a1) - b0.atan2(a0)).abs().to_degrees();
            assert!(hue_diff < 2.0, "{name:?} {hue_diff}");
        }
    }

    #[test]
    fn with_oklch_lightness_reduces_chroma_only_when_needed() {
        // Green at 0.86 fits sRGB, so its chroma stays. Blue does not fit.
        assert_eq!(
            ColorName::Green.color().with_oklch_lightness(0.86).to_hex(),
            "#A7E3A6"
        );
        assert_eq!(
            ColorName::Blue.color().with_oklch_lightness(0.86).to_hex(),
            "#CACCFF"
        );
    }

    #[test]
    fn with_oklch_lightness_keeps_gray_gray() {
        let gray = Color::rgb(128, 128, 128).with_oklch_lightness(0.9);
        assert_eq!(gray.r, gray.g);
        assert_eq!(gray.g, gray.b);
    }

    #[test]
    fn gray_has_no_hue() {
        let gray = Color::rgb(128, 128, 128);
        let (hue, saturation, _) = gray.to_hsl();
        assert_eq!(hue, 0.0);
        assert_eq!(saturation, 0.0);
    }
}
