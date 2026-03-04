use pxa_engine_win32::core::pixelizer::pipeline::PixelizerPipeline;
use pxa_engine_win32::core::pixelizer::downsample::mode_downsample;
use pxa_engine_win32::core::pixelizer::quantize::quantize_colors;
use pxa_engine_win32::core::pixelizer::edge_selout::apply_selout;
use pxa_engine_win32::core::pixelizer::config::PixelizeConfig;
use pxa_engine_win32::core::color::Color;
use image::{RgbaImage, DynamicImage, Rgba};

#[test]
fn test_pixelizer_mode_downsample() {
    // 创建一个 4x4 的原图
    let mut img = RgbaImage::new(4, 4);
    for y in 0..4 {
        for x in 0..4 {
            // 左半边画红，右半边画蓝
            if x < 2 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            } else {
                img.put_pixel(x, y, Rgba([0, 0, 255, 255]));
            }
        }
    }
    // 故意在左上角加一个干扰噪点（绿），测试众数算法是否能忽略它
    img.put_pixel(0, 0, Rgba([0, 255, 0, 255]));
    
    let dyn_img = DynamicImage::ImageRgba8(img);
    
    // 缩小到 2x2
    let pixels = mode_downsample(&dyn_img, 2, 2);
    
    assert_eq!(pixels.len(), 4);
    // 左上角的区块 (包含3个红，1个绿)，众数应该是红
    assert_eq!(pixels[0], Color::new(255, 0, 0, 255), "众数降采样应忽略少数派噪点");
    // 右上角全是蓝
    assert_eq!(pixels[1], Color::new(0, 0, 255, 255));
}

#[test]
fn test_pixelizer_color_quantization() {
    let mut pixels = vec![
        Color::new(255, 0, 0, 255),   // 纯红
        Color::new(240, 10, 10, 255), // 脏红（应该被合并）
        Color::new(0, 255, 0, 255),   // 纯绿
        Color::new(10, 240, 10, 255), // 脏绿（应该被合并）
    ];
    
    // 强行聚类为 2 种主色
    quantize_colors(&mut pixels, 2, 1000);
    
    // 聚类完成后，原来相近的颜色必须被统一
    assert_eq!(pixels[0], pixels[1], "相似的红色必须被合并为同一种色阶");
    assert_eq!(pixels[2], pixels[3], "相似的绿色必须被合并为同一种色阶");
    assert_ne!(pixels[0], pixels[2], "差异大的颜色不能被混淆");
}

#[test]
fn test_pixelizer_edge_selout() {
    let mut pixels = vec![Color::new(200, 200, 200, 255); 36];
    
    for y in 2..4 {
        for x in 2..4 {
            pixels[y * 6 + x] = Color::new(50, 50, 50, 255);
        }
    }
    
    apply_selout(&mut pixels, 6, 6);
    
    // 原本只有 200 和 50 两种颜色，Sobel 边缘检测会找到交界处并生成加深的过渡描边色
    let mut edge_detected = false;
    for p in &pixels {
        if p.r != 200 && p.r != 50 { edge_detected = true; break; }
    }
    
    assert!(edge_detected, "Sobel 边缘检测应发现颜色突变并加深边缘像素");
}

#[test]
fn test_pixelizer_full_pipeline_integrity() {
    let mut img = RgbaImage::new(4, 4);
    img.put_pixel(1, 1, Rgba([200, 200, 200, 255]));
    img.put_pixel(2, 2, Rgba([200, 200, 200, 255]));
    let dyn_img = DynamicImage::ImageRgba8(img);

    // 使用新的配置结构体调用对外 API
    let config = PixelizeConfig {
        target_w: 2,
        target_h: 2,
        use_selout: true,
        ..Default::default()
    };
    let result = PixelizerPipeline::process_image(&dyn_img, &config);
    
    // 2x2 = 4 个像素，每个像素 RGBA 4 个通道，总长应为 16
    assert_eq!(result.len(), 16);
}

#[test]
fn test_import_modal_flow_and_params() {
    // 1. 模拟组件初始化
    let mut modal_state = pxa_engine_win32::ui::import_modal::state::ImportModalState::new();
    assert!(!modal_state.is_open, "初始状态必须是关闭的");

    // 2. 模拟触发导入事件，传入图片
    let mut img = RgbaImage::new(10, 10);
    img.put_pixel(5, 5, Rgba([255, 0, 0, 255]));
    let dyn_img = DynamicImage::ImageRgba8(img);
    
    modal_state.open_with_image(dyn_img.clone());
    assert!(modal_state.is_open, "传入图片后，UI 模态框应被标记为弹出 (is_open = true)");
    assert!(modal_state.original_image.is_some(), "原始图像数据已被妥善保存供预览");

    // 3. 模拟用户在 UI 拖动滑块（分辨率调到 16x16）
    modal_state.config.target_w = 16;
    modal_state.config.target_h = 16;
    modal_state.config.use_selout = true;

    // 4. 验证引擎是否严格听从了 UI 修改后的参数
    let result = PixelizerPipeline::process_image(modal_state.original_image.as_ref().unwrap(), &modal_state.config);
    assert_eq!(result.len(), 16 * 16 * 4, "像素化管线必须严格遵循 UI 滑块传入的宽高参数");
    
    // 5. 模拟用户点击取消或关闭窗口
    modal_state.close();
    assert!(!modal_state.is_open, "关闭后状态重置");
    assert!(modal_state.original_image.is_none(), "关闭后应清理内存避免泄露");
}