use std::io::{Read, Cursor};
use std::sync::Arc;
use crate::format::error::{FormatError, Result};
use crate::core::store::PixelStore;
use crate::core::color::Color;
use crate::core::symmetry::{SymmetryConfig, SymmetryMode};
use crate::core::layer::{Layer, Chunk};
use crate::core::blend_mode::BlendMode;
use crate::core::layer::CHUNK_SIZE;
use crate::core::palette::Palette;
use rust_i18n::t;

pub fn serialize_canvas(store: &PixelStore, pan_x: f32, pan_y: f32, zoom_level: f64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(32);
    buf.extend_from_slice(&store.canvas_width.to_le_bytes());
    buf.extend_from_slice(&store.canvas_height.to_le_bytes());
    buf.push(store.primary_color.r);
    buf.push(store.primary_color.g);
    buf.push(store.primary_color.b);
    buf.push(store.primary_color.a);
    buf.extend_from_slice(&store.brush_size.to_le_bytes());
    buf.extend_from_slice(&pan_x.to_le_bytes());
    buf.extend_from_slice(&pan_y.to_le_bytes());
    buf.extend_from_slice(&zoom_level.to_le_bytes());
    buf
}

pub fn deserialize_canvas(data: &[u8], store: &mut PixelStore) -> Result<(f32, f32, f64)> {
    if data.len() < 32 {
        return Err(FormatError::UnexpectedEof(t!("error.payload_too_short", block = "CANV").to_string()));
    }

    store.canvas_width = u32::from_le_bytes(data[0..4].try_into().map_err(|_| FormatError::InvalidSliceLength)?);
    store.canvas_height = u32::from_le_bytes(data[4..8].try_into().map_err(|_| FormatError::InvalidSliceLength)?);
    store.primary_color = Color::new(data[8], data[9], data[10], data[11]);
    store.brush_size = u32::from_le_bytes(data[12..16].try_into().map_err(|_| FormatError::InvalidSliceLength)?);
    let pan_x = f32::from_le_bytes(data[16..20].try_into().map_err(|_| FormatError::InvalidSliceLength)?);
    let pan_y = f32::from_le_bytes(data[20..24].try_into().map_err(|_| FormatError::InvalidSliceLength)?);
    let zoom_level = f64::from_le_bytes(data[24..32].try_into().map_err(|_| FormatError::InvalidSliceLength)?);

    Ok((pan_x, pan_y, zoom_level))
}

fn sym_mode_to_u8(mode: SymmetryMode) -> u8 {
    match mode {
        SymmetryMode::None => 0,
        SymmetryMode::Horizontal => 1,
        SymmetryMode::Vertical => 2,
        SymmetryMode::Quad => 3,
        SymmetryMode::Translational => 4,
    }
}

fn u8_to_sym_mode(val: u8) -> SymmetryMode {
    match val {
        1 => SymmetryMode::Horizontal,
        2 => SymmetryMode::Vertical,
        3 => SymmetryMode::Quad,
        4 => SymmetryMode::Translational,
        _ => SymmetryMode::None,
    }
}

pub fn serialize_symmetry(sym: &SymmetryConfig) -> Vec<u8> {
    let mut buf = Vec::with_capacity(18);
    buf.push(sym_mode_to_u8(sym.mode));
    buf.extend_from_slice(&sym.axis_x.to_le_bytes());
    buf.extend_from_slice(&sym.axis_y.to_le_bytes());
    buf.extend_from_slice(&sym.translation_dx.to_le_bytes());
    buf.extend_from_slice(&sym.translation_dy.to_le_bytes());
    buf.push(if sym.visible_guides { 1 } else { 0 });
    buf
}

pub fn deserialize_symmetry(data: &[u8]) -> Result<SymmetryConfig> {
    if data.len() < 18 {
        return Err(FormatError::UnexpectedEof(t!("error.payload_too_short", block = "SYMM").to_string()));
    }

    Ok(SymmetryConfig {
        mode: u8_to_sym_mode(data[0]),
        axis_x: f32::from_le_bytes(data[1..5].try_into().map_err(|_| FormatError::InvalidSliceLength)?),
        axis_y: f32::from_le_bytes(data[5..9].try_into().map_err(|_| FormatError::InvalidSliceLength)?),
        translation_dx: i32::from_le_bytes(data[9..13].try_into().map_err(|_| FormatError::InvalidSliceLength)?),
        translation_dy: i32::from_le_bytes(data[13..17].try_into().map_err(|_| FormatError::InvalidSliceLength)?),
        visible_guides: data[17] != 0,
    })
}

pub fn serialize_layer(layer: &Layer) -> Vec<u8> {
    let id_bytes = layer.id.as_bytes();
    let name_bytes = layer.name.as_bytes();
    
    let mut buf = Vec::new();
    
    buf.extend_from_slice(&(id_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(id_bytes);
    
    buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(name_bytes);
    
    buf.push(if layer.visible { 1 } else { 0 });
    buf.push(if layer.locked { 1 } else { 0 });
    buf.push(layer.opacity);
    buf.push(layer.blend_mode.to_u8());
    
    buf.extend_from_slice(&layer.width.to_le_bytes());
    buf.extend_from_slice(&layer.height.to_le_bytes());
    buf.extend_from_slice(&layer.offset_x.to_le_bytes());
    buf.extend_from_slice(&layer.offset_y.to_le_bytes());
    
    buf.extend_from_slice(&(layer.chunks.len() as u32).to_le_bytes());
    
    for (&(cx, cy), chunk) in &layer.chunks {
        buf.extend_from_slice(&cx.to_le_bytes());
        buf.extend_from_slice(&cy.to_le_bytes());
        buf.extend_from_slice(&*chunk.data);
    }
    
    buf
}

pub fn deserialize_layer(data: &[u8], minor_version: u16) -> Result<Layer> {
    let mut cursor = Cursor::new(data);
    
    let read_u32 = |c: &mut Cursor<&[u8]>| -> Result<u32> {
        let mut b = [0u8; 4];
        c.read_exact(&mut b)?;
        Ok(u32::from_le_bytes(b))
    };
    
    let id_len = read_u32(&mut cursor)?;
    if id_len > 1024 { return Err(FormatError::InvalidData(t!("error.id_too_long").to_string())); }
    let mut id_buf = vec![0u8; id_len as usize];
    cursor.read_exact(&mut id_buf)?;
    let id = String::from_utf8(id_buf).map_err(|_| FormatError::InvalidUtf8(t!("error.invalid_utf8", msg = "ID").to_string()))?;
    
    let name_len = read_u32(&mut cursor)?;
    if name_len > 2048 { return Err(FormatError::InvalidData(t!("error.name_too_long").to_string())); }
    let mut name_buf = vec![0u8; name_len as usize];
    cursor.read_exact(&mut name_buf)?;
    let name = String::from_utf8(name_buf).map_err(|_| FormatError::InvalidUtf8(t!("error.invalid_utf8", msg = "Name").to_string()))?;
    
    let mut flags = [0u8; 2];
    cursor.read_exact(&mut flags)?;
    let opacity = if minor_version >= 1 {
        let mut op_buf = [0u8; 1];
        cursor.read_exact(&mut op_buf)?;
        op_buf[0]
    } else {
        255
    };

    let blend_mode = if minor_version >= 2 {
        let mut bm_buf = [0u8; 1];
        cursor.read_exact(&mut bm_buf)?;
        BlendMode::from_u8(bm_buf[0])
    } else { BlendMode::Normal };
    
    let width = read_u32(&mut cursor)?;
    let height = read_u32(&mut cursor)?;
    let mut offsets = [0u8; 8];
    cursor.read_exact(&mut offsets)?;
    let offset_x = i32::from_le_bytes(offsets[0..4].try_into().map_err(|_| FormatError::InvalidSliceLength)?);
    let offset_y = i32::from_le_bytes(offsets[4..8].try_into().map_err(|_| FormatError::InvalidSliceLength)?);
    
    let mut layer = Layer::new(id, name, width, height);
    layer.visible = flags[0] != 0;
    layer.locked = flags[1] != 0;
    layer.opacity = opacity;
    layer.blend_mode = blend_mode;
    layer.offset_x = offset_x;
    layer.offset_y = offset_y;
    
    let chunk_count = read_u32(&mut cursor)?;
    if chunk_count > 100_000 { return Err(FormatError::InvalidData(t!("error.too_many_chunks").to_string())); }
    let bytes_per_chunk = 8 + (CHUNK_SIZE * CHUNK_SIZE * 4) as u64;
    let total_required_bytes = chunk_count as u64 * bytes_per_chunk;
    let remaining_bytes = data.len() as u64 - cursor.position();

    if total_required_bytes > remaining_bytes {
        return Err(FormatError::UnexpectedEof(t!("error.payload_too_short", block = "LAYR_CHUNKS").to_string()));
    }
    for _ in 0..chunk_count {
        let cx = read_u32(&mut cursor)?;
        let cy = read_u32(&mut cursor)?;
        let mut chunk_data = Box::new([0u8; (CHUNK_SIZE * CHUNK_SIZE * 4) as usize]);
        cursor.read_exact(chunk_data.as_mut_slice())?;
        layer.chunks.insert((cx, cy), Chunk { data: Arc::from(chunk_data) });
    }
    
    Ok(layer)
}
pub fn serialize_palette(palette: &Palette) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + palette.colors.len() * 4);
    buf.extend_from_slice(&(palette.colors.len() as u32).to_le_bytes());
    for color in &palette.colors {
        buf.push(color.r);
        buf.push(color.g);
        buf.push(color.b);
        buf.push(color.a);
    }
    buf
}

pub fn deserialize_palette(data: &[u8]) -> Result<Palette> {
    if data.len() < 4 { return Err(FormatError::UnexpectedEof(t!("error.payload_too_short", block = "PALT").to_string())); }
    let count = u32::from_le_bytes(data[0..4].try_into().map_err(|_| FormatError::InvalidSliceLength)?) as usize;
    if count > 4096 { return Err(FormatError::InvalidData(t!("error.too_many_colors").to_string())); }
    if data.len() < 4 + count * 4 { return Err(FormatError::UnexpectedEof(t!("error.colors_incomplete").to_string())); }
    
    let mut colors = Vec::with_capacity(count);
    for i in 0..count {
        let offset = 4 + i * 4;
        colors.push(Color::new(data[offset], data[offset+1], data[offset+2], data[offset+3]));
    }
    Ok(Palette { name: t!("palette.project_palette").to_string(), colors })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_payload() {
        let mut original = PixelStore::new(1920, 1080);
        original.primary_color = Color::new(255, 128, 64, 255);
        original.brush_size = 15;
        let pan_x = -100.5;
        let pan_y = 200.25;
        let zoom_level = 3.1415;

        let bytes = serialize_canvas(&original, pan_x, pan_y, zoom_level);
        assert_eq!(bytes.len(), 32);

        let mut restored = PixelStore::new(1, 1);
        let (r_px, r_py, r_zl) = deserialize_canvas(&bytes, &mut restored).unwrap();

        assert_eq!(restored.canvas_width, 1920);
        assert_eq!(restored.canvas_height, 1080);
        assert_eq!(restored.primary_color, Color::new(255, 128, 64, 255));
        assert_eq!(restored.brush_size, 15);
        assert_eq!(r_px, -100.5);
        assert_eq!(r_py, 200.25);
        assert_eq!(r_zl, 3.1415);
    }

    #[test]
    fn test_symmetry_payload() {
        let mut original = SymmetryConfig::new(800, 600);
        original.mode = SymmetryMode::Quad;
        original.axis_x = 400.0;
        original.axis_y = 300.0;
        original.translation_dx = -50;
        original.translation_dy = 120;
        original.visible_guides = false;

        let bytes = serialize_symmetry(&original);
        assert_eq!(bytes.len(), 18);

        let restored = deserialize_symmetry(&bytes).unwrap();

        assert_eq!(restored.mode, SymmetryMode::Quad);
        assert_eq!(restored.axis_x, 400.0);
        assert_eq!(restored.axis_y, 300.0);
        assert_eq!(restored.translation_dx, -50);
        assert_eq!(restored.translation_dy, 120);
        assert_eq!(restored.visible_guides, false);
    }
    #[test]
    fn test_layer_payload() {
        let mut original = Layer::new("layer_001".into(), "测试图层".into(), 1920, 1080);
        original.visible = false;
        original.offset_x = -50;
        original.offset_y = 100;
        original.blend_mode = BlendMode::Multiply;
        
        original.set_pixel(10, 10, Color::new(255, 0, 0, 255)).unwrap();
        original.locked = true;

        let bytes = serialize_layer(&original);
        assert!(bytes.len() > 16000); 

        let restored = deserialize_layer(&bytes, 2).unwrap();
        
        assert_eq!(restored.id, "layer_001");
        assert_eq!(restored.name, "测试图层");
        assert_eq!(restored.visible, false);
        assert_eq!(restored.locked, true);
        assert_eq!(restored.blend_mode, BlendMode::Multiply);
        assert_eq!(restored.width, 1920);
        assert_eq!(restored.offset_x, -50);
        
        assert_eq!(restored.get_pixel(10, 10).unwrap().r, 255);
    }
}

pub fn serialize_selection(sel: &crate::core::selection::SelectionData) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(if sel.is_active { 1 } else { 0 });
    buf.extend_from_slice(&sel.width.to_le_bytes());
    buf.extend_from_slice(&sel.height.to_le_bytes());
    for &m in &sel.mask { buf.push(if m { 1 } else { 0 }); }
    buf
}

pub fn deserialize_selection(data: &[u8]) -> Result<crate::core::selection::SelectionData> {
    if data.len() < 9 { return Err(FormatError::UnexpectedEof("SELE 块太短".into())); }
    let is_active = data[0] != 0;
    let w = u32::from_le_bytes(data[1..5].try_into().map_err(|_| FormatError::InvalidSliceLength)?);
    let h = u32::from_le_bytes(data[5..9].try_into().map_err(|_| FormatError::InvalidSliceLength)?);
    
    if w > 16384 || h > 16384 {
        return Err(FormatError::InvalidData("Selection dimension exceeds safety limit".into()));
    }
    let mut sel = crate::core::selection::SelectionData::new(w, h);
    sel.is_active = is_active;
    for i in 0..(w * h) as usize {
        if i + 9 < data.len() { sel.mask[i] = data[i+9] != 0; }
    }
    Ok(sel)
}
