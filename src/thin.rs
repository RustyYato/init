//! Thin pointers where any necessary metadata is stored inline with the data

use core::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// An erasable value is one where the pointer to it can be made thin
///
/// # Safety
///
/// In the implementations of `erase` and `unerase` you are not allowed
/// to create references
///
/// `unerase` should be the inverse of `erase`, and they should
/// preserve provenance: i.e. `unerase(erase(x)) == x` (including provenance)
pub unsafe trait Erasable {
    /// Convert this fat pointer into a thin pointer
    ///
    /// # Safety
    ///
    /// `ptr` must point to an initialized, and well aligned pointer to Self
    unsafe fn erase(ptr: NonNull<Self>) -> NonNull<Erased>;

    /// Convert this thin pointer into a fat pointer
    ///
    /// # Safety
    ///
    /// This pointer must have come from `Self::erase`
    unsafe fn unerase(ptr: NonNull<Erased>) -> NonNull<Self>;

    /// Convert this thin pointer into a fat pointer
    ///
    /// If Self: Copy, this is a no-op, and the erased pointer is still valid
    ///
    /// # Safety
    ///
    /// This pointer must have come from `Self::erase`
    ///
    /// The erased pointer must not be passed into unerase unless Self: Copy
    unsafe fn drop_in_place(ptr: NonNull<Erased>) {
        // SAFETY: the caller ensures that this is safe
        unsafe { Self::unerase(ptr).drop_in_place() };
    }
}

/// A macro to implement Erasable on your own sized types
#[macro_export]
macro_rules! ErasableSized {
    (for[$($bounds:tt)*] $ty:ty ) => {
        /// SAFETY: ptr casts preserve provenance, and are inverses when they type-check
        /// and no references are created
        unsafe impl<$($bounds)*> $crate::thin::Erasable for $ty
        where
            Self: Sized
        {
            unsafe fn erase(ptr: $crate::__private_macros::core::ptr::NonNull<Self>) -> $crate::__private_macros::core::ptr::NonNull<$crate::thin::Erased> {
                ptr.cast()
            }

            unsafe fn unerase(ptr: $crate::__private_macros::core::ptr::NonNull<$crate::thin::Erased>) -> $crate::__private_macros::core::ptr::NonNull<Self> {
                ptr.cast()
            }
        }
    };
    ($ty:ty) => {
        $crate::ErasableSized! {
            for[] $ty
        }
    }
}

/// An erasable pointer is a type which can be converted to and from an erased pointer
///
/// # Safety
///
/// In the implementations of `into_erased` and `from_erased` you are not allowed
/// to create references
///
/// `from_erased` should be the inverse of `into_erased`, and they should
/// preserve provenance: i.e. `from_erased(into_erased(x)) == x` (including provenance)
///
/// If Self: Deref, and if Self::Target then it should be valid
/// to convert the erased pointer into a fat pointer via `<Self::Target as Erasable>::unerase`
/// and it should be valid to convert the fat pointer into a reference until `from_erased` or `drop_in_place` is called
///
/// If Self: Copy, it should be valid to convert the fat pointer into a reference for as long
/// as `Self` is a well formed type. (i.e. all lifetimes in `Self` are well formed, and in scope)
///
/// If Self: DerefMut, then it should be safe to convert the fat pointer into a mutable reference
pub unsafe trait ErasablePtr: Sized {
    /// Convert this fat pointer into a thin pointer
    ///
    /// The resulting erased pointer owns the pointer
    fn into_erased(self) -> NonNull<Erased>;

    /// Convert this thin pointer into a fat pointer
    ///
    /// The ownership of the erased pointer is transferred to the
    /// returned pointer
    ///
    /// If Self: Copy, then the erased pointer is still valid
    ///
    /// # Safety
    ///
    /// This pointer must have come from `Self::into_erased`
    unsafe fn from_erased(ptr: NonNull<Erased>) -> Self;

    /// Convert this thin pointer into a fat pointer
    ///
    /// The ownership of the erased pointer is consumed.
    ///
    /// If Self: Copy, this is a no-op, and the erased pointer is still valid
    ///
    /// # Safety
    ///
    /// This pointer must have come from `Self::into_erased`
    unsafe fn drop_in_place(ptr: NonNull<Erased>) {
        // SAFETY: caller ensures the same condition
        unsafe { Self::from_erased(ptr) };
    }
}

/// A dummy type for Erased Pointers
pub struct Erased;

/// a thin pointer created from a `P`
///
/// This type is unconditionally  [`Copy`], but can only be created
/// from `P: Copy`
pub struct ThinCopy<P> {
    ptr: NonNull<Erased>,
    ty: PhantomData<P>,
}

/// a thin pointer created from a `P`
pub struct Thin<P: ErasablePtr> {
    thin: ThinCopy<P>,
}

impl<P> Copy for ThinCopy<P> {}
impl<P> Clone for ThinCopy<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: ErasablePtr> Drop for Thin<P> {
    fn drop(&mut self) {
        // SAFETY: `ThinCopy` has the invariant that it's pointer came from `ErasablePtr::erase`
        unsafe { P::drop_in_place(self.thin.ptr) };
    }
}

impl<P: ErasablePtr> ThinCopy<P> {
    fn new(ptr: P) -> Self {
        Self {
            ptr: ptr.into_erased(),
            ty: PhantomData,
        }
    }

    /// Get the original pointer back from the erased pointer
    pub fn to_inner(this: Self) -> P {
        // SAFETY: the implementor of ErasablePtr ensures that this is correct
        unsafe { P::from_erased(this.ptr) }
    }

    /// Get a read-only pointer into the erased value
    pub const fn as_erased_ptr(&self) -> NonNull<Erased> {
        self.ptr
    }

    /// Get a read-only pointer into the underlying value
    pub fn as_ptr(&self) -> NonNull<P::Target>
    where
        P: Deref,
        P::Target: Erasable,
    {
        // SAFETY: The implementor of `ErasablePtr` ensures that this is safe
        unsafe { Erasable::unerase(self.ptr) }
    }

    /// Check if these two pointers point to the same object
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr == other.ptr
    }
}

impl<P: ErasablePtr + Copy> ThinCopy<P> {
    /// Convert a erasable pointer into a thin pointer
    pub fn erase(ptr: P) -> Self {
        Self::new(ptr)
    }
}

impl<P: ErasablePtr> Thin<P> {
    /// Convert a erasable pointer into a thin pointer
    pub fn erase(ptr: P) -> Self {
        Self {
            thin: ThinCopy::new(ptr),
        }
    }

    /// Get a read-only pointer into the erased value
    pub const fn as_erased_ptr(&self) -> NonNull<Erased> {
        self.thin.ptr
    }

    /// Get the original pointer back from the erased pointer
    pub fn into_inner(this: Self) -> P {
        ThinCopy::to_inner(ManuallyDrop::new(this).thin)
    }

    /// Get a read-only pointer into the underlying value
    pub fn as_ptr(&self) -> NonNull<P::Target>
    where
        P: Deref,
        P::Target: Erasable,
    {
        self.thin.as_ptr()
    }

    /// Get a pointer into the underlying value
    pub fn as_mut_ptr(&mut self) -> NonNull<P::Target>
    where
        P: Deref,
        P::Target: Erasable,
    {
        self.thin.as_ptr()
    }

    fn inner(this: &Self) -> ManuallyDrop<P> {
        // SAFETY: the P is not dropped, so this should be fine
        ManuallyDrop::new(unsafe { P::from_erased(this.thin.ptr) })
    }

    /// Check if these two pointers point to the same object
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.thin.ptr == other.thin.ptr
    }

    /// run a closure with a borrow of the pointer
    pub fn with<T>(&self, f: impl FnOnce(&P) -> T) -> T {
        f(&Self::inner(self))
    }

    /// run a closure with a borrow of the pointer
    pub fn with_mut<T>(&mut self, f: impl FnOnce(&mut P) -> T) -> T {
        let mut ptr = scopeguard::guard(Self::inner(self), |x| {
            crate::polyfill::write(self, Self::erase(ManuallyDrop::into_inner(x)))
        });
        f(&mut ptr)
    }
}

impl<P: Clone + ErasablePtr> Clone for Thin<P> {
    fn clone(&self) -> Self {
        Thin::erase((*Self::inner(self)).clone())
    }
}

impl<P: Deref<Target: Erasable> + ErasablePtr> Deref for ThinCopy<P> {
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The implementor of `ErasablePtr` ensures that this is sound
        unsafe { self.as_ptr().as_ref() }
    }
}

impl<P: Deref<Target: Erasable> + ErasablePtr> Deref for Thin<P> {
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        &self.thin
    }
}

impl<P: DerefMut<Target: Erasable> + ErasablePtr> DerefMut for Thin<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The implementor of `ErasablePtr` ensures that this is sound
        unsafe { self.as_mut_ptr().as_mut() }
    }
}

// SAFETY:
//
// no references are created
//
// from_erased is the inverse of `into_erased`
//
// and P: ErasablePtr guarantees that the deref conditions are valid
// (this is enforced on creation of `ThinCopy`)
unsafe impl<P> ErasablePtr for ThinCopy<P> {
    fn into_erased(self) -> NonNull<Erased> {
        self.ptr
    }

    unsafe fn from_erased(ptr: NonNull<Erased>) -> Self {
        Self {
            ptr,
            ty: PhantomData,
        }
    }
}

// SAFETY:
//
// no references are created
//
// from_erased is the inverse of `into_erased`
//
// and P: ErasablePtr guarantees that the deref conditions are valid
unsafe impl<P: ErasablePtr> ErasablePtr for Thin<P> {
    fn into_erased(self) -> NonNull<Erased> {
        ThinCopy::into_erased(ManuallyDrop::new(self).thin)
    }

    unsafe fn from_erased(ptr: NonNull<Erased>) -> Self {
        Self {
            // SAFETY: the caller ensures that this is safe
            thin: unsafe { ThinCopy::from_erased(ptr) },
        }
    }
}

#[cfg(feature = "alloc")]
// SAFETY:
//
// no references are created
//
// from_erased is the inverse of `into_erased`
//
// Box::into_raw gives a valid pointer which can be converted into a (mutable) reference
unsafe impl<T: Erasable> ErasablePtr for alloc::boxed::Box<T> {
    fn into_erased(self) -> NonNull<Erased> {
        let ptr = Self::into_raw(self);
        // SAFETY: Box is guaranteed to be non-null
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        // SAFETY: Box is guaranteed to be initialized and well aligned
        unsafe { T::erase(ptr) }
    }

    unsafe fn from_erased(ptr: NonNull<Erased>) -> Self {
        // SAFETY: ptr came from `into_erased`, which created ptr via T::erase
        let ptr = unsafe { T::unerase(ptr) }.as_ptr();
        // SAFETY: T::unerase is the inverse of T::erase, so this is the pointer from Box::into_raw
        unsafe { Self::from_raw(ptr) }
    }
}

#[cfg(feature = "alloc")]
// SAFETY:
//
// no references are created
//
// from_erased is the inverse of `into_erased`
//
// Rc::into_raw gives a valid pointer which can be converted into a reference
unsafe impl<T: Erasable> ErasablePtr for alloc::rc::Rc<T> {
    fn into_erased(self) -> NonNull<Erased> {
        let ptr = Self::into_raw(self);
        // SAFETY: Box is guaranteed to be non-null
        let ptr = unsafe { NonNull::new_unchecked(ptr.cast_mut()) };
        // SAFETY: Box is guaranteed to be initialized and well aligned
        unsafe { T::erase(ptr) }
    }

    unsafe fn from_erased(ptr: NonNull<Erased>) -> Self {
        // SAFETY: ptr came from `into_erased`, which created ptr via T::erase
        let ptr = unsafe { T::unerase(ptr) }.as_ptr();
        // SAFETY: T::unerase is the inverse of T::erase, so this is the pointer from Box::into_raw
        unsafe { Self::from_raw(ptr) }
    }
}

#[cfg(feature = "alloc")]
// SAFETY:
//
// no references are created
//
// from_erased is the inverse of `into_erased`
//
// Rc::into_raw gives a valid pointer which can be converted into a reference
unsafe impl<T: Erasable> ErasablePtr for alloc::sync::Arc<T> {
    fn into_erased(self) -> NonNull<Erased> {
        let ptr = Self::into_raw(self);
        // SAFETY: Box is guaranteed to be non-null
        let ptr = unsafe { NonNull::new_unchecked(ptr.cast_mut()) };
        // SAFETY: Box is guaranteed to be initialized and well aligned
        unsafe { T::erase(ptr) }
    }

    unsafe fn from_erased(ptr: NonNull<Erased>) -> Self {
        // SAFETY: ptr came from `into_erased`, which created ptr via T::erase
        let ptr = unsafe { T::unerase(ptr) }.as_ptr();
        // SAFETY: T::unerase is the inverse of T::erase, so this is the pointer from Box::into_raw
        unsafe { Self::from_raw(ptr) }
    }
}

// SAFETY:
//
// no references are created
//
// from_erased is the inverse of `into_erased`
//
// NonNull::from gives a valid pointer which can be converted into a reference
unsafe impl<T: Erasable> ErasablePtr for &T {
    fn into_erased(self) -> NonNull<Erased> {
        let ptr = NonNull::from(self);
        // SAFETY: Box is guaranteed to be initialized and well aligned
        unsafe { T::erase(ptr) }
    }

    unsafe fn from_erased(ptr: NonNull<Erased>) -> Self {
        // SAFETY: ptr came from `into_erased`, which created ptr via T::erase
        let ptr = unsafe { T::unerase(ptr) };
        // SAFETY: T::unerase is the inverse of T::erase, so this is the pointer we cast to a NonNull
        unsafe { ptr.as_ref() }
    }
}

// SAFETY:
//
// no references are created
//
// from_erased is the inverse of `into_erased`
//
// NonNull::from gives a valid pointer which can be converted into a (mutable) reference
unsafe impl<T: Erasable> ErasablePtr for &mut T {
    fn into_erased(self) -> NonNull<Erased> {
        let ptr = NonNull::from(self);
        // SAFETY: Box is guaranteed to be initialized and well aligned
        unsafe { T::erase(ptr) }
    }

    unsafe fn from_erased(ptr: NonNull<Erased>) -> Self {
        // SAFETY: ptr came from `into_erased`, which created ptr via T::erase
        let mut ptr = unsafe { T::unerase(ptr) };
        // SAFETY: T::unerase is the inverse of T::erase, so this is the pointer we cast to a NonNull
        unsafe { ptr.as_mut() }
    }
}

ErasableSized!(u8);
ErasableSized!(u16);
ErasableSized!(u32);
ErasableSized!(u64);
ErasableSized!(u128);
ErasableSized!(usize);
ErasableSized!(i8);
ErasableSized!(i16);
ErasableSized!(i32);
ErasableSized!(i64);
ErasableSized!(i128);
ErasableSized!(isize);
ErasableSized!(f32);
ErasableSized!(f64);
ErasableSized!(char);
ErasableSized!(bool);
ErasableSized!(for[P] ThinCopy<P>);
ErasableSized!(for[P: ErasablePtr] Thin<P>);
ErasableSized!(for[T, const N: usize] [T; N]);
ErasableSized!(());
ErasableSized!(for[A] (A,));
ErasableSized!(for[A, B] (A, B));
ErasableSized!(for[A, B, C] (A, B, C));
ErasableSized!(for[A, B, C, D] (A, B, C, D));
ErasableSized!(for[A, B, C, D, E] (A, B, C, D, E));
ErasableSized!(for[A, B, C, D, E, F] (A, B, C, D, E, F));
ErasableSized!(for[A, B, C, D, E, F, G] (A, B, C, D, E, F, G));
ErasableSized!(for[A, B, C, D, E, F, G, H] (A, B, C, D, E, F, G, H));
