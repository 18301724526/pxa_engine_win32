use crate::core::pixelizer::config::PixelizeConfig;
use image::DynamicImage;
use std::sync::{Arc, Mutex};
use egui::ColorImage;

pub struct ImportModalState {
    pub is_open: bool,
    pub original_image: Option<DynamicImage>,
    pub config: PixelizeConfig,
    pub preview_texture: Option<egui::TextureHandle>,
    // 用于异步生成预览，避免拖动滑块时 UI 卡死
    pub is_processing: bool,
    pub pending_result: Arc<Mutex<Option<(ColorImage, Vec<u8>)>>>, // 同时保存 UI 图像和底层生肉数据
    pub cached_pixel_data: Option<Vec<u8>>,
}

impl ImportModalState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            original_image: None,
            config: PixelizeConfig::default(),
            preview_texture: None,
            is_processing: false,
            pending_result: Arc::new(Mutex::new(None)),
            cached_pixel_data: None,
        }
    }

    pub fn open_with_image(&mut self, img: DynamicImage) {
        self.original_image = Some(img);
        self.config = PixelizeConfig::default(); // 重置为默认
        self.is_open = true;
        self.trigger_preview_update();
    }

    pub fn trigger_preview_update(&mut self) {
        if let Some(img) = &self.original_image {
            self.is_processing = true;
            let img_clone = img.clone();
            let cfg_clone = self.config.clone();
            let result_ptr = Arc::clone(&self.pending_result);

            // 放进后台线程运算，保证 UI 流畅
            std::thread::spawn(move || {
                let pixel_data = crate::core::pixelizer::PixelizerPipeline::process_image(&img_clone, &cfg_clone);
                let color_image = ColorImage::from_rgba_unmultiplied(
                    [cfg_clone.target_w as usize, cfg_clone.target_h as usize],
                    &pixel_data,
                );
                *result_ptr.lock().unwrap() = Some((color_image, pixel_data));
            });
        }
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.original_image = None;
        self.preview_texture = None;
        self.cached_pixel_data = None;
    }
}