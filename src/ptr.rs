//! The base pointer types that enforce safety of the API

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::NonNull,
};

struct Invariant<'a, T: ?Sized>(PhantomData<fn() -> *mut &'a mut T>);

/// A pointer to an uninitialized T
///
/// This pointer is guaratneed to be...
/// * non-null
/// * aligned
/// * unique for at least 'a
/// * allocated with a layout which fits T's layout
/// * valid for reads and writes
pub struct Uninit<'a, T: ?Sized> {
    ptr: NonNull<T>,
    inv: PhantomData<Invariant<'a, T>>,
}

/// A pointer to an uninitialized T
///
/// This pointer is guaratneed to be...
/// * non-null
/// * aligned
/// * unique for at least 'a
/// * allocated with a layout which fits T's layout
/// * initialized for T
/// * valid for reads and writes
/// * the `T` must not be moved in memory (i.e. the T is pinned)
pub struct Init<'a, T: ?Sized> {
    raw: Uninit<'a, T>,
}

impl<'a, T: ?Sized> Uninit<'a, T> {
    /// Create a new `Uninit` from a raw pointer
    ///
    /// # Safety
    ///
    /// * non-null
    /// * aligned
    /// * not aliased for at least 'a
    /// * allocated with a layout which fits T's layout
    ///
    /// If the pointer `ptr` is derived from another `Uninit<'b, _>`,
    /// then `'a`, then you must ensure that 'a == 'b.
    pub const unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            // SAFETY: the caller ensures that the pointer is non-null
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            inv: PhantomData,
        }
    }

    /// Convert this `Uninit` into a raw pointer
    pub const fn into_raw(self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Convert this `Uninit` into a raw pointer
    pub const fn into_non_null(self) -> NonNull<T> {
        self.ptr
    }

    /// Convert this `Uninit` into a raw pointer
    pub const fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Convert this `Uninit` into a raw pointer
    pub const fn as_non_null(&self) -> NonNull<T> {
        self.ptr
    }

    /// Convert this `Uninit` to a `Init` without checking
    /// if it was initialized yet
    ///
    /// # Safety
    ///
    /// You must ensure that this `Uninit` points to an initialized T
    pub const unsafe fn assume_init(self) -> Init<'a, T> {
        Init { raw: self }
    }

    #[doc(hidden)]
    pub fn ensure_lifetimes_eq<U: ?Sized>(&self, _: &Uninit<'a, U>) {}

    /// Initialize this pointer with the given initializer
    pub fn init<Args: crate::Initializer<T, Error = core::convert::Infallible>>(
        self,
        args: Args,
    ) -> Init<'a, T> {
        match args.try_init_into(self) {
            Ok(init) => init,
            Err(inf) => match inf {},
        }
    }

    /// Try to initialize this pointer with the given initializer
    pub fn try_init<Args: crate::Initializer<T>>(
        self,
        args: Args,
    ) -> Result<Init<'a, T>, Args::Error> {
        args.try_init_into(self)
    }
}

impl<'a, T> Uninit<'a, T> {
    /// Cast the pointer to an array to a pointer to a slice
    pub fn write(self, value: T) -> Init<'a, T> {
        // SAFETY: `Uninit` guarantees that the pointer is
        // * valid for writes
        // * properly aligned
        // * properly initialized value of type `T`
        unsafe { self.ptr.as_ptr().write(value) };
        // SAFETY: ^^ we just initialized the pointer
        unsafe { self.assume_init() }
    }
}

impl<'a, T, const N: usize> Uninit<'a, [T; N]> {
    /// Cast the pointer to an array to a pointer to a slice
    pub const fn as_slice(self) -> Uninit<'a, [T]> {
        // SAFETY: Any pointer that is allocated for [T; N]
        // must also be allocated for [T] (with length N)
        unsafe { Uninit::from_raw(self.into_raw() as *mut [T]) }
    }
}

impl<'a, T> Uninit<'a, [T]> {
    /// Check if the slice is
    pub const fn len(&self) -> usize {
        self.ptr.len()
    }

    /// Check if the slice is empty
    pub const fn is_empty(&self) -> bool {
        self.ptr.len() == 0
    }

    /// Check if the slice is empty
    pub const fn try_as_array<const N: usize>(self) -> Result<Uninit<'a, [T; N]>, Self> {
        if self.len() == N {
            // SAFETY: Any pointer that is allocated for [T] (with length N)
            // must also be allocated for [T; N]
            Ok(unsafe { Uninit::from_raw(self.into_raw().cast()) })
        } else {
            Err(self)
        }
    }
}

impl<'a, T: ?Sized> Init<'a, T> {
    /// Get the underlying pointer
    pub const fn as_ptr(&self) -> *const T {
        self.raw.ptr.as_ptr()
    }

    /// Get the underlying pointer
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.raw.ptr.as_ptr()
    }

    /// Convert this `Init` into a raw pointer without dropping the value
    pub const fn into_raw(self) -> *mut T {
        let ptr = self.raw.ptr;
        self.take_ownership();
        ptr.as_ptr()
    }

    #[doc(hidden)]
    pub fn ensure_lifetimes_eq<U: ?Sized>(&self, _: &Init<'a, U>) {}

    /// Leaks this `Init` because someone else has taken ownership of the underlying pointer
    pub const fn take_ownership(self) {
        core::mem::forget(self);
    }

    /// Get a mutable reference to the underlying type
    ///
    /// # Safety
    ///
    /// You must not move the value out of the reference
    pub unsafe fn get_mut_unchecked(&mut self) -> &mut T {
        // SAFETY: We have unique access to the ptr, and the caller
        // guarantees that they will not move the value out of the reference
        unsafe { self.raw.ptr.as_mut() }
    }

    /// Get a mutable reference to the underlying type
    ///
    /// # Safety
    ///
    /// The underlying allocation backing this `Init` must not be deallocated
    /// before this value is dropped.
    pub unsafe fn get_pin_mut_unchecked(&mut self) -> Pin<&mut T> {
        // SAFETY: We have unique access to the ptr, and the caller
        // guarantees that they will not move the value out of the reference
        // NonNull<T> has the same layout as `Pin<&mut T>`
        // and this gets around the auto-self-reference invalidation
        // caused by creating a &mut T
        unsafe { core::mem::transmute(self.raw.ptr.as_ptr()) }
    }
}

impl<'a, T> Init<'a, T> {
    /// Convert this `Init` into a raw pointer without dropping the value
    pub const fn into_inner(self) -> T
    where
        T: Unpin,
    {
        // SAFETY: `Init` guarantees that the pointer is
        // * valid for reads
        // * properly aligned
        // * properly initialized value of type `T`
        unsafe { self.into_raw().read() }
    }
}

impl<'a, T> Init<'a, [T]> {
    /// Check if the slice is
    pub const fn len(&self) -> usize {
        self.raw.len()
    }

    /// Check if the slice is empty
    pub const fn is_empty(&self) -> bool {
        self.raw.len() == 0
    }

    /// Check if the slice is empty
    pub const fn try_as_array<const N: usize>(self) -> Result<Init<'a, [T; N]>, Self> {
        if self.len() == N {
            // SAFETY: Any pointer that is allocated for [T] (with length N)
            // must also be allocated for [T; N]
            Ok(unsafe { Uninit::from_raw(self.into_raw().cast()).assume_init() })
        } else {
            Err(self)
        }
    }
}

impl<T: ?Sized> Drop for Init<'_, T> {
    fn drop(&mut self) {
        // SAFETY: `Init` guarantees that the pointer is initialized,  unique,
        // aligned, non-null, valid for reads and writes
        unsafe { self.raw.ptr.as_ptr().drop_in_place() }
    }
}

impl<T: ?Sized> Deref for Init<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: `Init` guarantees that the pointer is initialized,  unique,
        // aligned, non-null, valid for reads
        unsafe { self.raw.ptr.as_ref() }
    }
}

impl<T: ?Sized + Unpin> DerefMut for Init<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `Init` guarantees that the pointer is initialized,  unique,
        // aligned, non-null, valid for reads and writes
        unsafe { self.raw.ptr.as_mut() }
    }
}
