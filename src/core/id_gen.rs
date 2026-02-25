use std::sync::atomic::{AtomicUsize, Ordering};

static GLOBAL_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn gen_id() -> String {
    GLOBAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst).to_string()
}