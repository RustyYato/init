//! A custom writer type which safely initializes slices in place

use core::mem::ManuallyDrop;

use crate::{ptr::UninitSliceIter, Ctor, Init, Uninit};

/// A type which handles initializing a slice from a arbitrary sequence of initializers
///
/// This type will stop after the first initializer to error                    
pub struct SliceWriter<'brand, T> {
    ptr: Uninit<'brand, [T]>,
    iter: UninitSliceIter<'brand, T>,
    init: usize,
}

impl<T> Drop for SliceWriter<'_, T> {
    fn drop(&mut self) {
        let init =
            core::ptr::slice_from_raw_parts_mut(self.ptr.as_mut_ptr().cast::<T>(), self.init);
        // SAFETY: the SliceWriter ensures that self.ptr..self.ptr+self.init is initialized
        unsafe { init.drop_in_place() };
    }
}

impl<'brand, T> SliceWriter<'brand, T> {
    /// Create an initializer from an uninitialized slice
    pub fn new(mut uninit: Uninit<'brand, [T]>) -> Self {
        Self {
            // SAFETY: ptr is not used while iter is active
            iter: unsafe { uninit.iter_mut().unlink() },
            ptr: uninit,
            init: 0,
        }
    }

    /// The number of remaining elements to initialize
    pub fn remaining_len(&self) -> usize {
        self.iter.len()
    }

    /// try to initialize the next element with the given arguments
    ///
    /// returns Err(args) if there are no more elements to initialize
    /// returns Ok(_) with the result of the initializer if there was an element to initialize
    pub fn try_init<Args>(&mut self, args: Args) -> Result<Result<(), T::Error>, Args>
    where
        T: Ctor<Args>,
    {
        match self.iter.next() {
            Some(u) => {
                let r = u.try_init(args).map(Init::take_ownership);
                self.init += r.is_ok() as usize;
                self.iter.reset_if(r.is_err());
                Ok(r)
            }
            None => Err(args),
        }
    }

    /// initialize the next element with the given arguments without checking if there is a next element
    ///
    /// # Safety
    ///
    /// * `remaining_len` must be non-zero
    pub unsafe fn try_init_unchecked<Args>(&mut self, args: Args) -> Result<(), T::Error>
    where
        T: Ctor<Args>,
    {
        debug_assert!(!self.iter.is_empty());
        // SAFETY: there is at least one element in the iterator
        let u = unsafe { self.iter.next_unchecked() };
        let r = u.try_init(args).map(Init::take_ownership);
        self.init += r.is_ok() as usize;
        self.iter.reset_if(r.is_err());
        r
    }

    /// Check if all elements of the slice are initialized
    pub const fn is_initialized(&self) -> bool {
        self.init == self.ptr.len()
    }

    /// finish the slice writer and extract the initialized slice
    pub fn finish(self) -> Init<'brand, [T]> {
        assert!(self.is_initialized());
        let this = ManuallyDrop::new(self);
        // SAFETY: we checked that the slice is initialized
        unsafe { core::ptr::read(&this.ptr).assume_init() }
    }

    /// finish the slice writer and extract the initialized slice
    /// without checking if the slice is actually finished
    ///
    /// # Safety
    ///
    /// `is_initialized` must return true
    pub unsafe fn finish_unchecked(self) -> Init<'brand, [T]> {
        debug_assert!(self.is_initialized());
        let this = ManuallyDrop::new(self);
        // SAFETY: the caller ensures that the slice is initialized
        unsafe { core::ptr::read(&this.ptr).assume_init() }
    }
}
