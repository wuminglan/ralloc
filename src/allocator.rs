//! The global allocator.
//!
//! This contains primitives for the cross-thread allocator.

use prelude::*;

use sync;
use bookkeeper::Bookkeeper;

/// The global default allocator.
static ALLOCATOR: sync::Mutex<Allocator> = sync::Mutex::new(Allocator::new());

/// Lock the allocator.
#[inline]
pub fn lock<'a>() -> sync::MutexGuard<'a, Allocator> {
    ALLOCATOR.lock()
}

/// An allocator.
///
/// This keeps metadata and relevant information about the allocated blocks. All allocation,
/// deallocation, and reallocation happens through this.
pub struct Allocator {
    /// The inner bookkeeper.
    inner: Bookkeeper,
}

impl Allocator {
    /// Create a new, empty allocator.
    #[inline]
    pub const fn new() -> Allocator {
        Allocator {
            inner: Bookkeeper::new(),
        }
    }

    /// Allocate a block of memory.
    ///
    /// # Errors
    ///
    /// The OOM handler handles out-of-memory conditions.
    #[inline]
    pub fn alloc(&mut self, size: usize, align: usize) -> *mut u8 {
        *Pointer::from(self.inner.alloc(size, align))
    }

    /// Free a buffer.
    ///
    /// Note that this do not have to be a buffer allocated through ralloc. The only requirement is
    /// that it is not used after the free.
    ///
    /// # Errors
    ///
    /// The OOM handler handles out-of-memory conditions.
    #[inline]
    pub unsafe fn free(&mut self, ptr: *mut u8, size: usize) {
        self.inner.free(Block::from_raw_parts(Pointer::new(ptr), size))
    }

    /// Reallocate memory.
    ///
    /// Reallocate the buffer starting at `ptr` with size `old_size`, to a buffer starting at the
    /// returned pointer with size `size`.
    ///
    /// # Errors
    ///
    /// The OOM handler handles out-of-memory conditions.
    #[inline]
    pub unsafe fn realloc(&mut self, ptr: *mut u8, old_size: usize, size: usize, align: usize) -> *mut u8 {
        *Pointer::from(self.inner.realloc(
            Block::from_raw_parts(Pointer::new(ptr), old_size),
            size,
            align
        ))
    }

    /// Try to reallocate the buffer _inplace_.
    ///
    /// In case of success, return the new buffer's size. On failure, return the old size.
    ///
    /// This can be used to shrink (truncate) a buffer as well.
    #[inline]
    pub unsafe fn realloc_inplace(&mut self, ptr: *mut u8, old_size: usize, size: usize) -> Result<(), ()> {
        if self.inner.realloc_inplace(
            Block::from_raw_parts(Pointer::new(ptr), old_size),
            size
        ).is_ok() {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Assert that no leaks are done.
    ///
    /// This should be run in the end of your program, after destructors have been run. It will then
    /// panic if some item is not freed.
    ///
    /// In release mode, this is a NOOP.
    pub fn debug_assert_no_leak(&self) {
        #[cfg(feature = "debug_tools")]
        self.inner.assert_no_leak();
    }
}
