#[macro_export]
macro_rules! hash_set {
    ($($x:expr),*) => {{
        #[allow(unused_mut)]
        let mut set = std::collections::HashSet::new();
        $(let _ = set.insert($x);)*
        set
    }};
}
