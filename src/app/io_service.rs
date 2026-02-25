use crate::core::layer::Layer;
use crate::render::compositor::{Compositor, Viewport};
use crate::core::store::PixelStore;
use std::path::PathBuf;
use crate::core::symmetry::SymmetryConfig;
use crate::format::header::PxadHeader;
use crate::format::stream::{PxadReader, PxadWriter};
use crate::app::view_state::ViewState;
use crate::format::block::{read_block, write_block};
use crate::format::payload::*;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use crate::app::error::{AppError, Result};
use rust_i18n::t;

pub struct IoService;

impl IoService {
    pub fn pick_import_path() -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg"])
            .pick_file()
    }

    pub fn pick_export_path() -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_file_name("art.png")
            .add_filter("PNG", &["png"])
            .save_file()
    }

    pub fn pick_palette_import_path() -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("HEX Palette", &["hex", "txt"])
            .pick_file()
    }

    pub fn pick_palette_export_path() -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_file_name("palette.hex")
            .add_filter("HEX Palette", &["hex", "txt"])
            .save_file()
    }

    pub fn load_as_layer(path: PathBuf, target_width: u32, target_height: u32, id: String, name: String) -> Result<Layer> {
        let img = image::open(path)?;
        let resized = img.resize_exact(target_width, target_height, image::imageops::FilterType::Nearest);
        let rgba = resized.to_rgba8();
        let mut layer = Layer::new(id, name, target_width, target_height);
        layer.set_rect_data(0, 0, target_width, target_height, &rgba.into_vec());
        Ok(layer)
    }

    pub fn save_png(path: PathBuf, store: &PixelStore) -> Result<()> {
        let width = store.canvas_width;
        let height = store.canvas_height;
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let view = Viewport { screen_width: width, screen_height: height, zoom: 1.0, pan_x: 0.0, pan_y: 0.0 };
        
        Compositor::render(store,  &mut pixels, view);
        image::save_buffer(path, &pixels, width, height, image::ColorType::Rgba8)?;
        Ok(())
    }
    pub fn pick_project_save_path() -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_file_name("project.pxad")
            .add_filter("PXA Document", &["pxad"])
            .save_file()
    }

    pub fn pick_project_load_path() -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("PXA Document", &["pxad"])
            .pick_file()
    }

    pub fn save_project(path: PathBuf, store: &PixelStore, symmetry: &SymmetryConfig, view: &ViewState) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = PxadWriter::new(BufWriter::new(file));

        let mut header = PxadHeader::new();
        header.block_count = 4 + store.layers.len() as u64;
        header.write_to(&mut writer)?;

        write_block(&mut writer, *b"CANV", &serialize_canvas(store, view.pan_x, view.pan_y, view.zoom_level))?;
        write_block(&mut writer, *b"SYMM", &serialize_symmetry(symmetry))?;
        write_block(&mut writer, *b"PALT", &serialize_palette(&store.palette))?;
        write_block(&mut writer, *b"SELE", &crate::format::payload::serialize_selection(&store.selection))?;

        for layer in &store.layers {
            write_block(&mut writer, *b"LAYR", &serialize_layer(layer))?;
        }

        writer.finish()?;
        Ok(())
    }

    pub fn load_project(path: PathBuf) -> Result<(PixelStore, SymmetryConfig, f32, f32, f64)> {
        let file = File::open(path)?;
        let mut reader = PxadReader::new(BufReader::new(file));

        let header = PxadHeader::read_from(&mut reader)?;
        if header.major_version > crate::format::header::CURRENT_MAJOR_VERSION {
            return Err(AppError::VersionTooHigh);
        }
        if header.minor_version > crate::format::header::CURRENT_MINOR_VERSION {
            println!("{}", t!("warning.version_too_high"));
        }

        let mut store = PixelStore::new(1, 1);
        store.layers.clear(); 
        let mut symmetry = SymmetryConfig::new(1, 1);
        let mut pan_x = 0.0;
        let mut pan_y = 0.0;
        let mut zoom_level = 1.0;

        for _ in 0..header.block_count {
            let (b_type, payload) = read_block(&mut reader)?;
            
            match &b_type {
                b"CANV" => {
                    (pan_x, pan_y, zoom_level) = deserialize_canvas(&payload, &mut store)?;
                    if store.canvas_width == 0 || store.canvas_height == 0 {
                        return Err(AppError::Format(crate::format::error::FormatError::InvalidData(t!("error.canvas_size_zero").to_string())));
                    }
                    if store.canvas_width > 16384 || store.canvas_height > 16384 {
                        return Err(AppError::Format(crate::format::error::FormatError::InvalidData(t!("error.canvas_size_limit", max = 16384).to_string())));
                    }
                    let required_size = (store.canvas_width * store.canvas_height * 4) as usize;
                    store.composite_cache = vec![0u8; required_size];
                },
                b"SYMM" => symmetry = deserialize_symmetry(&payload)?,
                b"SELE" => store.selection = deserialize_selection(&payload)?,
                b"LAYR" => {
                    let layer = deserialize_layer(&payload, header.minor_version)?;
                    if layer.width != store.canvas_width || layer.height != store.canvas_height {
                        return Err(AppError::Format(crate::format::error::FormatError::InvalidData(
                            format!("图层 '{}' 尺寸 ({}x{}) 与画布 ({}x{}) 不匹配", 
                                layer.name, layer.width, layer.height, store.canvas_width, store.canvas_height)
                        )));
                    }
                    store.add_layer(layer);
                },
                _ => continue, 
            }
        }

        reader.verify_footer()?;

        if let Some(last_layer) = store.layers.last() {
            store.active_layer_id = Some(last_layer.id.clone());
        }

        Ok((store, symmetry, pan_x, pan_y, zoom_level))
    }
}