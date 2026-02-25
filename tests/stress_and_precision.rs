use pxa_engine_win32::app::state::{AppState, ToolType};
use pxa_engine_win32::core::layer::Layer;
use pxa_engine_win32::core::color::Color;
use crc32fast::Hasher;

fn get_state_hash(app: &AppState) -> u32 {
    let mut hasher = Hasher::new();
    let store = app.engine.store();

    hasher.update(store.active_layer_id.as_deref().unwrap_or("none").as_bytes());
    hasher.update(&[store.primary_color.r, store.primary_color.g, store.primary_color.b, store.primary_color.a]);

    hasher.update(&[store.selection.is_active as u8]);
    if store.selection.is_active {
        for &m in &store.selection.mask {
            hasher.update(&[m as u8]);
        }
    }

    for layer in &store.layers {
        hasher.update(layer.id.as_bytes());
        hasher.update(&layer.offset_x.to_le_bytes());
        hasher.update(&layer.offset_y.to_le_bytes());
        hasher.update(&[layer.opacity, layer.blend_mode.to_u8(), layer.visible as u8]);

        let mut keys: Vec<_> = layer.chunks.keys().collect();
        keys.sort_by_key(|&(x, y)| (y, x));

        for k in keys {
            let chunk = &layer.chunks[k];
            if !chunk.is_empty() {
                hasher.update(&k.0.to_le_bytes());
                hasher.update(&k.1.to_le_bytes());
                hasher.update(chunk.data.as_ref()); 
            }
        }
    }
    hasher.finalize()
}

#[test]
fn test_chaos_stress_fuzz() {
    let mut app = AppState::new();
    let mut seed: u64 = 42;
    let mut next_rand = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed
    };

    for i in 0..500 {
        let op = next_rand() % 10;
        match op {
            0 => {
                let tools = [ToolType::Pencil, ToolType::Eraser, ToolType::Bucket, ToolType::RectSelect, ToolType::Move];
                let t = tools[(next_rand() as usize) % tools.len()];
                app.set_tool(t);
            }
            1 | 2 => {
                let x = (next_rand() % 128) as u32;
                let y = (next_rand() % 128) as u32;
                let _ = app.on_mouse_down(x, y);
            }
            3 => {
                let x = (next_rand() % 128) as u32;
                let y = (next_rand() % 128) as u32;
                let _ = app.on_mouse_move(x, y);
            }
            4 => {
                let _ = app.on_mouse_up();
            }
            5 => {
                app.add_new_layer();
            }
            6 => {
                if app.engine.store().layers.len() > 1 {
                    app.delete_active_layer();
                }
            }
            7 | 8 => {

                let _ = app.on_mouse_up(); 
                
                let pre_undo = get_state_hash(&app);
                app.undo();

                app.redo();
                let post_redo = get_state_hash(&app);
                
                if pre_undo != post_redo {
                    panic!("Consistency Error at iter {}: Redo(Undo(State)) != State", i);
                }
            }
            _ => {
                app.redo();
            }
        }
    }
}

#[test]
fn test_high_zoom_accuracy() {
    let mut app = AppState::new();
    app.view.update_viewport(1000.0, 1000.0);
    app.view.zoom_level = 100.0;
    app.view.pan_x = 0.0;
    app.view.pan_y = 0.0;

    let store = app.engine.store();
    let center_canvas = app.view.screen_to_canvas(store, 500.0, 500.0).unwrap();
    assert_eq!(center_canvas, (64, 64));

    let pixel_step = 100.0;
    let next_pixel = app.view.screen_to_canvas(store, 500.0 + pixel_step, 500.0).unwrap();
    assert_eq!(next_pixel, (65, 64));

    let sub_pixel = app.view.screen_to_canvas(store, 500.0 + 49.9, 500.0 + 49.9).unwrap();
    assert_eq!(sub_pixel, (64, 64));
}

#[test]
fn test_sparse_storage_efficiency() {
    let mut layer = Layer::new("huge".into(), "Huge".into(), 10000, 10000);
    assert_eq!(layer.chunks_count(), 0);
    layer.set_pixel(0, 0, Color::new(1, 1, 1, 255)).unwrap();
    layer.set_pixel(9999, 9999, Color::new(1, 1, 1, 255)).unwrap();
    assert_eq!(layer.chunks_count(), 2);
}

#[test]
fn test_sparse_layer_rect_io() {
    let mut layer = Layer::new("sparse".into(), "Sparse".into(), 1000, 1000);
    let rect_data = vec![255u8; 16]; 
    layer.set_rect_data(100, 100, 2, 2, &rect_data);
    
    assert!(layer.chunks_count() >= 1);
    assert!(layer.chunks_count() <= 4);

    let far_rect = layer.get_rect_data(500, 500, 10, 10);
    assert!(far_rect.iter().all(|&b| b == 0));
    assert!(layer.chunks.get(&(500/64, 500/64)).is_none());
}