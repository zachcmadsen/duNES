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
pub struct TripleBuffer<T> {
    buffers: [UnsafeCell<T>; 3],
    back_buffer_index: AtomicU8,
}

impl<T> TripleBuffer<T> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(buffer: T) -> (Writer<T>, Reader<T>)
    where
        T: Clone,
    {
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
        let writer = Writer {
            buffer: triple_buffer.clone(),
            index: INIT_WRITE_BUFFER_INDEX,
        };
        let reader = Reader {
            buffer: triple_buffer,
            index: Cell::new(INIT_READ_BUFFER_INDEX),
        };

        (writer, reader)
    }
}

pub struct Writer<T> {
    buffer: Arc<TripleBuffer<T>>,
    index: u8,
}

unsafe impl<T: Send> Send for Writer<T> {}

impl<T> Writer<T> {
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.buffer.buffers[self.index as usize].get() }
    }

    pub fn swap(&mut self) {
        let new_back_index = self.index | UPDATE_FLAG_MASK;
        // TODO: Figure out better memory orderings for the swaps. They're all
        // SeqCst because it's probably correct, but I'm leaving performance on
        // the table.
        let old_back_index = self
            .buffer
            .back_buffer_index
            .swap(new_back_index, Ordering::SeqCst);
        self.index = old_back_index & INDEX_MASK;
    }
}

pub struct Reader<T> {
    buffer: Arc<TripleBuffer<T>>,
    index: Cell<u8>,
}

unsafe impl<T: Send> Send for Reader<T> {}

impl<T> Reader<T> {
    pub fn get(&self) -> &T {
        // Swap the read and back buffers if there's an update.
        if self.buffer.back_buffer_index.load(Ordering::SeqCst)
            & UPDATE_FLAG_MASK
            != 0
        {
            let new_back_index = self.index.get();
            let old_back_index = self
                .buffer
                .back_buffer_index
                .swap(new_back_index, Ordering::SeqCst);
            self.index.set(old_back_index & INDEX_MASK);
        }

        unsafe { &*self.buffer.buffers[self.index.get() as usize].get() }
    }
}
