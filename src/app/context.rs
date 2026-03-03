use crate::core::store::PixelStore;
use crate::history::manager::HistoryManager;
use crate::core::id::IdGenerator;

pub struct CanvasContext<'a> {
    pub store: &'a mut PixelStore,
    pub history: &'a mut HistoryManager,
    pub id_gen: &'a dyn IdGenerator,
}