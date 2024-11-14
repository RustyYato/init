pub use core;
use core::marker::PhantomData;

#[macro_export]
macro_rules! init_struct {
    ($struct_name:path { $($field:ident:$field_value:expr),+ $(,)? }) => {
        $crate::try_from_fn(|mut uninit| {
            let $struct_name { $($field: _,)* };

            let ptr: *mut $struct_name = uninit.as_mut_ptr();
            $(
                // SAFETY: re-borrowing a field as an uninit is sound
                let field = unsafe { $crate::Uninit::from_raw(&raw mut (*ptr).$field) };
                let mut $field = match field.try_init($field_value) {
                    Ok(field) => field,
                    Err(x) => {
                        use $crate::__private_macros::GetConverter;
                        let w = $crate::__private_macros::Wrapper(&x);
                        let converter = (&&&&w).__private_init_get_converter();
                        return Err(converter.convert(x))
                    },
                };

                let _ = $field.as_mut_ptr(); // to silence unused mut warnings
            )*
            $crate::__private_macros::core::mem::forget(($($field,)*));
            // SAFETY: all fields were initialized
            Ok(unsafe { uninit.assume_init() })
        })
    };
}

pub trait GetConverter<T, U> {
    type Converter;

    fn __private_init_get_converter(&self) -> Self::Converter;
}

pub struct Wrapper<'a, T: ?Sized>(pub &'a T);

pub struct TagFrom<T, U>(PhantomData<(T, U)>);
pub struct TagInf;

pub trait HasFrom<U> {}

impl<T, U: From<T>> HasFrom<U> for T {}

impl<T: IsEmptyType> GetConverter<T, ()> for &Wrapper<'_, T> {
    type Converter = TagInf;

    fn __private_init_get_converter(&self) -> Self::Converter {
        TagInf
    }
}

impl<T: HasFrom<U>, U> GetConverter<T, U> for Wrapper<'_, T> {
    type Converter = TagFrom<T, U>;

    fn __private_init_get_converter(&self) -> Self::Converter {
        TagFrom(PhantomData)
    }
}

pub trait IsEmptyType {
    fn empty(self) -> !;
}

impl IsEmptyType for core::convert::Infallible {
    fn empty(self) -> ! {
        match self {}
    }
}

impl TagInf {
    pub fn convert<T: IsEmptyType, U>(self, t: T) -> U {
        t.empty()
    }
}

impl<T, U: From<T>> TagFrom<T, U> {
    pub fn convert(self, t: T) -> U {
        U::from(t)
    }
}
