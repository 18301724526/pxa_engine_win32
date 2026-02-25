use crate::app::context::CanvasContext;
use crate::core::id_gen;
use crate::core::layer::Layer;
use crate::history::patch::ActionPatch;
use crate::render::blend::blend_pixels;
use crate::core::error::Result;
use rust_i18n::t;

pub struct LayerService;

impl LayerService {
    pub fn add_new_layer(ctx: CanvasContext) -> Result<()> {
        let id = format!("layer_{}", id_gen::gen_id());
        let mut next_idx = ctx.store.layers.len() + 1;
        for l in &ctx.store.layers {
            let num_part: String = l.name.chars()
                .rev()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .chars().rev().collect();
            
            if let Ok(n) = num_part.parse::<usize>() {
                if n >= next_idx { next_idx = n.saturating_add(1); }
            }
        }

        let name = t!("layer.default_name", num = next_idx).to_string();
        let layer = Layer::new(id.clone(), name, ctx.store.canvas_width, ctx.store.canvas_height);
        let index = ctx.store.layers.len();
        let old_active_id = ctx.store.active_layer_id.clone();
        
        let patch = ActionPatch::new_layer_add(
            format!("patch_{}", id), 
            id.clone(), 
            layer, 
            index,
            old_active_id
        );
        
        ctx.history.commit(patch, ctx.store)?;
        ctx.store.active_layer_id = Some(id);
        Ok(())
    }
    pub fn delete_active_layer(ctx: CanvasContext) -> Result<()> {
        let active_id = match &ctx.store.active_layer_id {
            Some(id) => id.clone(), 
            None => return Ok(())
        };
        
        if ctx.store.layers.len() <= 1 { return Ok(()); }
        
        if let Some(index) = ctx.store.layers.iter().position(|l| l.id == active_id) {
            let layer = ctx.store.layers[index].clone();
            if layer.locked { return Err(crate::core::error::CoreError::LayerLocked); }
            let old_active_id = ctx.store.active_layer_id.clone();
            
            let patch = ActionPatch::new_layer_remove(
                format!("patch_rm_{}", id_gen::gen_id()),
                active_id, 
                layer, 
                index,
                old_active_id
            );
            ctx.history.commit(patch, ctx.store)?;
        }
        Ok(())
    }
    pub fn toggle_visibility(ctx: CanvasContext, layer_id: &str) -> Result<()> {
        if let Some(layer) = ctx.store.get_layer(layer_id) {
            let new_vis = !layer.visible;
            let patch = ActionPatch::new_layer_visibility(
                format!("patch_vis_{}", id_gen::gen_id()),
                layer_id.to_string(), 
                new_vis
            );
            ctx.history.commit(patch, ctx.store)?;
        }
        Ok(())
    }
    pub fn duplicate_layer(ctx: CanvasContext, layer_id: &str) -> Result<()> {
        if let Some(index) = ctx.store.layers.iter().position(|l| l.id == layer_id) {
            let mut new_layer = ctx.store.layers[index].clone();
            let old_active_id = ctx.store.active_layer_id.clone();
            new_layer.id = format!("layer_{}", id_gen::gen_id());
            new_layer.name = t!("layer.copy_name", name = new_layer.name).to_string();
            
            let patch = ActionPatch::new_layer_add(
                format!("patch_{}", new_layer.id),
                new_layer.id.clone(),
                new_layer,
                index + 1,
                old_active_id
            );
            
            ctx.history.commit(patch, ctx.store)?;
            ctx.store.active_layer_id = Some(ctx.store.layers[index + 1].id.clone());
        }
        Ok(())
    }

    pub fn merge_selected_layers(ctx: CanvasContext, ids: Vec<String>) -> Result<()> {
        let mut indices: Vec<usize> = ids.iter().filter_map(|id| ctx.store.layers.iter().position(|l| l.id == *id)).collect();
        indices.sort_unstable();
        indices.dedup();
        if indices.len() <= 1 { return Ok(()); }
        Self::do_merge_layers(ctx, indices, t!("layer.merged_name").to_string())
    }

    fn do_merge_layers(ctx: CanvasContext, indices: Vec<usize>, new_name: String) -> Result<()> {
        let w = ctx.store.canvas_width;
        let h = ctx.store.canvas_height;
        let new_id = format!("layer_{}", id_gen::gen_id());
        let mut merged_layer = Layer::new(new_id.clone(), new_name, w, h);
        
        for y in 0..h {
            for x in 0..w {
                let mut current_color = [0u8, 0, 0, 0];
                for &idx in &indices {
                    let l = &ctx.store.layers[idx];
                    let lx = x as i32 - l.offset_x; let ly = y as i32 - l.offset_y;
                    if lx >= 0 && ly >= 0 && lx < l.width as i32 && ly < l.height as i32 {
                        if let Some(c) = l.get_pixel(lx as u32, ly as u32) {
                            let src = [c.r, c.g, c.b, c.a];
                            current_color = blend_pixels(current_color, src, l.blend_mode, l.opacity);
                        }
                    }
                }
                if current_color[3] > 0 {
                    let _ = merged_layer.set_pixel(x, y, crate::core::color::Color::new(current_color[0], current_color[1], current_color[2], current_color[3]));
                }
            }
        }

        let mut patches = Vec::new();
        for &idx in indices.iter().rev() {
            let layer = ctx.store.layers[idx].clone();
            let old_active_id = ctx.store.active_layer_id.clone();
            patches.push(ActionPatch::new_layer_remove(format!("rm_{}", layer.id), layer.id.clone(), layer, idx, old_active_id));
        }
        
        let insert_index = indices.first().copied().unwrap_or(0);
        let old_active_id = ctx.store.active_layer_id.clone();
        patches.push(ActionPatch::new_layer_add(format!("add_merged_{}", new_id), new_id.clone(), merged_layer, insert_index, old_active_id));
        
        let composite_patch = ActionPatch::new_composite(format!("merge_{}", id_gen::gen_id()), patches);
        ctx.history.commit(composite_patch, ctx.store)?;
        ctx.store.active_layer_id = Some(new_id);
        Ok(())
    }
}