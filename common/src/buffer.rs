use std::{
    cell::{Cell, UnsafeCell},
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

const INDEX_MASK: u8 = 0b00000011;
const UPDATE_FLAG_MASK: u8 = 0b00000100;

// TODO: Add unit tests (with loom?).
struct TripleBuffer<T> {
    buffers: [UnsafeCell<T>; 3],
    back_buffer_index: AtomicU8,
}

pub fn triple_buffer<T: Clone>(
    buffer: T,
) -> (TripleBufferWriter<T>, TripleBufferReader<T>) {
    const INIT_BACK_BUFFER_INDEX: u8 = 0;
    const INIT_WRITE_BUFFER_INDEX: u8 = 1;
    const INIT_READ_BUFFER_INDEX: u8 = 2;

    let triple_buffer = Arc::new(TripleBuffer {
        buffers: [
            UnsafeCell::new(buffer.clone()),
            UnsafeCell::new(buffer.clone()),
            UnsafeCell::new(buffer),
        ],
        back_buffer_index: AtomicU8::new(INIT_BACK_BUFFER_INDEX),
    });
    let writer = TripleBufferWriter {
        triple_buffer: triple_buffer.clone(),
        index: INIT_WRITE_BUFFER_INDEX,
    };
    let reader = TripleBufferReader {
        triple_buffer,
        index: Cell::new(INIT_READ_BUFFER_INDEX),
    };

    (writer, reader)
}

pub struct TripleBufferWriter<T> {
    triple_buffer: Arc<TripleBuffer<T>>,
    index: u8,
}

unsafe impl<T: Send> Send for TripleBufferWriter<T> {}

impl<T> TripleBufferWriter<T> {
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.triple_buffer.buffers[self.index as usize].get() }
    }

    pub fn swap(&mut self) {
        let new_back_index = self.index | UPDATE_FLAG_MASK;
        // TODO: Figure out better memory orderings for the swaps. They're all
        // SeqCst because it's probably correct, but I'm leaving performance on
        // the table.
        let old_back_index = self
            .triple_buffer
            .back_buffer_index
            .swap(new_back_index, Ordering::SeqCst);
        self.index = old_back_index & INDEX_MASK;
    }
}

pub struct TripleBufferReader<T> {
    triple_buffer: Arc<TripleBuffer<T>>,
    index: Cell<u8>,
}

unsafe impl<T: Send> Send for TripleBufferReader<T> {}

impl<T> TripleBufferReader<T> {
    pub fn get(&self) -> &T {
        // Swap the read and back buffers if there's an update.
        if self.triple_buffer.back_buffer_index.load(Ordering::SeqCst)
            & UPDATE_FLAG_MASK
            != 0
        {
            let new_back_index = self.index.get();
            let old_back_index = self
                .triple_buffer
                .back_buffer_index
                .swap(new_back_index, Ordering::SeqCst);
            self.index.set(old_back_index & INDEX_MASK);
        }

        unsafe {
            &*self.triple_buffer.buffers[self.index.get() as usize].get()
        }
    }
}
