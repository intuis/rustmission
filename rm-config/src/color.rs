use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

impl Color {
    #[must_use]
    pub const fn as_ratatui(&self) -> ratatui::style::Color {
        use ratatui::style::Color as RColor;
        use Color::*;
        match self {
            Red => RColor::Red,
            Green => RColor::Green,
            Blue => RColor::Blue,
            Yellow => RColor::Yellow,
            Magenta => RColor::Magenta,
            Cyan => RColor::Cyan,
            LightRed => RColor::LightRed,
            LightGreen => RColor::LightGreen,
            LightYellow => RColor::LightYellow,
            LightBlue => RColor::LightBlue,
            LightMagenta => RColor::LightMagenta,
            LightCyan => RColor::LightCyan,
            Black => RColor::Black,
            Gray => RColor::Gray,
            DarkGray => RColor::DarkGray,
            White => RColor::White,
            Rgb(r, g, b) => RColor::Rgb(*r, *g, *b),
            Indexed(u) => RColor::Indexed(*u),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::LightMagenta
    }
}

pub fn serialize<S: Serializer>(color: &Color, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&match color {
        Color::Red => "Red".to_string(),
        Color::Green => "Green".to_string(),
        Color::Black => "Black".to_string(),
        Color::Yellow => "Yellow".to_string(),
        Color::Blue => "Blue".to_string(),
        Color::Magenta => "Magenta".to_string(),
        Color::Cyan => "Cyan".to_string(),
        Color::Gray => "Gray".to_string(),
        Color::White => "White".to_string(),

        Color::DarkGray => "DarkGray".to_string(),
        Color::LightBlue => "LightBlue".to_string(),
        Color::LightCyan => "LightCyan".to_string(),
        Color::LightGreen => "LightGreen".to_string(),
        Color::LightMagenta => "LightMagenta".to_string(),
        Color::LightRed => "LightRed".to_string(),
        Color::LightYellow => "LightYellow".to_string(),
        Color::Indexed(index) => format!("{:03}", index),
        Color::Rgb(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
    })
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Color, D::Error> {
    use serde::de::{Error, Unexpected};

    let color_string = String::deserialize(deserializer)?;
    Ok(match color_string.to_lowercase().as_str() {
        "red" => Color::Red,
        "green" => Color::Green,
        "black" => Color::Black,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "white" => Color::White,

        "darkgray" => Color::DarkGray,
        "lightblue" => Color::LightBlue,
        "lightcyan" => Color::LightCyan,
        "lightgreen" => Color::LightGreen,
        "lightmagenta" => Color::LightMagenta,
        "lightred" => Color::LightRed,
        "lightyellow" => Color::LightYellow,
        _ => match color_string.len() {
            3 => {
                let index = color_string.parse::<u8>();
                if let Ok(index) = index {
                    Color::Indexed(index)
                } else {
                    return Err(Error::invalid_type(
                        Unexpected::Bytes(color_string.as_bytes()),
                        &"u8 index color",
                    ));
                }
            }
            4 | 7 => {
                if !color_string.starts_with('#') {
                    return Err(Error::invalid_value(
                        Unexpected::Char(color_string.chars().next().unwrap()),
                        &"# at the start",
                    ));
                }

                let color_string = color_string.trim_start_matches('#');

                let (r, g, b);

                match color_string.len() {
                    6 => {
                        r = u8::from_str_radix(&color_string[0..2], 16);
                        g = u8::from_str_radix(&color_string[2..4], 16);
                        b = u8::from_str_radix(&color_string[4..6], 16);
                    }
                    3 => {
                        r = u8::from_str_radix(&color_string[0..1], 16).map(|r| r * 17);
                        g = u8::from_str_radix(&color_string[1..2], 16).map(|g| g * 17);
                        b = u8::from_str_radix(&color_string[2..3], 16).map(|b| b * 17);
                    }
                    _ => unreachable!("Can't be reached since already checked"),
                }

                match (r, g, b) {
                    (Ok(r), Ok(g), Ok(b)) => Color::Rgb(r, g, b),
                    (_, _, _) => {
                        return Err(Error::invalid_value(
                            Unexpected::Bytes(color_string.as_bytes()),
                            &"hex color string",
                        ));
                    }
                }
            }
            _ => {
                return Err(serde::de::Error::invalid_length(
                    color_string.len(),
                    &"color string with length 4 or 7",
                ))
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::Color;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Test {
        #[serde(with = "super")]
        pub c: Color,
    }

    #[test]
    fn serialize_name() {
        let color: Color = Color::LightGreen;
        let t = Test { c: color };
        let color_string = serde_json::to_string(&t).unwrap();
        assert_eq!(color_string, r###"{"c":"LightGreen"}"###);
    }

    #[test]
    fn serialize_index() {
        let color: Color = Color::Indexed(123);
        let t = Test { c: color };
        let color_string = serde_json::to_string(&t).unwrap();
        assert_eq!(color_string, r###"{"c":"123"}"###);
    }

    #[test]
    fn serialize_hex() {
        let color: Color = Color::Rgb(18, 252, 28);
        let t = Test { c: color };
        let color_string = serde_json::to_string(&t).unwrap();
        assert_eq!(color_string, r###"{"c":"#12FC1C"}"###);
    }

    #[test]
    fn deserialize_name() {
        let color: Color = Color::LightGreen;
        let color_text = r###"{ "c": "LightGreen" }"###;
        let t: Test = serde_json::from_str::<Test>(color_text).unwrap();
        assert_eq!(t, Test { c: color });
    }

    #[test]
    fn deserialize_hex() {
        let color: Color = Color::Rgb(18, 252, 28);
        let color_text = r###"{ "c": "#12fc1c" }"###;
        let t: Test = serde_json::from_str::<Test>(color_text).unwrap();
        assert_eq!(t, Test { c: color });
    }

    #[test]
    fn deserialize_short_hex() {
        let color: Color = Color::Rgb(255, 255, 170);
        let color_text = r###"{ "c": "#FFA" }"###;
        let t: Test = serde_json::from_str::<Test>(color_text).unwrap();
        assert_eq!(t, Test { c: color });
    }

    #[test]
    fn deserialize_hex_and_short_hex() {
        let color_text_hex = r###"{ "c": "#FF99CC" }"###;
        let color_text_short_hex = r###"{ "c": "#F9C" }"###;
        let t_h: Test = serde_json::from_str::<Test>(color_text_hex).unwrap();
        let t_sh: Test = serde_json::from_str::<Test>(color_text_short_hex).unwrap();
        assert_eq!(t_h, t_sh);
    }

    #[test]
    fn deserialize_index() {
        let color: Color = Color::Indexed(123);
        let color_text = r###"{ "c": "123" }"###;
        let t: Test = serde_json::from_str::<Test>(color_text).unwrap();
        assert_eq!(t, Test { c: color });
    }
}
