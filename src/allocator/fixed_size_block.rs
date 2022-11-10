#![allow(dead_code)]
use core::{
    alloc::{GlobalAlloc, Layout},
    mem::{align_of, size_of},
    ptr::NonNull,
};

use linked_list_allocator::Heap;
use spin::Mutex;

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512];

struct ListNode {
    next: Option<&'static mut ListNode>,
}

pub struct FixedSizedBlockAllocator {
    linked_lists: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: Heap,
}

pub struct Locked<T> {
    item: Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(data: T) -> Locked<T> {
        Locked {
            item: Mutex::new(data),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.item.lock()
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizedBlockAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) {
            // found a list - we can use it for allocation
            Some(index) => match allocator.linked_lists[index].take() {
                // use memory from list
                Some(node) => {
                    allocator.linked_lists[index] = node.next.take();
                    node as *mut ListNode as *mut u8
                }
                // allocate memory as the list is not created yet
                None => {
                    let block_size = BLOCK_SIZES[index];
                    let block_align = block_size;
                    let layout = Layout::from_size_align(block_size, block_align).unwrap();
                    allocator.fallback_alloc(layout)
                }
            },
            // list not found - use fall back
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.linked_lists[index].take(),
                };
                let block_size = BLOCK_SIZES[index];
                assert!(size_of::<ListNode>() <= block_size);
                assert!(align_of::<ListNode>() <= block_size);

                let node_ptr = ptr as *mut ListNode;
                node_ptr.write(new_node);
                allocator.linked_lists[index] = Option::Some(&mut *node_ptr);
            }
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout)
            }
        }
    }
}

fn list_index(layout: &Layout) -> Option<usize> {
    let block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| block_size <= s)
}

impl FixedSizedBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizedBlockAllocator {
            linked_lists: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: Heap::empty(),
        }
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        use core::ptr;
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }

    pub unsafe fn init(&mut self, heap_bottom: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_bottom, heap_size);
    }
}
