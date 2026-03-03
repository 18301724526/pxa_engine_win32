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
}