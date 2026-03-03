use crate::core::color::Color;
use crate::core::layer::Layer;
use crate::core::error::Result;
use std::any::Any;

/// 抽象像素存储与图层管理的核心接口
pub trait PixelStorage: Send + Sync {
    fn canvas_width(&self) -> u32;
    fn canvas_height(&self) -> u32;
    fn add_layer(&mut self, layer: Layer);
    fn get_layer(&self, id: &str) -> Option<&Layer>;
    fn get_layer_mut(&mut self, id: &str) -> Option<&mut Layer>;
    fn get_pixel(&self, layer_id: &str, canvas_x: u32, canvas_y: u32) -> Option<Color>;
    fn mut_set_pixel(&mut self, layer_id: &str, canvas_x: u32, canvas_y: u32, color: Color) -> Result<()>;
    
    /// 提供向下转型能力，用于重构过渡期兼容旧代码
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::collections::HashMap;

    pub struct MockPixelStorage {
        pub width: u32,
        pub height: u32,
        pub layers: HashMap<String, Layer>,
    }

    impl MockPixelStorage {
        pub fn new(width: u32, height: u32) -> Self {
            Self { width, height, layers: HashMap::new() }
        }
    }

    impl PixelStorage for MockPixelStorage {
        fn canvas_width(&self) -> u32 { self.width }
        fn canvas_height(&self) -> u32 { self.height }
        fn add_layer(&mut self, layer: Layer) { self.layers.insert(layer.id.clone(), layer); }
        fn get_layer(&self, id: &str) -> Option<&Layer> { self.layers.get(id) }
        fn get_layer_mut(&mut self, id: &str) -> Option<&mut Layer> { self.layers.get_mut(id) }
        fn get_pixel(&self, _layer_id: &str, _x: u32, _y: u32) -> Option<Color> { Some(Color::transparent()) }
        fn mut_set_pixel(&mut self, _layer_id: &str, _x: u32, _y: u32, _color: Color) -> Result<()> { Ok(()) }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    #[test]
    fn test_mock_pixel_storage() {
        let mut store = MockPixelStorage::new(100, 100);
        store.add_layer(Layer::new("l1".into(), "Layer 1".into(), 100, 100));
        assert_eq!(store.canvas_width(), 100);
        assert!(store.get_layer("l1").is_some());
    }
}