use crate::core::store::PixelStore;
use crate::history::manager::HistoryManager;

pub struct CanvasContext<'a> {
    pub store: &'a mut PixelStore,
    pub history: &'a mut HistoryManager,
}