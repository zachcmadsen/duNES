mod sync {
    #[cfg(loom)]
    pub use loom::cell::{Cell, UnsafeCell};
    #[cfg(loom)]
    pub use loom::sync::{atomic::AtomicU8, Arc};

    #[cfg(not(loom))]
    pub use std::cell::{Cell, UnsafeCell};
    #[cfg(not(loom))]
    pub use std::sync::{atomic::AtomicU8, Arc};
}

use std::sync::atomic::Ordering;

use sync::{Arc, AtomicU8, Cell, UnsafeCell};

const INDEX_MASK: u8 = 0b00000011;
const UPDATE_FLAG: u8 = 0b00000100;

struct TripleBuffer<T> {
    buffers: [UnsafeCell<T>; 3],
    back_index: AtomicU8,
}

unsafe impl<T> Sync for TripleBuffer<T> where T: Send {}

pub struct Writer<T> {
    _tb: Arc<TripleBuffer<T>>,
    _index: u8,
}

impl<T> Writer<T> {
    #[cfg(not(loom))]
    pub fn _get_mut(&mut self) -> &mut T {
        unsafe { &mut *self._tb.buffers[self._index as usize].get() }
    }

    pub fn _swap(&mut self) {
        let new_back_index = self._index | UPDATE_FLAG;
        let old_back_index =
            self._tb.back_index.swap(new_back_index, Ordering::AcqRel);
        self._index = old_back_index & INDEX_MASK;
    }

    #[cfg(loom)]
    pub fn with_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        unsafe {
            self._tb.buffers[self._index as usize].with_mut(|p| f(&mut *p))
        }
    }
}

pub struct Reader<T> {
    tb: Arc<TripleBuffer<T>>,
    index: Cell<u8>,
}

impl<T> Reader<T> {
    #[cfg(not(loom))]
    pub fn get(&self) -> &T {
        self.maybe_swap();
        unsafe { &*self.tb.buffers[self.index.get() as usize].get() }
    }

    /// Swaps the read and back buffers if the back buffer was updated.
    fn maybe_swap(&self) {
        if self.tb.back_index.load(Ordering::Relaxed) & UPDATE_FLAG != 0 {
            let new_back_index = self.index.get();
            let old_back_index =
                self.tb.back_index.swap(new_back_index, Ordering::AcqRel);
            self.index.set(old_back_index & INDEX_MASK);
        }
    }

    #[cfg(loom)]
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.maybe_swap();
        unsafe { self.tb.buffers[self.index.get() as usize].with(|p| f(&*p)) }
    }
}

pub fn triple_buffer<T: Clone>(buffer: T) -> (Writer<T>, Reader<T>) {
    const INIT_BACK_INDEX: u8 = 0;
    const INIT_WRITE_INDEX: u8 = 1;
    const INIT_READ_INDEX: u8 = 2;

    let tb = Arc::new(TripleBuffer {
        buffers: [
            UnsafeCell::new(buffer.clone()),
            UnsafeCell::new(buffer.clone()),
            UnsafeCell::new(buffer),
        ],
        back_index: AtomicU8::new(INIT_BACK_INDEX),
    });

    (
        Writer { _tb: tb.clone(), _index: INIT_WRITE_INDEX },
        Reader { tb, index: Cell::new(INIT_READ_INDEX) },
    )
}

#[cfg(loom)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_read_and_write() {
        loom::model(|| {
            let (mut writer, reader) = triple_buffer(0u8);

            let writer_thread = loom::thread::spawn(move || {
                writer.with_mut(|val| *val = 1);
                writer.swap();
            });

            let val = reader.with(|&val| val);
            assert!(val <= 1);

            writer_thread.join().unwrap();
        });
    }
}
