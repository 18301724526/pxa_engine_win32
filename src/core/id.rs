use std::sync::atomic::{AtomicUsize, Ordering};

/// 抽象 ID 生成服务，支持依赖注入
pub trait IdGenerator: Send + Sync {
    fn generate(&self) -> String;
}

pub struct AtomicIdGenerator {
    counter: AtomicUsize,
}

impl AtomicIdGenerator {
    pub fn new(start: usize) -> Self {
        Self { counter: AtomicUsize::new(start) }
    }
}

impl IdGenerator for AtomicIdGenerator {
    fn generate(&self) -> String {
        self.counter.fetch_add(1, Ordering::SeqCst).to_string()
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::sync::Mutex;

    pub struct MockIdGenerator {
        pub prefix: String,
        pub counter: Mutex<usize>,
    }

    impl MockIdGenerator {
        pub fn new(prefix: &str) -> Self {
            Self { prefix: prefix.to_string(), counter: Mutex::new(1) }
        }
    }

    impl IdGenerator for MockIdGenerator {
        fn generate(&self) -> String {
            let mut count = self.counter.lock().unwrap();
            let id = format!("{}_{}", self.prefix, *count);
            *count += 1;
            id
        }
    }

    #[test]
    fn test_mock_id_generator() {
        let gen = MockIdGenerator::new("mock");
        assert_eq!(gen.generate(), "mock_1");
        assert_eq!(gen.generate(), "mock_2");
    }

    #[test]
    fn test_atomic_id_generator() {
        let gen = AtomicIdGenerator::new(100);
        assert_eq!(gen.generate(), "100");
        assert_eq!(gen.generate(), "101");
    }
}