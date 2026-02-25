use crate::core::blend_mode::BlendMode;

#[inline(always)]
pub fn blend_pixels(bg: [u8; 4], fg: [u8; 4], mode: BlendMode, global_opacity: u8) -> [u8; 4] {
    let src_a = (fg[3] as u32 * global_opacity as u32) / 255;
    if src_a == 0 { return bg; }
    let inv_a = 255 - src_a;

    let b0 = bg[0] as u32; let b1 = bg[1] as u32; let b2 = bg[2] as u32;
    let f0 = fg[0] as u32; let f1 = fg[1] as u32; let f2 = fg[2] as u32;

    let (r, g, b) = match mode {
        BlendMode::Normal => (f0, f1, f2),
        BlendMode::Multiply => ((b0 * f0) / 255, (b1 * f1) / 255, (b2 * f2) / 255),
        BlendMode::Screen => (
            255 - ((255 - b0) * (255 - f0)) / 255,
            255 - ((255 - b1) * (255 - f1)) / 255,
            255 - ((255 - b2) * (255 - f2)) / 255,
        ),
        BlendMode::Add => (
            (b0 + f0).min(255),
            (b1 + f1).min(255),
            (b2 + f2).min(255),
        ),
    };

    [
        ((r * src_a + b0 * inv_a) / 255) as u8,
        ((g * src_a + b1 * inv_a) / 255) as u8,
        ((b * src_a + b2 * inv_a) / 255) as u8,
        255
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_blend_multiply() {
        let bg = [255, 128, 0, 255];
        let fg = [128, 255, 255, 255];
        let res = blend_pixels(bg, fg, BlendMode::Multiply, 255);
        assert_eq!(res, [128, 128, 0, 255], "正片叠底计算错误");
    }
}