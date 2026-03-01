use crate::animation::project::AnimProject;

#[derive(Clone)]
pub enum AnimPatch {
    Timeline { 
        anim_id: String, 
        bone_id: String, 
        prop: crate::core::animation::timeline::TimelineProperty, 
        old: Option<crate::core::animation::timeline::Timeline>, 
        new: Option<crate::core::animation::timeline::Timeline> 
    },
    Skeleton { 
        old: crate::core::animation::skeleton::Skeleton, 
        new: crate::core::animation::skeleton::Skeleton 
    },
    Composite(Vec<AnimPatch>),
}

pub struct AnimHistory {
    pub undo_stack: Vec<AnimPatch>,
    pub redo_stack: Vec<AnimPatch>,
}

impl AnimHistory {
    pub fn new() -> Self { Self { undo_stack: Vec::new(), redo_stack: Vec::new() } }
    pub fn commit(&mut self, patch: AnimPatch) {
        self.undo_stack.push(patch);
        self.redo_stack.clear();
    }
    pub fn undo(&mut self, project: &mut AnimProject) -> bool {
        if let Some(patch) = self.undo_stack.pop() {
            self.apply_patch(project, &patch, true);
            self.redo_stack.push(patch);
            true
        } else { false }
    }
    pub fn redo(&mut self, project: &mut AnimProject) -> bool {
        if let Some(patch) = self.redo_stack.pop() {
            self.apply_patch(project, &patch, false);
            self.undo_stack.push(patch);
            true
        } else { false }
    }
    fn apply_patch(&self, project: &mut AnimProject, patch: &AnimPatch, is_undo: bool) {
        match patch {
            AnimPatch::Timeline { anim_id, bone_id, prop, old, new } => {
                if let Some(anim) = project.animations.get_mut(anim_id) {
                    let target = if is_undo { old } else { new };
                    if let Some(tl) = target {
                        if let Some(existing) = anim.timelines.iter_mut().find(|t| &t.target_id == bone_id && &t.property == prop) {
                            *existing = tl.clone();
                        } else {
                            anim.timelines.push(tl.clone());
                        }
                    } else {
                        anim.timelines.retain(|t| !(&t.target_id == bone_id && &t.property == prop));
                    }
                    anim.recalculate_duration();
                }
            }
            AnimPatch::Skeleton { old, new } => { project.skeleton = if is_undo { old.clone() } else { new.clone() }; }
            AnimPatch::Composite(patches) => {
                let iter: Box<dyn Iterator<Item = &AnimPatch>> = if is_undo { Box::new(patches.iter().rev()) } else { Box::new(patches.iter()) };
                for p in iter { self.apply_patch(project, p, is_undo); }
            }
        }
    }
}