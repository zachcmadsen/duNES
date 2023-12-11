use std::{
    cell::{Cell, UnsafeCell},
    mem::ManuallyDrop,
    slice,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

const INDEX_MASK: u8 = 0b00000011;
const UPDATE_FLAG_MASK: u8 = 0b00000100;

// TODO: Add unit tests (with loom?).
pub struct TripleBuffer {
    buffers: [UnsafeCell<*mut u8>; 3],
    capacity: usize,
    back_index: AtomicU8,
}

unsafe impl Send for TripleBuffer {}
unsafe impl Sync for TripleBuffer {}

impl Drop for TripleBuffer {
    fn drop(&mut self) {
        for buffer in &self.buffers {
            let ptr = unsafe { *buffer.get() };
            unsafe {
                Vec::from_raw_parts(ptr, 0, self.capacity);
            }
        }
    }
}

impl TripleBuffer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(capacity: usize) -> (Writer, Reader) {
        const INIT_BACK_INDEX: u8 = 0;
        const INIT_WRITE_INDEX: u8 = 1;
        const INIT_READ_INDEX: u8 = 2;

        let tb = Arc::new(TripleBuffer {
            buffers: [
                UnsafeCell::new(
                    ManuallyDrop::new(vec![0; capacity]).as_mut_ptr(),
                ),
                UnsafeCell::new(
                    ManuallyDrop::new(vec![0; capacity]).as_mut_ptr(),
                ),
                UnsafeCell::new(
                    ManuallyDrop::new(vec![0; capacity]).as_mut_ptr(),
                ),
            ],
            capacity,
            back_index: AtomicU8::new(INIT_BACK_INDEX),
        });

        (
            Writer { tb: tb.clone(), index: INIT_WRITE_INDEX },
            Reader { tb, index: Cell::new(INIT_READ_INDEX) },
        )
    }
}

pub struct Writer {
    tb: Arc<TripleBuffer>,
    index: u8,
}

impl Writer {
    pub fn get_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                *self.tb.buffers[self.index as usize].get(),
                self.tb.capacity,
            )
        }
    }

    pub fn swap(&mut self) {
        let new_back_index = self.index | UPDATE_FLAG_MASK;
        // TODO: Figure out better memory orderings for the swaps. They're all
        // SeqCst because it's probably correct, but I'm leaving performance on
        // the table.
        let old_back_index =
            self.tb.back_index.swap(new_back_index, Ordering::SeqCst);
        self.index = old_back_index & INDEX_MASK;
    }
}

pub struct Reader {
    tb: Arc<TripleBuffer>,
    index: Cell<u8>,
}

impl Reader {
    pub fn get(&self) -> &[u8] {
        // Swap the read and back buffers if there's an update.
        if self.tb.back_index.load(Ordering::SeqCst) & UPDATE_FLAG_MASK != 0 {
            let new_back_index = self.index.get();
            let old_back_index =
                self.tb.back_index.swap(new_back_index, Ordering::SeqCst);
            self.index.set(old_back_index & INDEX_MASK);
        }

        unsafe {
            slice::from_raw_parts(
                *self.tb.buffers[self.index.get() as usize].get(),
                self.tb.capacity,
            )
        }
    }
}
