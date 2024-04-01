type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    ParseError(String),
}


#[derive(Debug, PartialEq)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

fn hex_to_byte(hex: &str) -> Result<u8> {
    if hex.len() < 1 || hex.len() > 2 {
        return Err(Error::ParseError(hex.into()));
    }

    let result = hex.chars().map(|char| -> Result<u8> {
        let lower_char = char.to_ascii_lowercase();
        let num = "0123456789abcdef".find(lower_char).ok_or(Error::ParseError(hex.into()))?;
        Ok(num as u8)
    }).collect::<Result<Vec<_>>>()?;

    if result.len() == 1 {
        Ok(result[0] * 16 + result[0])
    } else {
        Ok(result[0] * 16 + result[1])
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { red: r, green: g, blue: b }
    }

    pub fn from_str(hex: &str) -> Result<Color> {
        let tmphex = if hex.starts_with("#") {
            &hex[1..]
        } else {
            &hex
        };

        let rgb = if tmphex.len() == 6 {
            Ok((&tmphex[0..2], &tmphex[2..4], &tmphex[4..6]))
        } else if tmphex.len() == 3 {
            Ok((&tmphex[0..1], &tmphex[1..2], &tmphex[2..3]))
        } else {
            Err(Error::ParseError(hex.into()))
        }?;

        let r = hex_to_byte(rgb.0)?;
        let g = hex_to_byte(rgb.1)?;
        let b = hex_to_byte(rgb.2)?;

        Ok(Color::new(r, g, b))
    }

    pub fn from_string(hex: &String) -> Result<Color> {
        Color::from_str(hex.as_str())
    }

    pub fn distance(&self, other: Color) -> f64 {
        let (r1, g1, b1) = (self.red as f64, self.green as f64, self.blue as f64);
        let (r2, g2, b2) = (other.red as f64, other.green as f64, other.blue as f64);
        f64::sqrt(
            ((r2 - r1) * 0.3).powi(2)
                + ((g2 - g1) * 0.59).powi(2)
                + ((b2 - b1) * 0.11).powi(2)
        )
    }

    pub fn wrap_ansi(&self, text: &str) -> String {
        let mut s = String::new();
        s.push_str(&format!("\x1b[38;2;{};{};{}m", self.red, self.green, self.blue));
        s.push_str(text);
        s.push_str("\x1b[0m");
        s
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_simple() {
        let color = Color::from_str("#Ff8000");
        assert!(color.is_ok(), "Failed parsing of color");
        assert_eq!(color.unwrap(), Color { red: 255, green: 128, blue: 0 });

        let color = Color::from_string(&"#Ff8000".into());
        assert!(color.is_ok(), "Failed parsing of color");
        assert_eq!(color.unwrap(), Color { red: 255, green: 128, blue: 0 });
    }

    #[test]
    fn test_color3_simple() {
        let color = Color::from_str("#f80");
        assert!(color.is_ok(), "Failed parsing of color");
        assert_eq!(color.unwrap(), Color { red: 255, green: 136, blue: 0 });

        let color = Color::from_string(&"#f80".into());
        assert!(color.is_ok(), "Failed parsing of color");
        assert_eq!(color.unwrap(), Color { red: 255, green: 136, blue: 0 });
    }

    #[test]
    fn test_color_distance() {
        let c1 = Color::from_str("#ff8000").unwrap();
        let c2 = Color::from_str("#fe8000").unwrap();
        let c3 = Color::from_str("#fa8000").unwrap();
        assert!(c1.distance(c2) < c1.distance(c3));
    }

    #[test]
    fn test_color_distance2() {
        let c1 = Color::from_str("#888888").unwrap();
        // change in blue
        let c2 = Color::from_str("#888877").unwrap();
        // change in green
        let c3 = Color::from_str("#887788").unwrap();

        // change in blue is perceived weaker than change in green, so distance is smaller
        assert!(c1.distance(c2) < c1.distance(c3));
    }
}
