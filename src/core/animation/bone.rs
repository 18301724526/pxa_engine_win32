use super::transform::Transform;

#[derive(Debug, Clone, PartialEq)]
pub struct BoneData {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub length: f32,
    pub local_transform: Transform,
    pub inherit_rotation: bool,
    pub inherit_scale: bool,
}

impl BoneData {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            parent_id: None,
            length: 0.0,
            local_transform: Transform::default(),
            inherit_rotation: true,
            inherit_scale: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bone_creation() {
        let bone = BoneData::new("bone_1".into(), "Root".into());
        assert_eq!(bone.id, "bone_1");
        assert_eq!(bone.name, "Root");
        assert_eq!(bone.parent_id, None);
        assert_eq!(bone.local_transform.scale_x, 1.0);
        assert_eq!(bone.inherit_rotation, true);
    }
}