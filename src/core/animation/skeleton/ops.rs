use super::Skeleton;
use crate::core::error::CoreError;

impl Skeleton {
    /// 核心重建逻辑：重新生成 name_to_index 映射表，并刷新所有骨骼的 parent_index
    pub fn rebuild_internal_state(&mut self) {
        // 1. 重建哈希映射表
        self.name_to_index.clear();
        for (idx, bone) in self.bones.iter().enumerate() {
            self.name_to_index.insert(bone.data.id.clone(), idx);
        }

        // 2. 刷新每个骨骼的父级索引
        for idx in 0..self.bones.len() {
            let parent_index = if let Some(parent_id) = &self.bones[idx].data.parent_id {
                self.name_to_index.get(parent_id).copied()
            } else {
                None
            };
            self.bones[idx].parent_index = parent_index;
        }
    }
    pub fn purge_bone_atomic(&mut self, bone_id: &str) -> Result<(), CoreError> {
        let target_idx = self.bone_id_to_index(bone_id)
            .ok_or_else(|| CoreError::BoneNotFound(bone_id.to_string()))?;

        let parent_id = self.bones[target_idx].data.parent_id.clone();
        let fallback_id = parent_id.clone().unwrap_or_default();

        for bone in &mut self.bones {
            if let Some(pid) = &bone.data.parent_id {
                if pid == bone_id {
                    bone.data.parent_id = parent_id.clone();
                }
            }
        }

        for slot in &mut self.slots {
            if slot.data.bone_id == bone_id {
                slot.data.bone_id = fallback_id.clone();
            }
        }

        self.bones.remove(target_idx);
        self.rebuild_internal_state();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::animation::bone::BoneData;
    use crate::core::animation::skeleton::data::RuntimeBone;
    use crate::core::animation::slot::{SlotData, RuntimeSlot};

    #[test]
    fn test_rebuild_after_manual_modification() {
        let mut skel = Skeleton::new();
        assert_eq!(skel.bone_id_to_index("root"), Some(0));
        
        let b1_data = BoneData::new("b1".into(), "B1".into());
        skel.bones.push(RuntimeBone::new(b1_data)); 
        
        assert_eq!(skel.bone_id_to_index("b1"), None);

        skel.rebuild_internal_state();

        assert_eq!(skel.bone_id_to_index("root"), Some(0));
        assert_eq!(skel.bone_id_to_index("b1"), Some(1));
    }

    #[test]
    fn test_purge_bone_hierarchy() {
        let mut skel = Skeleton::new();

        skel.add_bone(BoneData::new("grandpa".into(), "Grandpa".into()));

        let mut father = BoneData::new("father".into(), "Father".into());
        father.parent_id = Some("grandpa".into());
        skel.add_bone(father);

        let mut son = BoneData::new("son".into(), "Son".into());
        son.parent_id = Some("father".into());
        skel.add_bone(son);

        let slot = SlotData::new("slot1".into(), "Slot1".into(), "father".into());
        skel.slots.push(RuntimeSlot::new(slot));

        assert!(skel.purge_bone_atomic("father").is_ok());

        assert_eq!(skel.bones.len(), 3, "物理节点应已移除 (剩余 root, grandpa, son)");
        assert_eq!(skel.bone_id_to_index("father"), None, "映射表中不应存在 father");

        let son_idx = skel.bone_id_to_index("son").unwrap();
        assert_eq!(skel.bones[son_idx].data.parent_id, Some("grandpa".to_string()));
        let grandpa_idx = skel.bone_id_to_index("grandpa").unwrap();
        assert_eq!(skel.bones[son_idx].parent_index, Some(grandpa_idx));

        assert_eq!(skel.slots[0].data.bone_id, "grandpa");
    }
}