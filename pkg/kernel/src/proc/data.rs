use alloc::{collections::BTreeMap, sync::Arc};
use spin::RwLock;
use x86_64::structures::paging::{
    page::{PageRange, PageRangeInclusive},
    Page,
};

use crate::resource::ResourceSet;
use super::*;

#[derive(Debug, Clone)]
pub struct ProcessData {
    // shared data
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,
    // pub kernel_stack_range: (VirtAddr, VirtAddr), // 栈范围
    // pub memory_usage: u64, // 内存使用量
    // pub code_pages: usize, // 代码页数
    // pub code_start: VirtAddr, // 代码起始地址
    pub(super) resources: Arc<RwLock<ResourceSet>>, // 文件符描述表
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            // memory_usage: 0,
            // code_pages: 0,
            // code_start: VirtAddr::new(0),
            resources: Arc::new(RwLock::new(ResourceSet::default())),
        }
    }
}

impl ProcessData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.resources.read().read(fd, buf)
    }
    
    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        self.resources.read().write(fd, buf)
    }

    pub fn env(&self, key: &str) -> Option<String> {
        self.env.read().get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.env.write().insert(key.into(), val.into());
    }

    // pub fn set_memory_usage(&mut self, usage: u64) {
    //     self.memory_usage = usage;
    // }

    // pub fn set_code_pages(&mut self, pages: usize) {
    //     self.code_pages = pages;
    // }

    // pub fn set_code_start(&mut self, start: VirtAddr) {
    //     self.code_start = start;
    // }

    // pub fn memory_usage(&self) -> u64 {
    //     self.memory_usage
    // }

    // pub fn code_pages(&self) -> usize {
    //     self.code_pages
    // }

    // pub fn code_start(&self) -> VirtAddr {
    //     self.code_start
    // }
    // pub fn code_range(&self) -> PageRangeInclusive {
    //     Page::range_inclusive(
    //         Page::containing_address(self.code_start),
    //         Page::containing_address(self.code_start + self.code_pages as u64 * crate::memory::PAGE_SIZE - 1),
    //     )
    // }

    // pub fn code_range_size(&self) -> usize {
    //     self.code_pages * crate::memory::PAGE_SIZE
    // }

    // pub fn code_range_start(&self) -> VirtAddr {
    //     self.code_start
    // }

    // pub fn code_range_end(&self) -> VirtAddr {
    //     self.code_start + self.code_pages as u64 * crate::memory::PAGE_SIZE - 1
    // }
}
