use crate::core::color::Color;

#[derive(Debug, Clone)]
pub struct SlotData {
    pub id: String,
    pub name: String,
    pub bone_id: String, 
    pub color: Color,    
    pub attachment: Option<String>, 
}

impl SlotData {
    pub fn new(id: String, name: String, bone_id: String) -> Self {
        Self {
            id,
            name,
            bone_id,
            color: Color::new(255, 255, 255, 255),
            attachment: None,
        }
    }
}

#[derive(Debug, Clone)] 
pub struct RuntimeSlot {
    pub data: SlotData,
    pub current_color: Color,
    pub current_attachment: Option<String>,
}

impl RuntimeSlot {
    pub fn new(data: SlotData) -> Self {
        let current_color = data.color;
        let current_attachment = data.attachment.clone();
        Self {
            data,
            current_color,
            current_attachment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_creation() {
        let slot = SlotData::new("slot_1".into(), "ArmSlot".into(), "arm_bone".into());
        assert_eq!(slot.bone_id, "arm_bone");
        assert_eq!(slot.color.a, 255);
    }
}