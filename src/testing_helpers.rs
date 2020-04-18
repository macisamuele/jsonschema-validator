#[macro_export]
macro_rules! hash_set {
    ($($x:expr),*) => ({
        vec![$($x),*].into_iter().collect::<::std::collections::HashSet<_>>()
    });
}
