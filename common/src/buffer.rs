use std::{
    cell::{Cell, UnsafeCell},
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

const INIT_BACK_IDX: u8 = 0;
const INIT_WRITER_IDX: u8 = 1;
const INIT_READER_IDX: u8 = 2;

const UPDATE_FLAG: u8 = 0b00000100;
const INDEX_MASK: u8 = 0b00000011;

struct TripleBuffer<T> {
    bufs: [UnsafeCell<T>; 3],
    back_idx: AtomicU8,
}

pub fn triple_buffer<T: Clone>(val: T) -> (Writer<T>, Reader<T>) {
    let buf = Arc::new(TripleBuffer {
        bufs: [
            UnsafeCell::new(val.clone()),
            UnsafeCell::new(val.clone()),
            UnsafeCell::new(val),
        ],
        back_idx: AtomicU8::new(INIT_BACK_IDX),
    });
    let writer = Writer { buf: buf.clone(), idx: INIT_WRITER_IDX };
    let reader = Reader { buf, idx: Cell::new(INIT_READER_IDX) };
    (writer, reader)
}

pub struct Writer<T> {
    buf: Arc<TripleBuffer<T>>,
    idx: u8,
}

unsafe impl<T: Send> Send for Writer<T> {}

impl<T> Writer<T> {
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.buf.bufs[self.idx as usize].get() }
    }

    pub fn swap(&mut self) {
        let new_back_index = self.idx | UPDATE_FLAG;
        let old_back_index =
            self.buf.back_idx.swap(new_back_index, Ordering::AcqRel);
        self.idx = old_back_index & INDEX_MASK;
    }
}

pub struct Reader<T> {
    buf: Arc<TripleBuffer<T>>,
    idx: Cell<u8>,
}

unsafe impl<T: Send> Send for Reader<T> {}

impl<T> Reader<T> {
    pub fn get(&self) -> &T {
        // Update the reader index if the writer updated the back buffer.
        if self.buf.back_idx.load(Ordering::Relaxed) & UPDATE_FLAG != 0 {
            let new_back_index = self.idx.get();
            let old_back_index =
                self.buf.back_idx.swap(new_back_index, Ordering::AcqRel);
            self.idx.set(old_back_index & INDEX_MASK);
        }
        unsafe { &*self.buf.bufs[self.idx.get() as usize].get() }
    }
}
