use std::any::Any;

/// 抽象骨骼数据的访问与更新接口
pub trait SkeletonStorage: Send + Sync {
    fn get_bone_world_position(&self, id: &str) -> Option<(f32, f32)>;
    fn get_parent_world_matrix(&self, bone_idx: usize) -> [f32; 6];
    fn update(&mut self);
    
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[cfg(test)]
pub mod mocks {
    use super::*;

    pub struct MockSkeletonStorage {
        pub position: (f32, f32),
    }

    impl SkeletonStorage for MockSkeletonStorage {
        fn get_bone_world_position(&self, _id: &str) -> Option<(f32, f32)> { Some(self.position) }
        fn get_parent_world_matrix(&self, _bone_idx: usize) -> [f32; 6] { [1.0, 0.0, 0.0, 1.0, 0.0, 0.0] }
        fn update(&mut self) {}
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    #[test]
    fn test_mock_skeleton_storage() {
        let skel = MockSkeletonStorage { position: (10.0, 20.0) };
        assert_eq!(skel.get_bone_world_position("any"), Some((10.0, 20.0)));
    }
}