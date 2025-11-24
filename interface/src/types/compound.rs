use bincode::{Decode, Encode};
use derive_more::Display;
use std::{
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Display, Default, Encode, Decode)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("#{:02x}{:02x}{:02x}", (self.r * 255.0) as u8, (self.g * 255.0) as u8, (self.b * 255.0) as u8)]
pub struct IglooColor {
    /// 0.0 to 1.0
    pub r: f64,
    /// 0.0 to 1.0
    pub g: f64,
    /// 0.0 to 1.0
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Display, Default, Encode, Decode)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("{year:04}-{month:02}-{day:02}")]
pub struct IglooDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Display, Default, Encode, Decode)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("{hour:02}:{minute:02}:{second:02}")]
pub struct IglooTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum ColorParseError {
    #[error("Invalid hex color: {0}")]
    InvalidHex(String),
    #[error("Unknown color name: {0}")]
    UnknownName(String),
    #[error("Invalid CSS function format: {0}")]
    InvalidCssFunction(String),
    #[error("Invalid number in color: {0}")]
    InvalidNumber(String),
    #[error("Unrecognized color format: {0}")]
    UnrecognizedFormat(String),
}

#[derive(Debug, thiserror::Error)]
pub enum DateParseError {
    #[error("Invalid date format: expected 2003-10-17, 10/17/2003, or \"October 17th 2003\"")]
    InvalidFormat,
    #[error("Invalid date component: {0}")]
    InvalidComponent(String),
    #[error("Invalid date: {year}-{month:02}-{day:02}")]
    InvalidDate { year: u16, month: u8, day: u8 },
    #[error("Unknown month name: {0}")]
    UnknownMonth(String),
}

#[derive(Debug, thiserror::Error)]
pub enum TimeParseError {
    #[error("Invalid time format: expected HH:MM:SS, HH:MM, or HH:MM PM")]
    InvalidFormat,
    #[error("Invalid time component: {0}")]
    InvalidComponent(String),
    #[error("Invalid time: {hour:02}:{minute:02}:{second:02}")]
    InvalidTime { hour: u8, minute: u8, second: u8 },
}

impl FromStr for IglooColor {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if (trimmed.starts_with('#') || trimmed.len() == 6)
            && let Ok(color) = Self::from_hex(trimmed)
        {
            return Ok(color);
        }

        if trimmed.starts_with("rgb") {
            return Self::from_css_rgb(trimmed);
        }
        if trimmed.starts_with("hsl") {
            return Self::from_css_hsl(trimmed);
        }

        if let Ok(color) = Self::from_name(trimmed) {
            return Ok(color);
        }

        Err(ColorParseError::UnrecognizedFormat(s.to_string()))
    }
}

impl FromStr for IglooTime {
    type Err = TimeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        let lower = trimmed.to_lowercase();
        if lower.contains("am") || lower.contains("pm") {
            return Self::parse_12hour(trimmed);
        }

        Self::parse_24hour(trimmed)
    }
}

impl IglooColor {
    pub fn from_rgb(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
        }
    }

    pub fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }

    pub fn from_hex(hex: &str) -> Result<Self, ColorParseError> {
        let s = hex.trim().trim_start_matches('#');
        if s.len() != 6 {
            return Err(ColorParseError::InvalidHex(hex.to_string()));
        }
        Ok(Self::from_rgb_u8(
            u8::from_str_radix(&s[0..2], 16)
                .map_err(|_| ColorParseError::InvalidHex(hex.to_string()))?,
            u8::from_str_radix(&s[2..4], 16)
                .map_err(|_| ColorParseError::InvalidHex(hex.to_string()))?,
            u8::from_str_radix(&s[4..6], 16)
                .map_err(|_| ColorParseError::InvalidHex(hex.to_string()))?,
        ))
    }

    pub fn from_name(name: &str) -> Result<Self, ColorParseError> {
        let normalized = name.trim().to_lowercase();
        match normalized.as_str() {
            "black" => Ok(Self::from_rgb_u8(0, 0, 0)),
            "white" => Ok(Self::from_rgb_u8(255, 255, 255)),
            "red" => Ok(Self::from_rgb_u8(255, 0, 0)),
            "lime" => Ok(Self::from_rgb_u8(0, 255, 0)),
            "blue" => Ok(Self::from_rgb_u8(0, 0, 255)),
            "yellow" => Ok(Self::from_rgb_u8(255, 255, 0)),
            "cyan" | "aqua" => Ok(Self::from_rgb_u8(0, 255, 255)),
            "magenta" | "fuchsia" => Ok(Self::from_rgb_u8(255, 0, 255)),
            "silver" => Ok(Self::from_rgb_u8(192, 192, 192)),
            "gray" | "grey" => Ok(Self::from_rgb_u8(128, 128, 128)),
            "maroon" => Ok(Self::from_rgb_u8(128, 0, 0)),
            "olive" => Ok(Self::from_rgb_u8(128, 128, 0)),
            "green" => Ok(Self::from_rgb_u8(0, 128, 0)),
            "purple" => Ok(Self::from_rgb_u8(128, 0, 128)),
            "teal" => Ok(Self::from_rgb_u8(0, 128, 128)),
            "navy" => Ok(Self::from_rgb_u8(0, 0, 128)),
            "aliceblue" => Ok(Self::from_rgb_u8(240, 248, 255)),
            "antiquewhite" => Ok(Self::from_rgb_u8(250, 235, 215)),
            "aquamarine" => Ok(Self::from_rgb_u8(127, 255, 212)),
            "azure" => Ok(Self::from_rgb_u8(240, 255, 255)),
            "beige" => Ok(Self::from_rgb_u8(245, 245, 220)),
            "bisque" => Ok(Self::from_rgb_u8(255, 228, 196)),
            "blanchedalmond" => Ok(Self::from_rgb_u8(255, 235, 205)),
            "blueviolet" => Ok(Self::from_rgb_u8(138, 43, 226)),
            "brown" => Ok(Self::from_rgb_u8(165, 42, 42)),
            "burlywood" => Ok(Self::from_rgb_u8(222, 184, 135)),
            "cadetblue" => Ok(Self::from_rgb_u8(95, 158, 160)),
            "chartreuse" => Ok(Self::from_rgb_u8(127, 255, 0)),
            "chocolate" => Ok(Self::from_rgb_u8(210, 105, 30)),
            "coral" => Ok(Self::from_rgb_u8(255, 127, 80)),
            "cornflowerblue" => Ok(Self::from_rgb_u8(100, 149, 237)),
            "cornsilk" => Ok(Self::from_rgb_u8(255, 248, 220)),
            "crimson" => Ok(Self::from_rgb_u8(220, 20, 60)),
            "darkblue" => Ok(Self::from_rgb_u8(0, 0, 139)),
            "darkcyan" => Ok(Self::from_rgb_u8(0, 139, 139)),
            "darkgoldenrod" => Ok(Self::from_rgb_u8(184, 134, 11)),
            "darkgray" | "darkgrey" => Ok(Self::from_rgb_u8(169, 169, 169)),
            "darkgreen" => Ok(Self::from_rgb_u8(0, 100, 0)),
            "darkkhaki" => Ok(Self::from_rgb_u8(189, 183, 107)),
            "darkmagenta" => Ok(Self::from_rgb_u8(139, 0, 139)),
            "darkolivegreen" => Ok(Self::from_rgb_u8(85, 107, 47)),
            "darkorange" => Ok(Self::from_rgb_u8(255, 140, 0)),
            "darkorchid" => Ok(Self::from_rgb_u8(153, 50, 204)),
            "darkred" => Ok(Self::from_rgb_u8(139, 0, 0)),
            "darksalmon" => Ok(Self::from_rgb_u8(233, 150, 122)),
            "darkseagreen" => Ok(Self::from_rgb_u8(143, 188, 143)),
            "darkslateblue" => Ok(Self::from_rgb_u8(72, 61, 139)),
            "darkslategray" | "darkslategrey" => Ok(Self::from_rgb_u8(47, 79, 79)),
            "darkturquoise" => Ok(Self::from_rgb_u8(0, 206, 209)),
            "darkviolet" => Ok(Self::from_rgb_u8(148, 0, 211)),
            "deeppink" => Ok(Self::from_rgb_u8(255, 20, 147)),
            "deepskyblue" => Ok(Self::from_rgb_u8(0, 191, 255)),
            "dimgray" | "dimgrey" => Ok(Self::from_rgb_u8(105, 105, 105)),
            "dodgerblue" => Ok(Self::from_rgb_u8(30, 144, 255)),
            "firebrick" => Ok(Self::from_rgb_u8(178, 34, 34)),
            "floralwhite" => Ok(Self::from_rgb_u8(255, 250, 240)),
            "forestgreen" => Ok(Self::from_rgb_u8(34, 139, 34)),
            "gainsboro" => Ok(Self::from_rgb_u8(220, 220, 220)),
            "ghostwhite" => Ok(Self::from_rgb_u8(248, 248, 255)),
            "gold" => Ok(Self::from_rgb_u8(255, 215, 0)),
            "goldenrod" => Ok(Self::from_rgb_u8(218, 165, 32)),
            "greenyellow" => Ok(Self::from_rgb_u8(173, 255, 47)),
            "honeydew" => Ok(Self::from_rgb_u8(240, 255, 240)),
            "hotpink" => Ok(Self::from_rgb_u8(255, 105, 180)),
            "indianred" => Ok(Self::from_rgb_u8(205, 92, 92)),
            "indigo" => Ok(Self::from_rgb_u8(75, 0, 130)),
            "ivory" => Ok(Self::from_rgb_u8(255, 255, 240)),
            "khaki" => Ok(Self::from_rgb_u8(240, 230, 140)),
            "lavender" => Ok(Self::from_rgb_u8(230, 230, 250)),
            "lavenderblush" => Ok(Self::from_rgb_u8(255, 240, 245)),
            "lawngreen" => Ok(Self::from_rgb_u8(124, 252, 0)),
            "lemonchiffon" => Ok(Self::from_rgb_u8(255, 250, 205)),
            "lightblue" => Ok(Self::from_rgb_u8(173, 216, 230)),
            "lightcoral" => Ok(Self::from_rgb_u8(240, 128, 128)),
            "lightcyan" => Ok(Self::from_rgb_u8(224, 255, 255)),
            "lightgoldenrodyellow" => Ok(Self::from_rgb_u8(250, 250, 210)),
            "lightgray" | "lightgrey" => Ok(Self::from_rgb_u8(211, 211, 211)),
            "lightgreen" => Ok(Self::from_rgb_u8(144, 238, 144)),
            "lightpink" => Ok(Self::from_rgb_u8(255, 182, 193)),
            "lightsalmon" => Ok(Self::from_rgb_u8(255, 160, 122)),
            "lightseagreen" => Ok(Self::from_rgb_u8(32, 178, 170)),
            "lightskyblue" => Ok(Self::from_rgb_u8(135, 206, 250)),
            "lightslategray" | "lightslategrey" => Ok(Self::from_rgb_u8(119, 136, 153)),
            "lightsteelblue" => Ok(Self::from_rgb_u8(176, 196, 222)),
            "lightyellow" => Ok(Self::from_rgb_u8(255, 255, 224)),
            "limegreen" => Ok(Self::from_rgb_u8(50, 205, 50)),
            "linen" => Ok(Self::from_rgb_u8(250, 240, 230)),
            "mediumaquamarine" => Ok(Self::from_rgb_u8(102, 205, 170)),
            "mediumblue" => Ok(Self::from_rgb_u8(0, 0, 205)),
            "mediumorchid" => Ok(Self::from_rgb_u8(186, 85, 211)),
            "mediumpurple" => Ok(Self::from_rgb_u8(147, 112, 219)),
            "mediumseagreen" => Ok(Self::from_rgb_u8(60, 179, 113)),
            "mediumslateblue" => Ok(Self::from_rgb_u8(123, 104, 238)),
            "mediumspringgreen" => Ok(Self::from_rgb_u8(0, 250, 154)),
            "mediumturquoise" => Ok(Self::from_rgb_u8(72, 209, 204)),
            "mediumvioletred" => Ok(Self::from_rgb_u8(199, 21, 133)),
            "midnightblue" => Ok(Self::from_rgb_u8(25, 25, 112)),
            "mintcream" => Ok(Self::from_rgb_u8(245, 255, 250)),
            "mistyrose" => Ok(Self::from_rgb_u8(255, 228, 225)),
            "moccasin" => Ok(Self::from_rgb_u8(255, 228, 181)),
            "navajowhite" => Ok(Self::from_rgb_u8(255, 222, 173)),
            "oldlace" => Ok(Self::from_rgb_u8(253, 245, 230)),
            "olivedrab" => Ok(Self::from_rgb_u8(107, 142, 35)),
            "orange" => Ok(Self::from_rgb_u8(255, 165, 0)),
            "orangered" => Ok(Self::from_rgb_u8(255, 69, 0)),
            "orchid" => Ok(Self::from_rgb_u8(218, 112, 214)),
            "palegoldenrod" => Ok(Self::from_rgb_u8(238, 232, 170)),
            "palegreen" => Ok(Self::from_rgb_u8(152, 251, 152)),
            "paleturquoise" => Ok(Self::from_rgb_u8(175, 238, 238)),
            "palevioletred" => Ok(Self::from_rgb_u8(219, 112, 147)),
            "papayawhip" => Ok(Self::from_rgb_u8(255, 239, 213)),
            "peachpuff" => Ok(Self::from_rgb_u8(255, 218, 185)),
            "peru" => Ok(Self::from_rgb_u8(205, 133, 63)),
            "pink" => Ok(Self::from_rgb_u8(255, 192, 203)),
            "plum" => Ok(Self::from_rgb_u8(221, 160, 221)),
            "powderblue" => Ok(Self::from_rgb_u8(176, 224, 230)),
            "rosybrown" => Ok(Self::from_rgb_u8(188, 143, 143)),
            "royalblue" => Ok(Self::from_rgb_u8(65, 105, 225)),
            "saddlebrown" => Ok(Self::from_rgb_u8(139, 69, 19)),
            "salmon" => Ok(Self::from_rgb_u8(250, 128, 114)),
            "sandybrown" => Ok(Self::from_rgb_u8(244, 164, 96)),
            "seagreen" => Ok(Self::from_rgb_u8(46, 139, 87)),
            "seashell" => Ok(Self::from_rgb_u8(255, 245, 238)),
            "sienna" => Ok(Self::from_rgb_u8(160, 82, 45)),
            "skyblue" => Ok(Self::from_rgb_u8(135, 206, 235)),
            "slateblue" => Ok(Self::from_rgb_u8(106, 90, 205)),
            "slategray" | "slategrey" => Ok(Self::from_rgb_u8(112, 128, 144)),
            "snow" => Ok(Self::from_rgb_u8(255, 250, 250)),
            "springgreen" => Ok(Self::from_rgb_u8(0, 255, 127)),
            "steelblue" => Ok(Self::from_rgb_u8(70, 130, 180)),
            "tan" => Ok(Self::from_rgb_u8(210, 180, 140)),
            "thistle" => Ok(Self::from_rgb_u8(216, 191, 216)),
            "tomato" => Ok(Self::from_rgb_u8(255, 99, 71)),
            "turquoise" => Ok(Self::from_rgb_u8(64, 224, 208)),
            "violet" => Ok(Self::from_rgb_u8(238, 130, 238)),
            "wheat" => Ok(Self::from_rgb_u8(245, 222, 179)),
            "whitesmoke" => Ok(Self::from_rgb_u8(245, 245, 245)),
            "yellowgreen" => Ok(Self::from_rgb_u8(154, 205, 50)),

            _ => Err(ColorParseError::UnknownName(name.to_string())),
        }
    }

    pub fn from_css_rgb(css: &str) -> Result<Self, ColorParseError> {
        let s = css.trim();
        if !s.starts_with("rgb(") || !s.ends_with(')') {
            return Err(ColorParseError::InvalidCssFunction(css.to_string()));
        }

        let inner = &s[4..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

        if parts.len() != 3 {
            return Err(ColorParseError::InvalidCssFunction(css.to_string()));
        }

        let r = parts[0]
            .parse::<u8>()
            .map_err(|_| ColorParseError::InvalidNumber(parts[0].to_string()))?;
        let g = parts[1]
            .parse::<u8>()
            .map_err(|_| ColorParseError::InvalidNumber(parts[1].to_string()))?;
        let b = parts[2]
            .parse::<u8>()
            .map_err(|_| ColorParseError::InvalidNumber(parts[2].to_string()))?;

        Ok(Self::from_rgb_u8(r, g, b))
    }

    pub fn from_css_hsl(css: &str) -> Result<Self, ColorParseError> {
        let s = css.trim();
        if !s.starts_with("hsl(") || !s.ends_with(')') {
            return Err(ColorParseError::InvalidCssFunction(css.to_string()));
        }

        let inner = &s[4..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

        if parts.len() != 3 {
            return Err(ColorParseError::InvalidCssFunction(css.to_string()));
        }

        let h = parts[0]
            .parse::<f64>()
            .map_err(|_| ColorParseError::InvalidNumber(parts[0].to_string()))?;

        let s_val = parts[1]
            .trim_end_matches('%')
            .parse::<f64>()
            .map_err(|_| ColorParseError::InvalidNumber(parts[1].to_string()))?
            / 100.0;

        let l = parts[2]
            .trim_end_matches('%')
            .parse::<f64>()
            .map_err(|_| ColorParseError::InvalidNumber(parts[2].to_string()))?
            / 100.0;

        Ok(Self::from_hsl(h, s_val, l))
    }

    pub fn from_hsl(h: f64, s: f64, l: f64) -> Self {
        let h = h % 360.0;
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match h as i32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self {
            r: (r + m).clamp(0.0, 1.0),
            g: (g + m).clamp(0.0, 1.0),
            b: (b + m).clamp(0.0, 1.0),
        }
    }

    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        let h = h % 360.0;
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h as i32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self {
            r: (r + m).clamp(0.0, 1.0),
            g: (g + m).clamp(0.0, 1.0),
            b: (b + m).clamp(0.0, 1.0),
        }
    }

    pub fn to_rgb_u8(&self) -> (u8, u8, u8) {
        (
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        )
    }

    pub fn to_hsl(&self) -> (f64, f64, f64) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta == 0.0 {
            return (0.0, 0.0, l);
        }

        let s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let h = if max == self.r {
            60.0 * (((self.g - self.b) / delta) % 6.0)
        } else if max == self.g {
            60.0 * (((self.b - self.r) / delta) + 2.0)
        } else {
            60.0 * (((self.r - self.g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };

        (h, s, l)
    }

    pub fn to_hsv(&self) -> (f64, f64, f64) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;

        let v = max;

        if delta == 0.0 {
            return (0.0, 0.0, v);
        }

        let s = delta / max;

        let h = if max == self.r {
            60.0 * (((self.g - self.b) / delta) % 6.0)
        } else if max == self.g {
            60.0 * (((self.b - self.r) / delta) + 2.0)
        } else {
            60.0 * (((self.r - self.g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };

        (h, s, v)
    }

    pub fn blend(&self, other: &Self, ratio: f64) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);
        Self {
            r: self.r * (1.0 - ratio) + other.r * ratio,
            g: self.g * (1.0 - ratio) + other.g * ratio,
            b: self.b * (1.0 - ratio) + other.b * ratio,
        }
    }

    pub fn lighten(&self, amount: f64) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, s, (l + amount).clamp(0.0, 1.0))
    }

    pub fn darken(&self, amount: f64) -> Self {
        self.lighten(-amount)
    }

    pub fn saturate(&self, amount: f64) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, (s + amount).clamp(0.0, 1.0), l)
    }

    pub fn desaturate(&self, amount: f64) -> Self {
        self.saturate(-amount)
    }

    pub fn invert(&self) -> Self {
        Self {
            r: 1.0 - self.r,
            g: 1.0 - self.g,
            b: 1.0 - self.b,
        }
    }

    pub fn grayscale(&self) -> Self {
        let gray = self.luminance();
        Self {
            r: gray,
            g: gray,
            b: gray,
        }
    }

    pub fn luminance(&self) -> f64 {
        const LUMA_R: f64 = 0.2126;
        const LUMA_G: f64 = 0.7152;
        const LUMA_B: f64 = 0.0722;
        (LUMA_R * self.r + LUMA_G * self.g + LUMA_B * self.b).clamp(0.0, 1.0)
    }

    pub fn hue_shift(&self, degrees: f64) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl((h + degrees) % 360.0, s, l)
    }
}

impl Add for IglooColor {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: (self.r + rhs.r).clamp(0.0, 1.0),
            g: (self.g + rhs.g).clamp(0.0, 1.0),
            b: (self.b + rhs.b).clamp(0.0, 1.0),
        }
    }
}

impl Sub for IglooColor {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            r: (self.r - rhs.r).clamp(0.0, 1.0),
            g: (self.g - rhs.g).clamp(0.0, 1.0),
            b: (self.b - rhs.b).clamp(0.0, 1.0),
        }
    }
}

impl Mul for IglooColor {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: (self.r * rhs.r).clamp(0.0, 1.0),
            g: (self.g * rhs.g).clamp(0.0, 1.0),
            b: (self.b * rhs.b).clamp(0.0, 1.0),
        }
    }
}

impl Mul<f64> for IglooColor {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            r: (self.r * rhs).clamp(0.0, 1.0),
            g: (self.g * rhs).clamp(0.0, 1.0),
            b: (self.b * rhs).clamp(0.0, 1.0),
        }
    }
}

impl Div for IglooColor {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            r: if rhs.r == 0.0 {
                0.0
            } else {
                (self.r / rhs.r).clamp(0.0, 1.0)
            },
            g: if rhs.g == 0.0 {
                0.0
            } else {
                (self.g / rhs.g).clamp(0.0, 1.0)
            },
            b: if rhs.b == 0.0 {
                0.0
            } else {
                (self.b / rhs.b).clamp(0.0, 1.0)
            },
        }
    }
}

impl Div<f64> for IglooColor {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs == 0.0 {
            Self {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            }
        } else {
            Self {
                r: (self.r / rhs).clamp(0.0, 1.0),
                g: (self.g / rhs).clamp(0.0, 1.0),
                b: (self.b / rhs).clamp(0.0, 1.0),
            }
        }
    }
}

impl FromStr for IglooDate {
    type Err = DateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if let Ok(date) = Self::parse_iso(trimmed) {
            return Ok(date);
        }
        if let Ok(date) = Self::parse_us_numeric(trimmed) {
            return Ok(date);
        }

        Self::parse_written(trimmed)
    }
}

impl IglooDate {
    const DAYS_IN_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    pub fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        let date = Self { year, month, day };
        if date.is_valid() { Some(date) } else { None }
    }

    pub fn is_valid(&self) -> bool {
        if self.month < 1 || self.month > 12 {
            return false;
        }
        if self.day < 1 {
            return false;
        }
        let max_day = if self.month == 2 && self.is_leap_year() {
            29
        } else {
            Self::DAYS_IN_MONTH[(self.month - 1) as usize]
        };
        self.day <= max_day
    }

    pub fn is_leap_year(&self) -> bool {
        Self::is_leap_year_value(self.year)
    }

    fn is_leap_year_value(year: u16) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    pub fn days_since_epoch(&self) -> i32 {
        let mut days = 0i32;

        for y in 0..self.year {
            days += if Self::is_leap_year_value(y) {
                366
            } else {
                365
            };
        }

        for m in 1..self.month {
            days += if m == 2 && self.is_leap_year() {
                29
            } else {
                Self::DAYS_IN_MONTH[(m - 1) as usize] as i32
            };
        }

        days + self.day as i32
    }

    pub fn from_days_since_epoch(mut days: i32) -> Self {
        let mut year = 0u16;

        loop {
            let year_days = if Self::is_leap_year_value(year) {
                366
            } else {
                365
            };
            if days <= year_days {
                break;
            }
            days -= year_days;
            year += 1;
        }

        let is_leap = Self::is_leap_year_value(year);

        let mut month = 1u8;
        for m in 1..=12 {
            let month_days = if m == 2 && is_leap {
                29
            } else {
                Self::DAYS_IN_MONTH[(m - 1) as usize] as i32
            };
            if days <= month_days {
                month = m;
                break;
            }
            days -= month_days;
        }

        Self {
            year,
            month,
            day: days as u8,
        }
    }

    pub fn add_days(&self, days: i32) -> Self {
        Self::from_days_since_epoch(self.days_since_epoch() + days)
    }

    pub fn days_between(&self, other: &Self) -> i32 {
        other.days_since_epoch() - self.days_since_epoch()
    }

    /// 0 = Sunday, 6 = Saturday
    pub fn day_of_week(&self) -> u8 {
        ((self.days_since_epoch() + 6) % 7) as u8
    }

    pub fn day_of_year(&self) -> u16 {
        let mut day = 0u16;
        for m in 1..self.month {
            day += if m == 2 && self.is_leap_year() {
                29
            } else {
                Self::DAYS_IN_MONTH[(m - 1) as usize] as u16
            };
        }
        day + self.day as u16
    }

    fn parse_iso(s: &str) -> Result<Self, DateParseError> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(DateParseError::InvalidFormat);
        }
        let year = parts[0]
            .parse()
            .map_err(|_| DateParseError::InvalidComponent(parts[0].to_string()))?;
        let month = parts[1]
            .parse()
            .map_err(|_| DateParseError::InvalidComponent(parts[1].to_string()))?;
        let day = parts[2]
            .parse()
            .map_err(|_| DateParseError::InvalidComponent(parts[2].to_string()))?;
        Self::new(year, month, day).ok_or(DateParseError::InvalidDate { year, month, day })
    }

    fn parse_us_numeric(s: &str) -> Result<Self, DateParseError> {
        let sep = if s.contains('/') { '/' } else { '-' };
        let parts: Vec<&str> = s.split(sep).collect();
        if parts.len() != 3 {
            return Err(DateParseError::InvalidFormat);
        }
        let month = parts[0]
            .parse()
            .map_err(|_| DateParseError::InvalidComponent(parts[0].to_string()))?;
        let day = parts[1]
            .parse()
            .map_err(|_| DateParseError::InvalidComponent(parts[1].to_string()))?;
        let year = parts[2]
            .parse()
            .map_err(|_| DateParseError::InvalidComponent(parts[2].to_string()))?;
        Self::new(year, month, day).ok_or(DateParseError::InvalidDate { year, month, day })
    }

    fn parse_month_name(name: &str) -> Result<u8, DateParseError> {
        let normalized = name.trim().to_lowercase();
        match normalized.as_str() {
            "january" | "jan" => Ok(1),
            "february" | "feb" => Ok(2),
            "march" | "mar" => Ok(3),
            "april" | "apr" => Ok(4),
            "may" => Ok(5),
            "june" | "jun" => Ok(6),
            "july" | "jul" => Ok(7),
            "august" | "aug" => Ok(8),
            "september" | "sep" | "sept" => Ok(9),
            "october" | "oct" => Ok(10),
            "november" | "nov" => Ok(11),
            "december" | "dec" => Ok(12),
            _ => Err(DateParseError::UnknownMonth(name.to_string())),
        }
    }

    fn parse_written(s: &str) -> Result<Self, DateParseError> {
        let cleaned = s.replace(',', "");
        let parts: Vec<&str> = cleaned.split_whitespace().collect();

        if parts.len() < 3 {
            return Err(DateParseError::InvalidFormat);
        }

        if let Ok(month) = Self::parse_month_name(parts[0]) {
            let day_str = parts[1]
                .trim_end_matches("st")
                .trim_end_matches("nd")
                .trim_end_matches("rd")
                .trim_end_matches("th");
            let day = day_str
                .parse()
                .map_err(|_| DateParseError::InvalidComponent(parts[1].to_string()))?;
            let year = parts[2]
                .parse()
                .map_err(|_| DateParseError::InvalidComponent(parts[2].to_string()))?;
            return Self::new(year, month, day).ok_or(DateParseError::InvalidDate {
                year,
                month,
                day,
            });
        }

        let day_str = parts[0]
            .trim_end_matches("st")
            .trim_end_matches("nd")
            .trim_end_matches("rd")
            .trim_end_matches("th");
        if let Ok(day) = day_str.parse::<u8>()
            && let Ok(month) = Self::parse_month_name(parts[1])
        {
            let year = parts[2]
                .parse()
                .map_err(|_| DateParseError::InvalidComponent(parts[2].to_string()))?;
            return Self::new(year, month, day).ok_or(DateParseError::InvalidDate {
                year,
                month,
                day,
            });
        }

        Err(DateParseError::InvalidFormat)
    }

    pub fn add_weeks(&self, weeks: i32) -> Self {
        self.add_days(weeks * 7)
    }

    pub fn add_months(&self, months: i32) -> Self {
        let mut year = self.year as i32;
        let mut month = self.month as i32 + months as i32;

        while month > 12 {
            month -= 12;
            year += 1;
        }
        while month < 1 {
            month += 12;
            year -= 1;
        }

        let max_day = if month == 2 && Self::is_leap_year_value(year as u16) {
            29
        } else {
            Self::DAYS_IN_MONTH[(month - 1) as usize]
        };
        let day = self.day.min(max_day);

        Self {
            year: year as u16,
            month: month as u8,
            day,
        }
    }

    pub fn add_years(&self, years: i16) -> Self {
        let mut result = *self;

        if years >= 0 {
            result.year = self.year.saturating_add(years as u16);
        } else {
            result.year = self.year.saturating_sub(years.unsigned_abs() as u16);
        }

        if result.month == 2 && result.day == 29 && !result.is_leap_year() {
            result.day = 28;
        }

        result
    }
}

impl IglooTime {
    pub fn new(hour: u8, minute: u8, second: u8) -> Option<Self> {
        let time = Self {
            hour,
            minute,
            second,
        };
        if time.is_valid() { Some(time) } else { None }
    }

    pub fn is_valid(&self) -> bool {
        self.hour < 24 && self.minute < 60 && self.second < 60
    }

    pub fn to_seconds(&self) -> i32 {
        self.hour as i32 * 3600 + self.minute as i32 * 60 + self.second as i32
    }

    pub fn from_seconds(mut secs: i32) -> Self {
        secs = secs.rem_euclid(86400);
        let hour = (secs / 3600) as u8;
        secs %= 3600;
        let minute = (secs / 60) as u8;
        let second = (secs % 60) as u8;
        Self {
            hour,
            minute,
            second,
        }
    }

    pub fn add_seconds(&self, secs: i32) -> Self {
        Self::from_seconds(self.to_seconds() + secs)
    }

    pub fn seconds_between(&self, other: &Self) -> i32 {
        other.to_seconds() - self.to_seconds()
    }

    fn parse_24hour(s: &str) -> Result<Self, TimeParseError> {
        let parts: Vec<&str> = s.split(':').collect();

        let hour = parts[0]
            .parse()
            .map_err(|_| TimeParseError::InvalidComponent(parts[0].to_string()))?;
        let minute = parts[1]
            .parse()
            .map_err(|_| TimeParseError::InvalidComponent(parts[1].to_string()))?;
        let second = if parts.len() == 3 {
            parts[2]
                .parse()
                .map_err(|_| TimeParseError::InvalidComponent(parts[2].to_string()))?
        } else {
            0
        };

        Self::new(hour, minute, second).ok_or(TimeParseError::InvalidTime {
            hour,
            minute,
            second,
        })
    }

    fn parse_12hour(s: &str) -> Result<Self, TimeParseError> {
        let lower = s.to_lowercase();
        let is_pm = lower.contains("pm");

        let time_part = lower.replace("am", "").replace("pm", "").trim().to_string();

        let parts: Vec<&str> = time_part.split(':').collect();

        let mut hour: u8 = parts[0]
            .parse()
            .map_err(|_| TimeParseError::InvalidComponent(parts[0].to_string()))?;
        let minute = parts[1]
            .parse()
            .map_err(|_| TimeParseError::InvalidComponent(parts[1].to_string()))?;
        let second = if parts.len() == 3 {
            parts[2]
                .parse()
                .map_err(|_| TimeParseError::InvalidComponent(parts[2].to_string()))?
        } else {
            0
        };

        if is_pm && hour != 12 {
            hour += 12;
        } else if !is_pm && hour == 12 {
            hour = 0;
        }

        Self::new(hour, minute, second).ok_or(TimeParseError::InvalidTime {
            hour,
            minute,
            second,
        })
    }

    pub fn add_minutes(&self, minutes: i32) -> Self {
        self.add_seconds(minutes * 60)
    }

    pub fn add_hours(&self, hours: i32) -> Self {
        self.add_seconds(hours * 3600)
    }
}
