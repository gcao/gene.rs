use uuid::Uuid;

pub fn new_uuidv4() -> String {
    return format!("{}", Uuid::new_v4());
}

#[macro_export]
// Create a **BTreeMap** from a list of key-value pairs
macro_rules! map {
    // trailing comma case
    ($($key:expr => $value:expr,)+) => (map!($($key => $value),+));

    ( $($key:expr => $value:expr),* ) => {
        {
            let mut _map = ::std::collections::BTreeMap::new();
            $(
                let _ = _map.insert($key.into(), $value);
            )*
            _map
        }
    };
}
