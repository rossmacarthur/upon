// Heavily based on `serde_json::json!`
#[macro_export]
macro_rules! data {
    ($($d:tt)+) => {
        $crate::_data!($($d)+)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! _data {
    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of a list [...]. Produces a List of
    // the elements.
    //
    // Must be invoked as: data!(@list [] $($tt)*)
    //////////////////////////////////////////////////////////////////////////

    // Done with trailing comma.
    (@list [$($elems:expr,)*]) => {
        $crate::_data_list![$($elems,)*]
    };

    // Done without trailing comma.
    (@list [$($elems:expr),*]) => {
        $crate::_data_list![$($elems),*]
    };

    // Next element is `None`.
    (@list [$($elems:expr,)*] None $($rest:tt)*) => {
        $crate::_data!(@list [$($elems,)* $crate::_data!(None)] $($rest)*)
    };

    // Next element is an array.
    (@list [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        $crate::_data!(@list [$($elems,)* $crate::_data!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@list [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        $crate::_data!(@list [$($elems,)* $crate::_data!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@list [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        $crate::_data!(@list [$($elems,)* $crate::_data!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@list [$($elems:expr,)*] $last:expr) => {
        $crate::_data!(@list [$($elems,)* $crate::_data!($last)])
    };

    // Comma after the most recent element.
    (@list [$($elems:expr),*] , $($rest:tt)*) => {
        $crate::_data!(@list [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@list [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        $crate::_data_unexpected!($unexpected)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of a map {...}. Each entry is
    // inserted into the given map variable.
    //
    // Must be invoked as: data!(@map $map () ($($tt)*) ($($tt)*))
    //
    // We require two copies of the input tokens so that we can match on one
    // copy and trigger errors on the other copy.
    //////////////////////////////////////////////////////////////////////////

    // Done.
    (@map $map:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@map $map:ident [$key:ident] ($value:expr) , $($rest:tt)*) => {
        let _ = $map.insert(stringify!($key).into(), $value);
        $crate::_data!(@map $map () ($($rest)*) ($($rest)*));
    };

    // Current entry followed by unexpected token.
    (@map $map:ident [$key:ident] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        $crate::_data_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@map $map:ident [$key:ident] ($value:expr)) => {
        let _ = $map.insert(stringify!($key).into(), $value);
    };

    // Next value is `None`.
    (@map $map:ident ($key:ident) (: None $($rest:tt)*) $copy:tt) => {
        $crate::_data!(@map $map [$key] ($crate::_data!(None)) $($rest)*);
    };

    // Next value is an array.
    (@map $map:ident ($key:ident) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        $crate::_data!(@map $map [$key] ($crate::_data!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@map $map:ident ($key:ident) (: {$($mapping:tt)*} $($rest:tt)*) $copy:tt) => {
        $crate::_data!(@map $map [$key] ($crate::_data!({$($mapping)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@map $map:ident ($key:ident) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        $crate::_data!(@map $map [$key] ($crate::_data!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@map $map:ident ($key:ident) (: $value:expr) $copy:tt) => {
        $crate::_data!(@map $map [$key] ($crate::_data!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@map $map:ident ($key:ident) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::_data!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@map $map:ident ($key:ident) () $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::_data!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@map $map:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        $crate::_data_unexpected!($colon);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@map $map:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        $crate::_data_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@map $map:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        $crate::_data!(@map $map ($key) (: $($rest)*) (: $($rest)*));
    };

    // Refuse to absorb colon token into key expression.
    (@map $map:ident ($($key:tt)*) (: $($unexpected:tt)+) $copy:tt) => {
        $crate::_data_expect_expr_comma!($($unexpected)+);
    };

    // Munch a token into the current key.
    (@map $map:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        $crate::_data!(@map $map ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: data!($($data)+)
    //////////////////////////////////////////////////////////////////////////
    (None) => {
        $crate::Value::None
    };

    ([]) => {
        $crate::Value::List($crate::_data_list![])
    };

    ([ $($tt:tt)+ ]) => {
        $crate::Value::List($crate::_data!(@list [] $($tt)+))
    };

    ({}) => {
        $crate::Value::Map($crate::Map::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::Value::Map({
            let mut map = $crate::Map::new();
            $crate::_data!(@map map () ($($tt)+) ($($tt)+));
            map
        })
    };

    // Default to `From` implementation.
    ($other:expr) => {
        $crate::Value::from($other)
    };
}

// The data macro above cannot invoke vec directly because it uses
// local_inner_macros. A vec invocation there would resolve to $crate::vec.
// Instead invoke vec here outside of local_inner_macros.
#[macro_export]
#[doc(hidden)]
macro_rules! _data_list {
    ($($content:tt)*) => {
        ::std::vec![$($content)*]
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! _data_unexpected {
    () => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! _data_expect_expr_comma {
    ($e:expr , $($tt:tt)*) => {};
}

#[cfg(test)]
mod tests {
    use crate::{List, Map, Value};

    #[test]
    fn data_none() {
        let v = data!(None);
        assert_eq!(v, Value::None);
    }

    #[test]
    fn data_string() {
        let v = data!("testing...");
        assert_eq!(v, Value::from("testing..."));
    }

    #[test]
    fn data_list() {
        let v = data!(["testing...", None, {}, []]);
        assert_eq!(
            v,
            Value::from([
                Value::from("testing..."),
                Value::None,
                Value::Map(Map::new()),
                Value::List(List::new()),
            ])
        )
    }

    #[test]
    fn data_map() {
        let v = data!({ x: "hello" });
        let exp = Value::from([("x".into(), "hello".into())]);
        assert_eq!(v, exp);

        let v = data!({ x: "hello", });
        let exp = Value::from([("x".into(), "hello".into())]);
        assert_eq!(v, exp);

        let v = data!({ x: "hello", y: String::from("world!") });
        let exp = Value::from([("x".into(), "hello".into()), ("y".into(), "world!".into())]);
        assert_eq!(v, exp);
    }

    #[test]
    fn data_map_nested() {
        let v = data!({
            w: "hello",
            x: {
                y: "hello",
                z: "world!",
            },
        });
        let exp = Value::from([
            ("w".into(), "hello".into()),
            (
                "x".into(),
                Value::from([("y".into(), "hello".into()), ("z".into(), "world!".into())]),
            ),
        ]);
        assert_eq!(v, exp);
    }
}
