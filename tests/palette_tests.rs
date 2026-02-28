use pxa_engine_win32::app::state::AppState;
use pxa_engine_win32::app::commands::AppCommand;
use pxa_engine_win32::app::command_handler::CommandHandler;
use pxa_engine_win32::core::color::Color;
use pxa_engine_win32::format::hex_palette::{load_from_hex, save_to_hex};
use std::env;
use std::fs;

fn setup_palette_test() -> AppState {
    AppState::new()
}

// ---------------------------------------------------------
// 1. 添加颜色 (去重) & 2. 删除颜色 & 3. 选择颜色
// ---------------------------------------------------------
#[test]
fn test_palette_interactive_management() {
    let mut app = setup_palette_test();
    let initial_count = app.engine.store().palette.colors.len(); // 默认 Pico-8 有 16 色

    // 1. 添加颜色
    let new_color = Color::new(12, 34, 56, 255);
    CommandHandler::execute(&mut app, AppCommand::AddColorToPalette(new_color));
    assert_eq!(app.engine.store().palette.colors.len(), initial_count + 1, "应该成功添加新颜色");

    // 1. 添加重复颜色 (验证去重逻辑)
    CommandHandler::execute(&mut app, AppCommand::AddColorToPalette(new_color));
    assert_eq!(app.engine.store().palette.colors.len(), initial_count + 1, "重复的颜色应该被去重，数量不增加");

    // 2. 删除颜色 (删除索引为 0 的颜色)
    let color_at_1_before_del = app.engine.store().palette.colors[1];
    CommandHandler::execute(&mut app, AppCommand::RemovePaletteColor(0));
    assert_eq!(app.engine.store().palette.colors.len(), initial_count, "删除后颜色数量应该减少 1");
    assert_eq!(app.engine.store().palette.colors[0], color_at_1_before_del, "原先索引 1 的颜色应该前移");

    // 3. 选择颜色 (点击调色板颜色更新主色)
    let target_color = app.engine.store().palette.colors[5];
    CommandHandler::execute(&mut app, AppCommand::SetPrimaryColor(target_color));
    assert_eq!(app.engine.store().primary_color, target_color, "主色应该被成功更新为调色板选中的颜色");
}

// ---------------------------------------------------------
// 4. 导出 & 5. 导入 & 6. 错误处理 (忽略注释/空行/非法格式)
// ---------------------------------------------------------
#[test]
fn test_palette_io_and_error_handling() {
    let mut path = env::temp_dir();
    path.push("test_robust_palette.hex");

    // 制造一个包含各种“脏数据”的伪造 hex 文件
    let dirty_hex_content = "\
// This is a comment line
FF0000

#00FF00
INVALID_STRING_FORMAT
0000FF
    "; // 包含注释、空行、带#号的颜色、非法字符串、纯净颜色

    fs::write(&path, dirty_hex_content).expect("Failed to write mock hex file");

    // 5 & 6. 测试导入：引擎的 load_from_hex 应该能完美过滤并提纯出 3 个合法颜色，且不崩溃
    let loaded_palette = load_from_hex(&path).expect("加载非法数据文件时不应抛出严重错误，应静默过滤");
    
    assert_eq!(loaded_palette.colors.len(), 3, "应该只提取出 3 个合法的颜色");
    assert_eq!(loaded_palette.colors[0], Color::new(255, 0, 0, 255), "第一个应解析为红色");
    assert_eq!(loaded_palette.colors[1], Color::new(0, 255, 0, 255), "带有 # 号的绿色也应被正确解析");
    assert_eq!(loaded_palette.colors[2], Color::new(0, 0, 255, 255), "最后一个应解析为蓝色");

    // 4. 测试导出
    let mut export_path = env::temp_dir();
    export_path.push("test_export_palette.hex");
    
    save_to_hex(&export_path, &loaded_palette).expect("保存失败");
    let exported_content = fs::read_to_string(&export_path).expect("读取导出文件失败");
    
    // 导出的文件应该是绝对纯净、标准的 RRGGBB 格式（每行 6 字符）
    let expected_export = "FF0000\n00FF00\n0000FF\n";
    // 适配 Windows 的 CRLF 或 Linux 的 LF
    assert_eq!(exported_content.replace("\r\n", "\n"), expected_export, "导出的格式必须是纯净标准的 Hex");

    // 清理临时文件
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(export_path);
}