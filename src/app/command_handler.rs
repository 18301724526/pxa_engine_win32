use crate::app::state::AppState;
use std::collections::VecDeque;

pub enum AppEvent {
    ShowExitModal,
    CloseWindow,
    ShowError(String),
    DragWindow,
    MinimizeWindow,
    MaximizeWindow,
}

pub trait Command: Send + Sync {
    fn execute(&self, state: &mut AppState, events: &mut VecDeque<AppEvent>) -> Result<(), String>;
}

pub struct CommandBus {
    queue: VecDeque<Box<dyn Command>>,
    pub events: VecDeque<AppEvent>,
}

impl CommandBus {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            events: VecDeque::new(),
        }
    }

    pub fn dispatch(&mut self, cmd: Box<dyn Command>) {
        self.queue.push_back(cmd);
    }

    pub fn append_from(&mut self, other: &mut Self) {
        self.events.append(&mut other.events);
        self.queue.append(&mut other.queue);
    }

    pub fn process_all(&mut self, state: &mut AppState) {
        let mut current_queue = std::mem::take(&mut self.queue);
        while let Some(cmd) = current_queue.pop_front() {
            if let Err(e) = cmd.execute(state, &mut self.events) {
                self.events.push_back(AppEvent::ShowError(e));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppMode;

    struct TestCmd;
    impl Command for TestCmd {
        fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
            state.mode = AppMode::Animation;
            Ok(())
        }
    }

    #[test]
    fn test_command_bus_routing() {
        let mut state = AppState::new();
        state.command_bus.dispatch(Box::new(TestCmd));
        state.process_commands();
        assert_eq!(state.mode, AppMode::Animation);
    }

    struct BugReproduceCmd;
    impl Command for BugReproduceCmd {
        fn execute(&self, state: &mut AppState, _events: &mut VecDeque<AppEvent>) -> Result<(), String> {
            // 模拟 import_image() 的行为：绕过局部变量，直接向全局 state 压入新事件
            state.command_bus.events.push_back(AppEvent::CloseWindow);
            
            // 顺便模拟一下连带产生的新命令
            struct DummyCmd;
            impl Command for DummyCmd {
                fn execute(&self, _s: &mut AppState, _e: &mut VecDeque<AppEvent>) -> Result<(), String> { Ok(()) }
            }
            state.command_bus.dispatch(Box::new(DummyCmd));
            Ok(())
        }
    }

    #[test]
    fn test_process_commands_does_not_swallow_events() {
        let mut state = AppState::new();
        
        // 派发会直接操作 state 的命令
        state.command_bus.dispatch(Box::new(BugReproduceCmd));
        
        // 触发执行核心循环
        state.process_commands();
        
        // 如果 Bug 存在，这里的长度将是 0，断言必定失败（Panic）！
        assert_eq!(state.command_bus.events.len(), 1, "【BUG曝光】：执行期间产生的新事件被 process_commands 覆盖吞噬了！");
        assert_eq!(state.command_bus.queue.len(), 1, "【BUG曝光】：执行期间产生的新命令被 process_commands 覆盖吞噬了！");
    }
}

#[test]
    fn test_pending_import_image_survives_command_bus() {
        let mut state = AppState::new();

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(1, 1));
        state.pending_import_image = Some(img);

        struct DummyCmd;
        impl Command for DummyCmd {
            fn execute(&self, _s: &mut AppState, _e: &mut VecDeque<AppEvent>) -> Result<(), String> { Ok(()) }
        }
        state.command_bus.dispatch(Box::new(DummyCmd));
        state.process_commands();

        assert!(
            state.pending_import_image.is_some(), 
            "【架构防线】：挂载在状态机的图片必须存活，绝不能再像以前那样被事件总线吞噬！"
        );
    }