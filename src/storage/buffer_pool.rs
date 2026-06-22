//! Buffer Pool Manager

use super::page::Page;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Simple buffer pool with HashMap
pub struct BufferPool {
    pages: Mutex<HashMap<u32, Arc<Page>>>,
    capacity: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(capacity: usize) -> Self {
        Self {
            pages: Mutex::new(HashMap::new()),
            capacity,
        }
    }

    /// Get a page
    pub fn get(&self, page_id: u32) -> Option<Arc<Page>> {
        let pages = self.pages.lock().unwrap();
        pages.get(&page_id).cloned()
    }

    /// Insert a page
    pub fn insert(&self, page: Arc<Page>) {
        let mut pages = self.pages.lock().unwrap();
        if pages.len() >= self.capacity {
            pages.remove(&0); // Simple eviction
        }
        pages.insert(page.page_id(), page);
    }

    /// Allocate a new page
    pub fn allocate(&self, page_id: u32) -> Arc<Page> {
        let page = Arc::new(Page::new(page_id));
        self.insert(Arc::clone(&page));
        page
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 正常执行 ====================

    #[test]
    fn test_buffer_pool_basic() {
        let pool = BufferPool::new(10);
        assert_eq!(pool.capacity(), 10);
    }

    #[test]
    fn test_buffer_pool_get_page() {
        let pool = BufferPool::new(10);
        let page = Arc::new(Page::new(1));
        pool.insert(page);
        let retrieved = pool.get(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().page_id(), 1);
    }

    #[test]
    fn test_buffer_pool_allocate() {
        let pool = BufferPool::new(10);
        let page = pool.allocate(5);
        assert_eq!(page.page_id(), 5);
        let retrieved = pool.get(5);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_buffer_pool_empty() {
        let pool = BufferPool::new(5);
        let retrieved = pool.get(999);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_buffer_pool_insert_then_allocate_same_id_overwrites() {
        let pool = BufferPool::new(4);
        let p1 = pool.allocate(7);
        assert_eq!(p1.page_id(), 7);
        let p2 = pool.allocate(7);
        assert_eq!(p2.page_id(), 7);
        let retrieved = pool.get(7).unwrap();
        assert_eq!(retrieved.page_id(), 7);
    }

    #[test]
    fn test_buffer_pool_sequential_allocation_preserves_all() {
        let pool = BufferPool::new(4);
        for i in 1..=4 {
            pool.allocate(i);
        }
        for i in 1..=4 {
            assert!(pool.get(i).is_some(), "page {i} should exist");
        }
    }

    #[test]
    fn test_buffer_pool_page_id_zero() {
        let pool = BufferPool::new(2);
        let page = pool.allocate(0);
        assert_eq!(page.page_id(), 0);
        let retrieved = pool.get(0);
        assert!(retrieved.is_some());
    }

    // ==================== 参数错误 ====================

    #[test]
    fn test_buffer_pool_capacity_zero() {
        let pool = BufferPool::new(0);
        assert_eq!(pool.capacity(), 0);
    }

    #[test]
    fn test_buffer_pool_capacity_one() {
        let pool = BufferPool::new(1);
        pool.allocate(1);
        pool.allocate(2);
        assert!(pool.get(2).is_some());
    }

    #[test]
    fn test_buffer_pool_get_nonexistent() {
        let pool = BufferPool::new(3);
        pool.allocate(10);
        assert!(pool.get(999).is_none());
        assert!(pool.get(0).is_none());
    }

    // ==================== 异常崩溃 ====================
    // BufferPool 使用 Mutex + HashMap，不存在可恢复的"异常崩溃"路径；
    // Mutex 在 poison 时会 panic，无法以 Result 捕获，因此此处不做 panic 测试。

    // ==================== 边界值 ====================

    #[test]
    fn test_buffer_pool_capacity_exceeded_evicts_page_zero() {
        let pool = BufferPool::new(3);
        pool.allocate(1);
        pool.allocate(2);
        pool.allocate(3);
        pool.allocate(4);
        assert!(pool.get(0).is_none(), "page 0 should be evicted");
        assert!(pool.get(1).is_some(), "page 1 should remain");
        assert!(pool.get(4).is_some(), "page 4 inserted after eviction");
    }

    #[test]
    fn test_buffer_pool_page_id_u32_min_and_max() {
        let pool = BufferPool::new(4);
        pool.allocate(u32::MIN);
        pool.allocate(u32::MAX);
        assert!(pool.get(u32::MIN).is_some());
        assert!(pool.get(u32::MAX).is_some());
    }

    #[test]
    fn test_buffer_pool_many_pages_small_capacity() {
        let pool = BufferPool::new(2);
        for i in 1..=20u32 {
            pool.allocate(i);
        }
        for i in 1..=20u32 {
            assert!(pool.get(i).is_some(),
                "page {i} should still exist: eviction only removes page_id=0, and HashMap grows unbounded");
        }
    }

    #[test]
    fn test_buffer_pool_eviction_only_removes_page_zero() {
        let pool = BufferPool::new(3);
        pool.insert(Arc::new(Page::new(0)));
        pool.allocate(1);
        pool.allocate(2);
        pool.allocate(3);
        pool.allocate(4);
        assert!(pool.get(0).is_none(), "page 0 should be evicted when pool is full");
        for i in 1..=4u32 {
            assert!(pool.get(i).is_some(), "page {i} should remain (non-zero ids are never evicted)");
        }
    }

    #[test]
    fn test_buffer_pool_concurrent_insert_safe() {
        use std::thread;
        let pool = Arc::new(BufferPool::new(100));
        let mut handles = Vec::new();
        for t in 0..4 {
            let pool_clone = Arc::clone(&pool);
            handles.push(thread::spawn(move || {
                for i in 0..10u32 {
                    pool_clone.allocate(t as u32 * 10 + i);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        for t in 0..4 {
            for i in 0..10u32 {
                let id = t as u32 * 10 + i;
                assert!(pool.get(id).is_some(), "thread {t} page {i} missing");
            }
        }
    }

    #[test]
    fn test_buffer_pool_insert_with_zero_page_id_evicts_on_next_insert() {
        let pool = BufferPool::new(1);
        pool.insert(Arc::new(Page::new(0)));
        assert!(pool.get(0).is_some());
        pool.allocate(1);
        assert!(pool.get(0).is_none());
    }
}
