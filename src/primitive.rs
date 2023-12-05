use crate::Ctor;

use core::pin::Pin;

macro_rules! pick {
    ($a:literal $b:literal) => {
        $a
    };
    ($a:literal) => {
        $a
    };
}

macro_rules! primitive {
    ($($t:ident $(= $val:literal)?)*) => {$(
        impl Ctor for $t {
            type Error = core::convert::Infallible;

            #[inline]
            fn try_init(
                ptr: crate::ptr::Uninit<Self>,
                _: (),
            ) -> Result<crate::ptr::Init<Self>, Self::Error> {
                Ok(ptr.write(pick!($($val)? 0)))
            }
        }

        impl Ctor<$t> for $t {
            type Error = core::convert::Infallible;

            #[inline]
            fn try_init(
                ptr: crate::ptr::Uninit<Self>,
                src: $t,
            ) -> Result<crate::ptr::Init<Self>, Self::Error> {
                Ok(ptr.write(src))
            }
        }

        impl Ctor<&$t> for $t {
            type Error = core::convert::Infallible;

            #[inline]
            fn try_init<'a>(
                ptr: crate::ptr::Uninit<'a, Self>,
                src: &$t,
            ) -> Result<crate::ptr::Init<'a, Self>, Self::Error> {
                Ok(ptr.write(*src))
            }
        }

        impl Ctor<&mut $t> for $t {
            type Error = core::convert::Infallible;

            #[inline]
            fn try_init<'a>(
                ptr: crate::ptr::Uninit<'a, Self>,
                src: &mut $t,
            ) -> Result<crate::ptr::Init<'a, Self>, Self::Error> {
                Ok(ptr.write(*src))
            }
        }

        impl Ctor<Pin<&$t>> for $t {
            type Error = core::convert::Infallible;

            #[inline]
            fn try_init<'a>(
                ptr: crate::ptr::Uninit<'a, Self>,
                src: Pin<&$t>,
            ) -> Result<crate::ptr::Init<'a, Self>, Self::Error> {
                Ok(ptr.write(*src))
            }
        }

        impl Ctor<Pin<&mut $t>> for $t {
            type Error = core::convert::Infallible;

            #[inline]
            fn try_init<'a>(
                ptr: crate::ptr::Uninit<'a, Self>,
                src: Pin<&mut $t>,
            ) -> Result<crate::ptr::Init<'a, Self>, Self::Error> {
                Ok(ptr.write(*src))
            }
        }
    )*};
}

primitive! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
    char = '\0' bool = false
    f32 = 0.0 f64 = 0.0
}

impl<T> Ctor for Option<T> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init(
        ptr: crate::ptr::Uninit<Self>,
        _: (),
    ) -> Result<crate::ptr::Init<Self>, Self::Error> {
        Ok(ptr.write(None))
    }
}

impl<T> Ctor<Option<T>> for Option<T> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init(
        ptr: crate::ptr::Uninit<Self>,
        val: Option<T>,
    ) -> Result<crate::ptr::Init<Self>, Self::Error> {
        Ok(ptr.write(val))
    }
}
