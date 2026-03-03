#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_bone_creation_tool_logic() {
        let mut tool = CreateBoneTool::new();

        tool.on_pointer_down(0, 0, &mut crate::core::store::PixelStore::new(1,1), &crate::core::symmetry::SymmetryConfig::new(1,1)).unwrap();
        tool.on_pointer_move(100, 0, &mut crate::core::store::PixelStore::new(1,1), &crate::core::symmetry::SymmetryConfig::new(1,1)).unwrap();
        
        assert_eq!(tool.start_pos, Some((0.0, 0.0)));
        assert_eq!(tool.preview_end, Some((100.0, 0.0)));
    }
}