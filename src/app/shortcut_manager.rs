use crate::app::commands::AppCommand;
use crate::app::state::{AppMode, ToolType};
use std::collections::HashMap;

pub struct ShortcutManager {
    pixel_shortcuts: HashMap<String, AppCommand>,
    anim_shortcuts: HashMap<String, AppCommand>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut manager = Self {
            pixel_shortcuts: HashMap::new(),
            anim_shortcuts: HashMap::new(),
        };
        // 软件启动时加载默认配置
        manager.load_default_shortcuts();
        // 将来在这里可以追加: manager.load_from_user_config("shortcuts.toml");
        manager
    }

    /// 动态绑定像素模式的快捷键
    pub fn bind_pixel_shortcut(&mut self, key: &str, cmd: AppCommand) {
        self.pixel_shortcuts.insert(key.to_string(), cmd);
    }

    /// 动态绑定动画模式的快捷键
    pub fn bind_anim_shortcut(&mut self, key: &str, cmd: AppCommand) {
        self.anim_shortcuts.insert(key.to_string(), cmd);
    }

    /// 将来用于读取配置文件的接口预留
    pub fn load_from_user_config(&mut self, _config_path: &str) {
        // TODO: 解析 TOML/JSON
        // let user_config = read_toml(config_path);
        // for (key, action) in user_config.pixel_binds {
        //     self.bind_pixel_shortcut(&key, parse_action(action));
        // }
    }

    /// 默认的兜底硬编码映射（当没有配置文件时生效）
    fn load_default_shortcuts(&mut self) {
        // 画笔尺寸
        self.bind_pixel_shortcut("[", AppCommand::ChangeBrushSize(-1));
        self.bind_pixel_shortcut("]", AppCommand::ChangeBrushSize(1));
        
        // 工具切换
        self.bind_pixel_shortcut("p", AppCommand::SelectTool(ToolType::Pencil));
        self.bind_pixel_shortcut("e", AppCommand::SelectTool(ToolType::Eraser));
        self.bind_pixel_shortcut("b", AppCommand::SelectTool(ToolType::Bucket));
        self.bind_pixel_shortcut("t", AppCommand::SelectTool(ToolType::Transform));
        self.bind_pixel_shortcut("c", AppCommand::SelectTool(ToolType::Pen));

        self.bind_anim_shortcut("c", AppCommand::SelectTool(ToolType::BoneRotate));
        self.bind_anim_shortcut("v", AppCommand::SelectTool(ToolType::BoneTranslate));
    }

    /// 根据当前模式和输入的字符，返回对应的命令
    pub fn handle_text_input(&self, text: &str, mode: AppMode) -> Option<AppCommand> {
        let map = match mode {
            AppMode::PixelEdit => &self.pixel_shortcuts,
            AppMode::Animation => &self.anim_shortcuts,
        };
        map.get(text).cloned()
    }
}