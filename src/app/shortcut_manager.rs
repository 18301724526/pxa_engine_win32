use crate::app::command_handler::Command;
use crate::app::commands::*;
use crate::app::state::{AppMode, ToolType};
use std::collections::HashMap;

pub struct ShortcutManager {
    pixel_shortcuts: HashMap<String, Box<dyn Fn() -> Box<dyn Command> + Send + Sync>>,
    anim_shortcuts: HashMap<String, Box<dyn Fn() -> Box<dyn Command> + Send + Sync>>,
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
    pub fn bind_pixel_shortcut<F: Fn() -> Box<dyn Command> + Send + Sync + 'static>(&mut self, key: &str, f: F) {
        self.pixel_shortcuts.insert(key.to_string(), Box::new(f));
    }

    /// 动态绑定动画模式的快捷键
    pub fn bind_anim_shortcut<F: Fn() -> Box<dyn Command> + Send + Sync + 'static>(&mut self, key: &str, f: F) {
        self.anim_shortcuts.insert(key.to_string(), Box::new(f));
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
        self.bind_pixel_shortcut("[", || Box::new(ChangeBrushSizeCmd(-1)));
        self.bind_pixel_shortcut("]", || Box::new(ChangeBrushSizeCmd(1)));
        
        // 工具切换
        self.bind_pixel_shortcut("p", || Box::new(SelectToolCmd(ToolType::Pencil)));
        self.bind_pixel_shortcut("e", || Box::new(SelectToolCmd(ToolType::Eraser)));
        self.bind_pixel_shortcut("b", || Box::new(SelectToolCmd(ToolType::Bucket)));
        self.bind_pixel_shortcut("t", || Box::new(SelectToolCmd(ToolType::Transform)));
        self.bind_pixel_shortcut("c", || Box::new(SelectToolCmd(ToolType::Pen)));

        self.bind_anim_shortcut("c", || Box::new(SelectToolCmd(ToolType::BoneRotate)));
        self.bind_anim_shortcut("v", || Box::new(SelectToolCmd(ToolType::BoneTranslate)));
    }

    /// 根据当前模式和输入的字符，返回对应的命令
    pub fn handle_text_input(&self, text: &str, mode: AppMode) -> Option<Box<dyn Command>> {
        let map = match mode {
            AppMode::PixelEdit => &self.pixel_shortcuts,
            AppMode::Animation => &self.anim_shortcuts,
        };
        map.get(text).map(|f| f())
    }
}