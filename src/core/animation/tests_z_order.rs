#[cfg(test)]
mod tests {
    use crate::core::animation::skeleton::Skeleton;
    use crate::core::animation::slot::{SlotData, RuntimeSlot};
    use crate::core::animation::bone::BoneData;

    #[test]
    fn test_slot_rendering_order() {
        let mut skel = Skeleton::new();
        skel.add_bone(BoneData::new("b1".into(), "b1".into()));

        let slot_a = SlotData::new("A".into(), "A".into(), "b1".into());
        let slot_b = SlotData::new("B".into(), "B".into(), "b1".into());
        
        skel.slots.push(RuntimeSlot::new(slot_a));
        skel.slots.push(RuntimeSlot::new(slot_b));

        assert_eq!(skel.slots[0].data.id, "A");
        assert_eq!(skel.slots[1].data.id, "B");

        skel.slots.swap(0, 1);
        
        assert_eq!(skel.slots[0].data.id, "B");
        assert_eq!(skel.slots[1].data.id, "A");
    }
}