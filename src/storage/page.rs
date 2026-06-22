//! Page management for storage engine

/// Page structure
#[derive(Debug, Clone)]
pub struct Page {
    pub page_id: u32,
    pub data: Vec<u8>,
}

impl Page {
    /// Create a new page
    pub fn new(page_id: u32) -> Self {
        Self {
            page_id,
            data: vec![0u8; 4096],
        }
    }

    /// Get page ID
    pub fn page_id(&self) -> u32 {
        self.page_id
    }

    /// Get page size
    pub fn size() -> usize {
        4096
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 正常执行 ====================

    #[test]
    fn test_page_creation() {
        let page = Page::new(1);
        assert_eq!(page.page_id(), 1);
        assert_eq!(page.data.len(), 4096);
    }

    #[test]
    fn test_page_data_access() {
        let mut page = Page::new(1);
        page.data[0] = 0xAB;
        page.data[1] = 0xCD;
        assert_eq!(page.data[0], 0xAB);
        assert_eq!(page.data[1], 0xCD);
    }

    #[test]
    fn test_page_default_size() {
        assert_eq!(Page::size(), 4096);
    }

    #[test]
    fn test_page_all_zeroes_after_new() {
        let page = Page::new(42);
        for (i, &b) in page.data.iter().enumerate() {
            assert_eq!(b, 0, "byte {i} should be zero");
        }
        assert_eq!(page.page_id(), 42);
    }

    #[test]
    fn test_page_clone_indeptendent() {
        let mut p1 = Page::new(1);
        p1.data[0] = 0x01;
        let mut p2 = p1.clone();
        p2.data[0] = 0xFF;
        assert_eq!(p1.data[0], 0x01);
        assert_eq!(p2.data[0], 0xFF);
    }

    #[test]
    fn test_page_write_full_buffer_then_read() {
        let mut page = Page::new(7);
        for (i, b) in page.data.iter_mut().enumerate() {
            *b = (i as u8).wrapping_add(1);
        }
        for (i, &b) in page.data.iter().enumerate() {
            assert_eq!(b, (i as u8).wrapping_add(1));
        }
    }

    #[test]
    fn test_page_data_slice_access() {
        let mut page = Page::new(1);
        let block: [u8; 8] = *b"\xAA\xBB\xCC\xDD\xEE\xFF\x11\x22";
        page.data[8..16].copy_from_slice(&block);
        assert_eq!(&page.data[8..16], &block);
    }

    // ==================== 参数错误 ====================
    // Page::new 只接受 u32，无"非法参数"分支，所有 u32 都是合法 page_id。

    // ==================== 异常崩溃 ====================

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_page_data_index_oob_panics() {
        let page = Page::new(1);
        let _ = page.data[4096];
    }

    #[test]
    #[should_panic(expected = "does not match destination slice length")]
    fn test_page_data_slice_oob_panics() {
        let mut page = Page::new(1);
        let buf = [0u8; 16];
        page.data[4090..4096].copy_from_slice(&buf);
    }

    // ==================== 边界值 ====================

    #[test]
    fn test_page_id_zero() {
        let page = Page::new(0);
        assert_eq!(page.page_id(), 0);
        assert_eq!(page.data.len(), 4096);
    }

    #[test]
    fn test_page_id_u32_max() {
        let page = Page::new(u32::MAX);
        assert_eq!(page.page_id(), u32::MAX);
        assert_eq!(page.data.len(), 4096);
    }

    #[test]
    fn test_page_first_and_last_byte() {
        let mut page = Page::new(1);
        page.data[0] = 0x00;
        page.data[4095] = 0xFF;
        assert_eq!(page.data[0], 0x00);
        assert_eq!(page.data[4095], 0xFF);
    }

    #[test]
    fn test_page_u16_and_u32_misaligned_writes() {
        let mut page = Page::new(1);
        let u16_le: [u8; 2] = 0xBEEF_u16.to_le_bytes();
        let u32_le: [u8; 4] = 0xDEADBEEF_u32.to_le_bytes();
        page.data[0..2].copy_from_slice(&u16_le);
        page.data[2..6].copy_from_slice(&u32_le);
        assert_eq!(u16::from_le_bytes([page.data[0], page.data[1]]), 0xBEEF);
        assert_eq!(
            u32::from_le_bytes([page.data[2], page.data[3], page.data[4], page.data[5]]),
            0xDEADBEEF
        );
    }

    #[test]
    fn test_page_size_includes_header_and_buffer() {
        let header = std::mem::size_of::<u32>();
        let vec_repr = std::mem::size_of::<Vec<u8>>();
        let page_size = std::mem::size_of::<Page>();
        assert!(page_size >= header + vec_repr,
            "Page ({page_size} bytes) must at least contain u32 ({header}) + Vec<u8> ({vec_repr})");
    }
}
