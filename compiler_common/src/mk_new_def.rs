//! Constructor-creating macro.

/// Constructor-creating macro.
///
/// # For an `enum`eration
///
/// ```rust
/// # use compiler_common::mk_new;
/// enum TestEnum {
///     Tuple(String, usize),
///     Structure { s: String, n: usize },
///     Nullary,
/// }
/// mk_new! { impl TestEnum =>
///     Nullary: nullary()
///     Structure: structure {
///         s : impl Into<String> = s.into(),
///         n : usize, // `= <expr>` is optional on `struct`-like
///     }
///     Tuple: tuple (
///         s : impl Into<String> = s.into(),
///         n : usize = n, // `= <expr>` is **not** optional on `tuple`-like
///     )
/// }
///
/// let _ = TestEnum::nullary();
/// let _ = TestEnum::tuple("tuple", 0);
/// let _ = TestEnum::structure("structure", 0);
/// ```
///
/// # For a `struct`ure
///
/// ```rust
/// # use compiler_common::mk_new;
/// struct TestStruct {
///     pub _s: String,
///     pub _n: usize,
/// }
/// mk_new! { impl TestStruct =>
///     new {
///         _s : impl Into<String> = _s.into(),
///         _n : usize, // `= <expr>` is optional on `struct`-like
///     }
/// }
///
/// let _ = TestStruct::new("blah", 0);
/// ```
#[macro_export]
macro_rules! mk_new {
    { // top-level, specifies `Self`, can be bypassed
        // type params
        impl $({$($tparams:tt)*})?
        // `Self`
        $slf:ty
        // where clause
        $(where ($($where_clause:tt)*))?
        =>
        $($stuff:tt)*
    } => {
        impl $(<$($tparams)*>)? $slf $(where $($where_clause)*)? {
            $crate::mk_new! {
                $($stuff)*
            }
        }
    };
    { // pure `struct` constructor
        $(#[$meta:meta])*
        $fn_id:ident {
            $( $field:ident $(: $param_ty:ty)? $(= $value:expr )? ),* $(,)?
        }
        $($stuff:tt)*
    } => {
        $(#[$meta])*
        pub fn $fn_id( $($($field : $param_ty ,)?)* ) -> Self {
            Self {
                $(
                    $field $(: $value)? ,
                )*
            }
        }
        $crate::mk_new! { $($stuff)* }
    };

    { // `enum` variant: `struct`-like
        $(#[$meta:meta])*
        $id:ident : $fn_id:ident {
            $( $field:ident $(: $param_ty:ty)? $(= $value:expr )? ),* $(,)?
        }
        $($stuff:tt)*
    } => {
        $(#[$meta])*
        pub fn $fn_id( $($($field : $param_ty ,)?)* ) -> Self {
            Self::$id {
                $(
                    $field $(: $value)? ,
                )*
            }
        }
        $crate::mk_new! { $($stuff)* }
    };
    { // `enum` variant: nullary
        $(#[$meta:meta])*
        $id:ident : $fn_id:ident()
        $($stuff:tt)*
    } => {
        $(#[$meta])*
        pub fn $fn_id() -> Self {
            Self::$id
        }
        $crate::mk_new! { $($stuff)* }
    };
    { // `enum` variant: tuple-like
        $(#[$meta:meta])*
        $id:ident : $fn_id:ident (
            $( $field:ident $(: $param_ty:ty)? = $value:expr ),* $(,)?
        )
        $($stuff:tt)*
    } => {
        $(#[$meta])*
        pub fn $fn_id( $($($field : $param_ty ,)?)* ) -> Self {
            Self::$id ( $( $value, )* )
        }
        $crate::mk_new! { $($stuff)* }
    };
    {} => {};
    { $($stuff:tt)+ } => {
        compile_error!("unexpected syntax")
    }
}
