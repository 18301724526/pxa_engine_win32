use crate::core::color::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveType {
    Linear,
    Stepped,
    Bezier(f32, f32, f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeValue {
    Rotate(f32),
    Translate(f32, f32),
    Scale(f32, f32),
    Color(u8, u8, u8, u8), 
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimelineProperty {
    Rotation,
    Translation,
    Scale,
    Color,
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32, 
    pub value: KeyframeValue,
    pub curve: CurveType,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub target_id: String,
    pub property: TimelineProperty,
    pub keyframes: Vec<Keyframe>,
}

impl Timeline {
    pub fn new(target_id: String, property: TimelineProperty) -> Self {
        Self {
            target_id,
            property,
            keyframes: Vec::new(),
        }
    }

    pub fn add_keyframe(&mut self, time: f32, value: KeyframeValue, curve: CurveType) {
        let match_type = match (&self.property, &value) {
            (TimelineProperty::Rotation, KeyframeValue::Rotate(_)) => true,
            (TimelineProperty::Translation, KeyframeValue::Translate(_, _)) => true,
            (TimelineProperty::Scale, KeyframeValue::Scale(_, _)) => true,
            (TimelineProperty::Color, KeyframeValue::Color(_, _, _, _)) => true,
            _ => false,
        };

        if match_type {
            if let Some(existing) = self.keyframes.iter_mut().find(|k| (k.time - time).abs() < 0.0001) {
                existing.value = value;
                existing.curve = curve;
            } else {
                self.keyframes.push(Keyframe { time, value, curve });
                self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
            }
        }
    }

    pub fn sample(&self, time: f32) -> Option<KeyframeValue> {
        if self.keyframes.is_empty() {
            return None;
        }

        if time <= self.keyframes.first()?.time {
            return Some(self.keyframes.first()?.value.clone());
        }

        if time >= self.keyframes.last()?.time {
            return Some(self.keyframes.last()?.value.clone());
        }

        let mut idx = 0;
        for (i, frame) in self.keyframes.iter().enumerate() {
            if time < frame.time {
                idx = i.saturating_sub(1);
                break;
            }
        }

        let start = &self.keyframes[idx];
        let end = &self.keyframes[idx + 1];

        match start.curve {
            CurveType::Stepped => Some(start.value.clone()),
            CurveType::Linear => {
                let duration = end.time - start.time;
                if duration <= 0.0001 {
                    return Some(start.value.clone());
                }
                let t = (time - start.time) / duration;
                Self::lerp_value(&start.value, &end.value, t)
            }
            CurveType::Bezier(cx1, cy1, cx2, cy2) => {
                let duration = end.time - start.time;
                if duration <= 0.0001 {
                    return Some(start.value.clone());
                }
                let t = (time - start.time) / duration;
                let eased_t = Self::solve_bezier_y(cx1, cy1, cx2, cy2, t);
                Self::lerp_value(&start.value, &end.value, eased_t)
            }
        }
    }

    pub fn solve_bezier_y(cx1: f32, cy1: f32, cx2: f32, cy2: f32, t: f32) -> f32 {
        if t <= 0.0 { return 0.0; }
        if t >= 1.0 { return 1.0; }
        
        let mut min_u = 0.0;
        let mut max_u = 1.0;
        let mut u = t;
        
        for _ in 0..10 { 
            let x = 3.0 * (1.0 - u).powi(2) * u * cx1 + 3.0 * (1.0 - u) * u.powi(2) * cx2 + u.powi(3);
            if (x - t).abs() < 0.001 { break; }
            if x < t { min_u = u; } else { max_u = u; }
            u = (min_u + max_u) / 2.0;
        }
        
        3.0 * (1.0 - u).powi(2) * u * cy1 + 3.0 * (1.0 - u) * u.powi(2) * cy2 + u.powi(3)
    }

    fn lerp_value(v1: &KeyframeValue, v2: &KeyframeValue, t: f32) -> Option<KeyframeValue> {
        match (v1, v2) {
            (KeyframeValue::Rotate(r1), KeyframeValue::Rotate(r2)) => {
                let mut diff = r2 - r1;
                while diff <= -180.0 { diff += 360.0; }
                while diff > 180.0 { diff -= 360.0; }
                
                Some(KeyframeValue::Rotate(r1 + diff * t))
            },
            (KeyframeValue::Translate(x1, y1), KeyframeValue::Translate(x2, y2)) => {
                Some(KeyframeValue::Translate(
                    x1 + (x2 - x1) * t,
                    y1 + (y2 - y1) * t,
                ))
            },
            (KeyframeValue::Scale(x1, y1), KeyframeValue::Scale(x2, y2)) => {
                Some(KeyframeValue::Scale(
                    x1 + (x2 - x1) * t,
                    y1 + (y2 - y1) * t,
                ))
            },
            (KeyframeValue::Color(r1, g1, b1, a1), KeyframeValue::Color(r2, g2, b2, a2)) => {
                Some(KeyframeValue::Color(
                    (*r1 as f32 + (*r2 as f32 - *r1 as f32) * t) as u8,
                    (*g1 as f32 + (*g2 as f32 - *g1 as f32) * t) as u8,
                    (*b1 as f32 + (*b2 as f32 - *b1 as f32) * t) as u8,
                    (*a1 as f32 + (*a2 as f32 - *a1 as f32) * t) as u8,
                ))
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub timelines: Vec<Timeline>,
}

impl Animation {
    pub fn new(name: String, duration: f32) -> Self {
        Self {
            name,
            duration,
            timelines: Vec::new(),
        }
    }
    pub fn initialize_tracks(&mut self, skeleton: &crate::core::animation::skeleton::Skeleton) {
        for bone in &skeleton.bones {
            let props = [
                TimelineProperty::Translation,
                TimelineProperty::Rotation,
                TimelineProperty::Scale,
            ];
            for prop in props {
                if !self.timelines.iter().any(|t| t.target_id == bone.data.id && t.property == prop) {
                    self.timelines.push(Timeline::new(bone.data.id.clone(), prop));
                }
            }
        }
    }

    pub fn recalculate_duration(&mut self) {
        let mut max_time = 0.0_f32;
        for tl in &self.timelines {
            for kf in &tl.keyframes {
                if kf.time > max_time {
                    max_time = kf.time;
                }
            }
        }
        self.duration = max_time;
    }
    
    pub fn apply(&self, skeleton: &mut super::skeleton::Skeleton, time: f32) {
        let t = if self.duration > 0.0 { time % self.duration } else { 0.0 };

        for timeline in &self.timelines {
            if let Some(val) = timeline.sample(t) {
                match timeline.property {
                    TimelineProperty::Color => {
                        if let Some(slot) = skeleton.slots.iter_mut().find(|s| s.data.id == timeline.target_id) {
                            if let KeyframeValue::Color(r, g, b, a) = val {
                                slot.current_color = Color::new(r, g, b, a);
                            }
                        }
                    },
                    _ => {
                        if let Some(bone) = skeleton.bones.iter_mut().find(|b| b.data.id == timeline.target_id) {
                            match val {
                                KeyframeValue::Rotate(r) => bone.local_transform.rotation = r,
                                KeyframeValue::Translate(x, y) => {
                                    bone.local_transform.x = x;
                                    bone.local_transform.y = y;
                                },
                                KeyframeValue::Scale(x, y) => {
                                    bone.local_transform.scale_x = x;
                                    bone.local_transform.scale_y = y;
                                },
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_wrap_around() {
        let v1 = KeyframeValue::Rotate(350.0);
        let v2 = KeyframeValue::Rotate(10.0);
        if let Some(KeyframeValue::Rotate(r)) = Timeline::lerp_value(&v1, &v2, 0.5) {
            assert!((r - 360.0).abs() < 0.001 || (r - 0.0).abs() < 0.001, "Rotation failed: {}", r);
        } else {
            panic!("Lerp failed");
        }
    }
}