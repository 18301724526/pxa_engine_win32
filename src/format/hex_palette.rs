use crate::core::palette::Palette;
use crate::core::color::Color;
use std::io::{BufRead, Write};
use crate::format::error::Result;
use std::fs::File;
use std::path::Path;

pub fn save_to_hex(path: &Path, palette: &Palette) -> Result<()> {
    let mut file = File::create(path)?;
    for color in &palette.colors {
        writeln!(file, "{:02X}{:02X}{:02X}", color.r, color.g, color.b)?;
    }
    Ok(())
}

pub fn load_from_hex(path: &Path) -> Result<Palette> {
    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut colors = Vec::new();

    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.is_empty() { continue; }
        
        let hex_str = if line.starts_with('#') { &line[1..] } else { &line };
        
        if hex_str.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex_str[0..2], 16),
                u8::from_str_radix(&hex_str[2..4], 16),
                u8::from_str_radix(&hex_str[4..6], 16),
            ) {
                colors.push(Color::new(r, g, b, 255));
            }
        }
    }
    
    let name = path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
        
    Ok(Palette { name, colors })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_hex_palette_io() {
        let mut palette = Palette::new();
        palette.name = "TestPalette".into();
        palette.colors = vec![
            Color::new(255, 0, 0, 255),
            Color::new(0, 255, 0, 255),
            Color::new(0, 0, 255, 255),
        ];

        let mut path = env::temp_dir();
        path.push("test_palette.hex");

        save_to_hex(&path, &palette).expect("保存失败");
        
        let loaded = load_from_hex(&path).expect("加载失败");
        assert_eq!(loaded.name, "test_palette", "名称应回退为无后缀的文件名");
        assert_eq!(loaded.colors.len(), 3);
        assert_eq!(loaded.colors[0].r, 255);

        let _ = std::fs::remove_file(path);
    }
}