#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::animation::skeleton::Skeleton;

    #[test]
    fn test_bone_creation_tool_logic() {
        let mut skeleton = Skeleton::new();
        let mut tool = CreateBoneTool::new();

        tool.on_pointer_down(0, 0, &mut crate::core::store::PixelStore::new(1,1), &crate::core::symmetry::SymmetryConfig::new(1,1)).unwrap();
        tool.on_pointer_move(100, 0, &mut crate::core::store::PixelStore::new(1,1), &crate::core::symmetry::SymmetryConfig::new(1,1)).unwrap();
        
        tool.commit_to_skeleton(&mut skeleton);

        assert_eq!(skeleton.bones.len(), 1);
        let bone = &skeleton.bones[0];
        assert!((bone.data.length - 100.0).abs() < 0.001);
        assert!((bone.local_transform.rotation - 0.0).abs() < 0.001);
    }
}