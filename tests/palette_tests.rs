use pxa_engine_win32::app::state::AppState;
use pxa_engine_win32::app::commands::*; // 更新引入
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::format::hex_palette::{load_from_hex, save_to_hex};
use std::env;
use std::fs;

fn exec(app: &mut AppState, cmd: Box<dyn pxa_engine_win32::app::command_handler::Command>) {
    app.enqueue_command(cmd);
    app.process_commands();
}

fn setup_palette_test() -> AppState {
    AppState::new()
}

#[test]
fn test_palette_interactive_management() {
    let mut app = setup_palette_test();
    let initial_count = app.pixel.engine.store().palette.colors.len(); 

    let new_color = Color::new(12, 34, 56, 255);
    exec(&mut app, Box::new(AddColorToPaletteCmd(new_color)));
    assert_eq!(app.pixel.engine.store().palette.colors.len(), initial_count + 1);

    exec(&mut app, Box::new(AddColorToPaletteCmd(new_color)));
    assert_eq!(app.pixel.engine.store().palette.colors.len(), initial_count + 1);

    let color_at_1_before_del = app.pixel.engine.store().palette.colors[1];
    exec(&mut app, Box::new(RemovePaletteColorCmd(0)));
    assert_eq!(app.pixel.engine.store().palette.colors.len(), initial_count);
    assert_eq!(app.pixel.engine.store().palette.colors[0], color_at_1_before_del);

    let target_color = app.pixel.engine.store().palette.colors[5];
    exec(&mut app, Box::new(SetPrimaryColorCmd(target_color)));
    assert_eq!(app.pixel.engine.store().primary_color, target_color);
}

#[test]
fn test_palette_io_and_error_handling() {
    let mut path = env::temp_dir();
    path.push("test_robust_palette.hex");

    let dirty_hex_content = "\
// This is a comment line
FF0000

#00FF00
INVALID_STRING_FORMAT
0000FF
    "; 

    fs::write(&path, dirty_hex_content).expect("Failed to write mock hex file");

    let loaded_palette = load_from_hex(&path).expect("加载非法数据文件时不应抛出严重错误");
    
    assert_eq!(loaded_palette.colors.len(), 3);
    assert_eq!(loaded_palette.colors[0], Color::new(255, 0, 0, 255));
    assert_eq!(loaded_palette.colors[1], Color::new(0, 255, 0, 255));
    assert_eq!(loaded_palette.colors[2], Color::new(0, 0, 255, 255));

    let mut export_path = env::temp_dir();
    export_path.push("test_export_palette.hex");
    
    save_to_hex(&export_path, &loaded_palette).expect("保存失败");
    let exported_content = fs::read_to_string(&export_path).expect("读取导出文件失败");
    
    let expected_export = "FF0000\n00FF00\n0000FF\n";
    assert_eq!(exported_content.replace("\r\n", "\n"), expected_export);

    let _ = fs::remove_file(path);
    let _ = fs::remove_file(export_path);
}