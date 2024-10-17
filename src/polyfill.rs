#![allow(clippy::transmutes_expressible_as_ptr_casts, clippy::useless_transmute)]

// ripped from std
pub const fn without_provenance_mut<T>(addr: usize) -> *mut T {
    // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
    // We use transmute rather than a cast so tools like Miri can tell that this
    // is *not* the same as with_exposed_provenance.
    // SAFETY: every valid integer is also a valid pointer (as long as you don't dereference that
    // pointer).
    unsafe { core::mem::transmute(addr) }
}

// ripped from std
pub fn addr<T>(ptr: *mut T) -> usize {
    // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
    // SAFETY: Pointer-to-integer transmutes are valid (if you are okay with losing the
    // provenance).
    unsafe { core::mem::transmute(ptr.cast::<()>()) }
}
