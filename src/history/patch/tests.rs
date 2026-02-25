use super::*;
use crate::core::color::Color;
#[test]
fn test_patch_data() {
    let mut p = ActionPatch::new_pixel_diff("id".into(), "l".into());
    p.add_pixel_diff(0, 0, Color::transparent(), Color::new(1, 1, 1, 255));
    if let Some(d) = p.pixel_diffs() {
        assert_eq!(d.len(), 1);
    }
}