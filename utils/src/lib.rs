#[macro_export]
macro_rules! linked_hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(linked_hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { linked_hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = linked_hashmap!(@count $($key),*);
            let mut _map = ::linked_hash_map::LinkedHashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
                _map
        }
    };
}
