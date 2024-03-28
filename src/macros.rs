#[doc(hidden)]
#[macro_export]
macro_rules! init_struct_ {
    ($uninit:ident => $type:path [$fields_holder:ident] {
        $($fields:ident ($value:expr))*
    }) => {(|| -> $crate::core::result::Result<_, _> {
        let $type { $($fields: _,)* };
        let _: $crate::ptr::Uninit<_> = $uninit;
        let ptr = $uninit.as_ptr();

        #[allow(non_camel_case_types)]
        struct __FieldsManager<$($fields),*> {
            $($fields: $fields),*
        }

        let $fields_holder = __FieldsManager {
            // SAFETY: Each field is disjoint, so all fields are not aliased
            $($fields:  unsafe { $crate::ptr::Uninit::from_raw($crate::core::ptr::addr_of_mut!((*ptr).$fields)) }),*
        };

        $(
            let $fields = $fields_holder.$fields.try_init($value)?;
        )*

        $(
            $fields.take_ownership();
        )*

        // SAFETY: All fields are initialized, so the struct is initialized
        Ok(unsafe { $uninit.assume_init() })
    })()};
}

/// Initialize a struct field by field
#[macro_export]
macro_rules! init_struct {
    ($uninit:expr => $type:path {
        $($($fields:ident: $value:expr),+ $(,)?)?
    }) => {
        match $uninit {
            uninit => $crate::init_struct_! {
                uninit => $type [fields] {
                    $($($fields ($value))+)?
                }
            },
        }
    };
    ($uninit:expr => $type:path => $fields_holder:ident {
        $($($fields:ident: $value:expr),+ $(,)?)?
    }) => {
        match $uninit {
            uninit => $crate::init_struct_! {
                uninit => $type [$fields_holder] {
                    $($($fields ($value))+)?
                }
            },
        }
    };
}

struct Foo {
    a: i32,
    b: i32,
}

fn test(ptr: crate::ptr::Uninit<Foo>) -> Result<crate::ptr::Init<Foo>, core::convert::Infallible> {
    init_struct! {
        ptr => Foo {
            a: 0,
            b: 3
        }
    }
}
