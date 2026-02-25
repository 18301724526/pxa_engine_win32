#[cfg(test)]
mod tests {
    use crate::core::layer::Layer;
    use crate::core::color::Color;

    #[test]
    fn test_layer_versioning() {
        let mut layer = Layer::new("v_test".into(), "Test".into(), 10, 10);
        let v0 = layer.version;
        
        layer.set_pixel(1, 1, Color::new(255, 0, 0, 255)).unwrap();
        let v1 = layer.version;
        
        assert!(v1 > v0, "修改像素后版本号应增加");

        layer.locked = true;
        let _ = layer.set_pixel(2, 2, Color::new(0, 255, 0, 255));
        let v2 = layer.version;

        assert_eq!(v1, v2, "锁定图层修改失败时不应增加版本号");
    }
}