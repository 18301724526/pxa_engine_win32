use super::color::Color;
use super::blend_mode::BlendMode;
use std::collections::HashMap;
use std::sync::Arc;
use crate::core::error::{CoreError, Result};

pub const CHUNK_SIZE: u32 = 64;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub data: Arc<[u8; (CHUNK_SIZE * CHUNK_SIZE * 4) as usize]>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            data: Arc::new([0; (CHUNK_SIZE * CHUNK_SIZE * 4) as usize]),
        }
    }
    pub fn data_mut(&mut self) -> &mut [u8; (CHUNK_SIZE * CHUNK_SIZE * 4) as usize] {
        Arc::make_mut(&mut self.data)
    }
    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|&b| b == 0)
    }
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub width: u32,
    pub height: u32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub anim_offset_x: i32,
    pub anim_offset_y: i32,
    pub opacity: u8,
    pub blend_mode: BlendMode,
    pub chunks: HashMap<(u32, u32), Chunk>,
    pub version: u64,
}

impl Layer {
    pub fn new(id: String, name: String, width: u32, height: u32) -> Self {
        Self {
            id,
            name,
            visible: true,
            locked: false,
            width,
            height,
            offset_x: 0,
            offset_y: 0,
            anim_offset_x: 0,
            anim_offset_y: 0,
            opacity: 255,
            blend_mode: BlendMode::Normal,
            chunks: HashMap::new(),
            version: 0,
        }
    }

    pub fn chunks_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let cx = x / CHUNK_SIZE;
        let cy = y / CHUNK_SIZE;
        
        if let Some(chunk) = self.chunks.get(&(cx, cy)) {
            let lx = x % CHUNK_SIZE;
            let ly = y % CHUNK_SIZE;
            let idx = ((ly * CHUNK_SIZE + lx) * 4) as usize;
            Some(Color::new(
                chunk.data[idx],
                chunk.data[idx + 1],
                chunk.data[idx + 2],
                chunk.data[idx + 3],
            ))
        } else {
            Some(Color::transparent())
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) -> Result<()> {
        if self.locked {
            return Err(CoreError::LayerLocked);
        }
        self.set_pixel_raw(x, y, color)
    }

    pub fn set_pixel_raw(&mut self, x: u32, y: u32, color: Color) -> Result<()> {
        if x >= self.width || y >= self.height {
            return Err(CoreError::OutOfBounds { x, y });
        }
        
        let cx = x / CHUNK_SIZE;
        let cy = y / CHUNK_SIZE;
        let lx = x % CHUNK_SIZE;
        let ly = y % CHUNK_SIZE;
        let idx = ((ly * CHUNK_SIZE + lx) * 4) as usize;

        if color.a == 0 && !self.chunks.contains_key(&(cx, cy)) {
            return Ok(());
        }

        let chunk = self.chunks.entry((cx, cy)).or_insert_with(Chunk::new);
        
        let data = chunk.data_mut();
        self.version += 1;
        data[idx] = color.r;
        data[idx + 1] = color.g;
        data[idx + 2] = color.b;
        data[idx + 3] = color.a;
        Ok(())
    }
    pub fn prune_empty_chunks(&mut self) {
        self.chunks.retain(|_, chunk| !chunk.is_empty());
    }

    pub fn get_rect_data(&self, x: u32, y: u32, w: u32, h: u32) -> Vec<u8> {
        let mut buffer = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            for col in 0..w {
                let color = self.get_pixel(x + col, y + row).unwrap_or(Color::transparent());
                let idx = ((row * w + col) * 4) as usize;
                buffer[idx] = color.r;
                buffer[idx + 1] = color.g;
                buffer[idx + 2] = color.b;
                buffer[idx + 3] = color.a;
            }
        }
        buffer
    }

    pub fn set_rect_data(&mut self, x: u32, y: u32, w: u32, h: u32, data: &[u8]) {
        for row in 0..h {
            for col in 0..w {
                let idx = ((row * w + col) * 4) as usize;
                if idx + 3 < data.len() {
                    let color = Color::new(data[idx], data[idx+1], data[idx+2], data[idx+3]);
                    let _ = self.set_pixel(x + col, y + row, color);
                }
            }
        }
    }

    pub fn shift_and_resize(&mut self, dx: i32, dy: i32, new_width: u32, new_height: u32) {
        let mut new_chunks = HashMap::with_capacity(self.chunks.len());
        let is_aligned = dx % CHUNK_SIZE as i32 == 0 && dy % CHUNK_SIZE as i32 == 0;

        for ((cx, cy), chunk) in self.chunks.drain() {
            let base_x = cx as i32 * CHUNK_SIZE as i32 + dx;
            let base_y = cy as i32 * CHUNK_SIZE as i32 + dy;

            if base_x + CHUNK_SIZE as i32 <= 0 || base_x >= new_width as i32 ||
               base_y + CHUNK_SIZE as i32 <= 0 || base_y >= new_height as i32 {
                continue;
            }

            if is_aligned && base_x >= 0 && base_y >= 0 && 
               (base_x + CHUNK_SIZE as i32) <= new_width as i32 && 
               (base_y + CHUNK_SIZE as i32) <= new_height as i32 {
                new_chunks.insert(((base_x as u32) / CHUNK_SIZE, (base_y as u32) / CHUNK_SIZE), chunk);
                continue;
            }

            for (i, pixel) in chunk.data.chunks_exact(4).enumerate() {
                if pixel[3] > 0 { 
                    let lx = (i as u32) % CHUNK_SIZE;
                    let ly = (i as u32) / CHUNK_SIZE;
                    let nx = base_x + lx as i32;
                    let ny = base_y + ly as i32;

                    if nx >= 0 && nx < new_width as i32 && ny >= 0 && ny < new_height as i32 {
                        let n_cx = (nx as u32) / CHUNK_SIZE;
                        let n_cy = (ny as u32) / CHUNK_SIZE;
                        let n_lx = (nx as u32) % CHUNK_SIZE;
                        let n_ly = (ny as u32) % CHUNK_SIZE;

                        let target_chunk = new_chunks.entry((n_cx, n_cy)).or_insert_with(Chunk::new);
                        let n_idx = ((n_ly * CHUNK_SIZE + n_lx) * 4) as usize;
                        target_chunk.data_mut()[n_idx..n_idx+4].copy_from_slice(pixel);
                    }
                }
            }
        }
        self.chunks = new_chunks;
        self.width = new_width;
        self.height = new_height;
    }
}

#[cfg(test)]
mod tests;