use anyhow::{anyhow, Result};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color{
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn from_rgba(r: u8, g: u8, b:u8, a: u8)->Self{
        Self {r,g,b,a}
    }

    pub const fn from_rgb(r: u8, b: u8, g: u8) -> Self {
        Self::from_rgba(r, g, b, 255)
    }

    pub fn as_hsl(&self) -> HslColor {
         // Taken and converted from: https://stackoverflow.com/a/9493060
         let r = self.r as f64 / 255.0;
         let g = self.g as f64 / 255.0;
         let b = self.b as f64 / 255.0;
         let vmax = r.max(g.max(b));
         let vmin = r.min(g.min(b));
         let l = (vmax + vmin) / 2.0;

         if vmax == vmin {
             return HslColor::new(0.0, 0.0, l); // achromatic
         }

         let d = vmax - vmin;
         let s = if l > 0.5 {
             d / (2.0 - vmax - vmin)
         } else {
             d / (vmax + vmin)
         };

         let mut h = (vmax + vmin) / 2.0;

         if vmax == r {
             h = (g - b) / d;
             if g < b {
                 h += 6.0
             }
         }

         if vmax == g {
             h = (b - r) / d + 2.0;
         }

         if vmax == b {
             h = (r - g) / d + 4.0;
         }

         h /= 6.0;

         // The color conversion moves every value into the [0,1] number space.
         // But we want the hue in [0,360], s in [0,100] and l in [0,100]
         HslColor::new(h * 360f64, s * 100f64, l * 100f64)
     }
}

impl From<HslColor> for Color {
    fn from(v: HslColor) -> Self {
        // Taken and converted from: https://stackoverflow.com/a/9493060

        fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
            let mut t = t;
            if t < 0f64 {
                t += 1f64
            };
            if t > 1f64 {
                t -= 1f64
            };
            if t < 1f64 / 6f64 {
                return p + (q - p) * 6f64 * t;
            }
            if t < 1f64 / 2f64 {
                return q;
            }
            if t < 2f64 / 3f64 {
                return p + (q - p) * (2f64 / 3f64 - t) * 6f64;
            };
            return p;
        }

        let r;
        let g;
        let b;

        // The input for this algorithm expects all the h,s and l values in the
        // range [0,1].
        let h = v.h / 360f64;
        let s = v.s / 100f64;
        let l = v.l / 100f64;

        if s == 0.0 {
            r = l;
            g = l;
            b = l;
        } else {
            let q = if l < 0.5 {
                l * (1.0 + s)
            } else {
                l + s - l * s
            };
            let p = 2.0 * l - q;

            r = hue_to_rgb(p, q, h + 1f64 / 3f64);
            g = hue_to_rgb(p, q, h);
            b = hue_to_rgb(p, q, h - 1f64 / 3f64);
        }
        Color::from_rgb(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        )
    }
}

pub struct HslColor {
    pub h: f64, // Hue in [0,360]
    pub s: f64, // Saturation in [0,100]
    pub l: f64, // Lightness in [0,100]
}

impl HslColor {
    pub fn new(h: f64, s: f64, l: f64) -> Self {
        Self { h, s, l }
    }
}
