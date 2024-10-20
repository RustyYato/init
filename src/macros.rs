pub use core;

#[macro_export]
macro_rules! init_struct {
    ($struct_name:path { $($field:ident:$field_value:expr),+ $(,)? }) => {
        $crate::try_from_fn(|mut uninit| {
            let $struct_name { $($field: _,)* };

            let ptr: *mut $struct_name = uninit.as_mut_ptr();
            $(
                // SAFETY: re-borrowing a field as an uninit is sound
                let field = unsafe { $crate::Uninit::from_raw(&raw mut (*ptr).$field) };
                let $field = field.try_init($field_value)?;
            )*
            $crate::__private_macros::core::mem::forget(($($field,)*));
            // SAFETY: all fields were initialized
            Ok(unsafe { uninit.assume_init() })
        })
    };
}
