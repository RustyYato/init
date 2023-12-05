//! A construct to iteratively initialize a slice in place

use core::{mem::ManuallyDrop, ptr::NonNull};

use crate::{
    iter::UninitIter,
    ptr::{Init, Uninit},
};

/// A construct to iteratively initialize a slice in place
pub struct SliceWriter<'a, T> {
    iter: UninitIter<'a, T>,
    start: NonNull<T>,
    init: usize,
    poisoned: bool,
}

impl<'a, T> SliceWriter<'a, T> {
    /// Create a new writer from the slice
    pub fn new(u: Uninit<'a, [T]>) -> Self {
        let ptr = u.as_non_null();

        Self {
            iter: u.into_iter(),
            start: ptr.cast(),
            init: 0,
            poisoned: false,
        }
    }

    /// The number of remaining elements to initialize or zero if the writer is poisoned
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        if self.poisoned {
            0
        } else {
            self.iter.len()
        }
    }

    /// If this writer has more elements or is poisoned
    pub fn is_complete(&self) -> bool {
        self.iter.is_empty() || self.poisoned
    }

    /// If this writer has more elements or is poisoned
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    /// Tries to initialize the next element of the slice if it exists
    /// if there are no more elements in the slice, then `Ok(false)` is returned
    /// if the next element is initialized `Ok(true)` is returned
    /// if there was an error while initializing, then the writer is poisoned
    /// and an `Err` is returned or this function panics
    pub fn try_init<Args: crate::Initializer<T>>(
        &mut self,
        args: Args,
    ) -> Result<bool, Args::Error> {
        if self.is_complete() {
            Ok(false)
        } else {
            // SAFETY: this writer is not complete
            unsafe { self.try_init_unchecked(args)? };
            Ok(true)
        }
    }

    ///
    ///
    /// # Safety
    ///
    /// This writer must not be complete
    pub unsafe fn try_init_unchecked<Args: crate::Initializer<T>>(
        &mut self,
        args: Args,
    ) -> Result<(), Args::Error> {
        debug_assert!(!self.is_complete());

        let poison = Poison(&mut self.poisoned);
        // SAFETY: This iterator isn't empty, because this writer isn't complete
        let uninit = unsafe { self.iter.next_unchecked() };
        uninit.try_init(args)?.take_ownership();
        self.init += 1;
        poison.cure();

        Ok(())
    }

    fn initialized(&self) -> *mut [T] {
        core::ptr::NonNull::slice_from_raw_parts(self.start, self.init).as_ptr()
    }

    /// Extract all elements of the slice which were initialized
    /// note: if this writer is poisoned, this may not be all the elements of the slice
    pub fn finish(self) -> Init<'a, [T]> {
        let this = ManuallyDrop::new(self);

        // SAFETY: This ptr is derived from an `Uninit` and `init` is only incremented
        // for every initialized element of the slice
        unsafe { Uninit::from_raw(this.initialized()).assume_init() }
    }
}

impl<T> Drop for SliceWriter<'_, T> {
    fn drop(&mut self) {
        // SAFETY: initialized represents all the values of the slice writer which were initialized
        unsafe { self.initialized().drop_in_place() }
    }
}

struct Poison<'a>(&'a mut bool);

impl<'a> Poison<'a> {
    pub fn cure(self) {
        core::mem::forget(self);
    }
}

impl Drop for Poison<'_> {
    fn drop(&mut self) {
        *self.0 = true;
    }
}
