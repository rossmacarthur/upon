// Heavily based on `serde_json::json!`
/// Convenient macro for constructing a [`Value`][crate::Value].
///
/// The macro always returns a [`Value::Map`][crate::Value::Map] variant at the
/// top level but the map values can be any variant. The keys must be
/// identifiers not string literals, similar to Rust's struct initialization
/// syntax.
///
/// # Examples
///
/// ```
/// let v = upon::value!{
///     users: [
///         {
///             name: "John Smith",
///             age: 36,
///             is_enabled: true,
///             address: None,
///         },
///         {
///             name: "Jane Doe",
///             age: 34,
///             is_enabled: false,
///         },
///     ]
/// };
/// ```
///
/// Variables and expressions can be interpolated as map values,
/// [`to_value`][crate::to_value] will be used to convert the expression to a
/// [`Value`][crate::Value].
///
/// ```
/// let names = vec!["John", "James"];
/// let addr = "42 Wallaby Way, Sydney";
///
/// let v = upon::value!{
///     names: names,
///     surname: "Smith",
///     address: addr,
/// };
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[macro_export]
macro_rules! value {
    ( $($tt:tt)+ ) => {
        $crate::Value::Map({
            let mut map = ::std::collections::BTreeMap::new();
            $crate::_value!(@map map () ($($tt)+) ($($tt)+));
            map
        })
    };

    () => {
        $crate::Value::Map(::std::collections::BTreeMap::new())
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! _value {
    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of a list [...]. Produces a List of
    // the elements.
    //
    // Must be invoked as: value!(@list [] $($tt)*)
    //////////////////////////////////////////////////////////////////////////

    // Done with trailing comma.
    (@list [$($elems:expr,)*]) => {
        $crate::_vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@list [$($elems:expr),*]) => {
        $crate::_vec![$($elems),*]
    };

    // Next element is `None`.
    (@list [$($elems:expr,)*] None $($rest:tt)*) => {
        $crate::_value!(@list [$($elems,)* $crate::_value!(None)] $($rest)*)
    };

    // Next element is an array.
    (@list [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        $crate::_value!(@list [$($elems,)* $crate::_value!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@list [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        $crate::_value!(@list [$($elems,)* $crate::_value!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@list [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        $crate::_value!(@list [$($elems,)* $crate::_value!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@list [$($elems:expr,)*] $last:expr) => {
        $crate::_value!(@list [$($elems,)* $crate::_value!($last)])
    };

    // Comma after the most recent element.
    (@list [$($elems:expr),*] , $($rest:tt)*) => {
        $crate::_value!(@list [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@list [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        $crate::_value_unexpected!($unexpected)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of a map {...}. Each entry is
    // inserted into the given map variable.
    //
    // Must be invoked as: value!(@map $map () ($($tt)*) ($($tt)*))
    //
    // We require two copies of the input tokens so that we can match on one
    // copy and trigger errors on the other copy.
    //////////////////////////////////////////////////////////////////////////

    // Done.
    (@map $map:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@map $map:ident [$key:ident] ($value:expr) , $($rest:tt)*) => {
        let _ = $map.insert(stringify!($key).into(), $value);
        $crate::_value!(@map $map () ($($rest)*) ($($rest)*));
    };

    // Current entry followed by unexpected token.
    (@map $map:ident [$key:ident] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        $crate::_value_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@map $map:ident [$key:ident] ($value:expr)) => {
        let _ = $map.insert(stringify!($key).into(), $value);
    };

    // Next value is `None`.
    (@map $map:ident ($key:ident) (: None $($rest:tt)*) $copy:tt) => {
        $crate::_value!(@map $map [$key] ($crate::_value!(None)) $($rest)*);
    };

    // Next value is an array.
    (@map $map:ident ($key:ident) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        $crate::_value!(@map $map [$key] ($crate::_value!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@map $map:ident ($key:ident) (: {$($mapping:tt)*} $($rest:tt)*) $copy:tt) => {
        $crate::_value!(@map $map [$key] ($crate::_value!({$($mapping)*})) $($rest)*);
    };

    // Next value is an ident followed by comma.
    (@map $map:ident ($key:ident) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        $crate::_value!(@map $map [$key] ($crate::_value!($value)) , $($rest)*);
    };

    // Last value is an ident with no trailing comma.
    (@map $map:ident ($key:ident) (: $value:expr) $copy:tt) => {
        $crate::_value!(@map $map [$key] ($crate::_value!($value)));
    };

    // Missing value for last entry.
    // Trigger a reasonable error message.
    (@map $map:ident ($key:ident) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::_value!();
    };

    // Missing colon and value for last entry.
    // Trigger a reasonable error message.
    (@map $map:ident ($key:ident) () $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::_value!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@map $map:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        $crate::_value_unexpected!($colon);
    };

    // Take an ident for the current key.
    (@map $map:ident () ($key:ident $($rest:tt)*) $copy:tt) => {
        $crate::_value!(@map $map ($key) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: value!($($value)+)
    //////////////////////////////////////////////////////////////////////////
    (None) => {
        $crate::Value::None
    };

    (false) => {
        $crate::Value::Bool(false)
    };

    (true) => {
        $crate::Value::Bool(false)
    };

    ([]) => {
        $crate::Value::List($crate::_vec![])
    };

    ([ $($tt:tt)+ ]) => {
        $crate::Value::List($crate::_value!(@list [] $($tt)+))
    };

    ({}) => {
        $crate::Value::Map(::std::collections::BTreeMap::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::Value::Map({
            let mut map = ::std::collections::BTreeMap::new();
            $crate::_value!(@map map () ($($tt)+) ($($tt)+));
            map
        })
    };

    // Default to `Serialize` implementation.
    ($other:expr) => {
        $crate::to_value($other).unwrap()
    };
}

// The `value!` macro above cannot invoke vec directly because it uses
// local_inner_macros. A vec invocation there would resolve to $crate::vec.
// Instead invoke vec here outside of local_inner_macros.
#[macro_export]
#[doc(hidden)]
macro_rules! _vec {
    ($($content:tt)*) => {
        ::std::vec![$($content)*]
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! _value_unexpected {
    () => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! _value_expect_key_comma {
    ($key:ident , $($tt:tt)*) => {};
}
