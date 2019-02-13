#[cfg(feature = "parallel")]
#[macro_export]
macro_rules! maybe_import_dependencies_for_parallel_run {
    () => {
        #[allow(unused_imports)]
        // avoid complex statements to avoid importing rayon prelude, just accept it and let the compiler decide
        use rayon::prelude::*;
    };
}

#[cfg(not(feature = "parallel"))]
#[macro_export]
macro_rules! maybe_import_dependencies_for_parallel_run {
    () => {};
}

#[cfg(feature = "parallel")]
#[macro_export]
macro_rules! iterate {
    ($object:expr) => {{
        use rayon::prelude::*;

        $object.par_iter()
    }};
}

#[cfg(not(feature = "parallel"))]
#[macro_export]
macro_rules! iterate {
    ($object:expr) => {
        $object.iter()
    };
}

#[cfg(feature = "parallel")]
#[macro_export]
macro_rules! iterate_mutable {
    ($object:expr) => {{
        use rayon::prelude::*;

        $object.par_iter_mut()
    }};
}

#[cfg(not(feature = "parallel"))]
#[macro_export]
macro_rules! iterate_mutable {
    ($object:expr) => {
        $object.iter_mut()
    };
}

#[cfg(feature = "parallel")]
#[macro_export]
macro_rules! into_iterator {
    ($object:expr) => {{
        use rayon::prelude::*;

        $object.into_par_iter()
    }};
}

#[cfg(not(feature = "parallel"))]
#[macro_export]
macro_rules! into_iterator {
    ($object:expr) => {
        $object.into_iter()
    };
}

#[cfg(test)]
mod tests {
    maybe_import_dependencies_for_parallel_run!();
    use test_case_derive::test_case;

    #[test_case(|&item| { item > 42 }, false)]
    #[test_case(|&item| { item < 2 }, true)]
    fn test_any_iter<P: Fn(&i32) -> bool + Sync + Send>(predicate: P, expected_result: bool) {
        assert_eq!(iterate![vec![1, 2, 3, 4]].any(predicate), expected_result);
    }

    #[test_case(|&item| { item > &10 }, vec![])]
    #[test_case(|&item| { item < &2 }, vec![1])]
    fn test_filter_iter<P: Fn(&&i32) -> bool + Sync + Send>(predicate: P, expected_result: Vec<i32>) {
        assert_eq!(iterate![vec![1, 2, 3, 4]].filter(predicate).cloned().collect::<Vec<_>>(), expected_result);
    }

    #[test_case(|&item| { if item % 2 == 0 {None} else {Some(item*2)}}, vec![2, 6])]
    #[test_case(|&item| { if item == 0 {Some(42)} else {None}}, vec![])]
    fn test_filter_map_iter<P: Fn(&i32) -> Option<i32> + Sync + Send>(filter_op: P, expected_result: Vec<i32>) {
        assert_eq!(iterate![vec![1, 2, 3, 4]].filter_map(filter_op).collect::<Vec<_>>(), expected_result);
    }

    #[test_case(|&mut item| { item > 42 }, false)]
    #[test_case(|&mut item| { item < 2 }, true)]
    fn test_any_iter_mut<P: Fn(&mut i32) -> bool + Sync + Send>(predicate: P, expected_result: bool) {
        assert_eq!(iterate_mutable![vec![1, 2, 3, 4]].any(predicate), expected_result);
    }

    #[test_case(|&&mut item| { item > 10 }, vec![])]
    #[test_case(|&&mut item| { item < 2 }, vec![&mut 1])]
    fn test_filter_iter_mut<P: Fn(&&mut i32) -> bool + Sync + Send>(predicate: P, expected_result: Vec<&mut i32>) {
        assert_eq!(iterate_mutable![vec![1, 2, 3, 4]].filter(predicate).collect::<Vec<_>>(), expected_result);
    }

    #[test_case(|&item| { if item % 2 == 0 {None} else {Some(item*2)}}, vec![2, 6])]
    #[test_case(|&item| { if item == 0 {Some(42)} else {None}}, vec![])]
    fn test_filter_map_iter_mut<P: Fn(&i32) -> Option<i32> + Sync + Send>(filter_op: P, expected_result: Vec<i32>) {
        assert_eq!(iterate![vec![1, 2, 3, 4]].filter_map(filter_op).collect::<Vec<_>>(), expected_result);
    }

    #[test_case(|item| { item > 42 }, false)]
    #[test_case(|item| { item < 2 }, true)]
    fn test_any_into_iter<P: Fn(i32) -> bool + Sync + Send>(predicate: P, expected_result: bool) {
        assert_eq!(into_iterator![vec![1, 2, 3, 4]].any(predicate), expected_result);
    }

    #[test_case(|&item| { item > 10 }, vec![])]
    #[test_case(|&item| { item < 2 }, vec![1])]
    fn test_filter_into_iter<P: Fn(&i32) -> bool + Sync + Send>(predicate: P, expected_result: Vec<i32>) {
        assert_eq!(into_iterator![vec![1, 2, 3, 4]].filter(predicate).collect::<Vec<_>>(), expected_result);
    }

    #[test_case(|item| { if item % 2 == 0 {None} else {Some(item*2)}}, vec![2, 6])]
    #[test_case(|item| { if item == 0 {Some(42)} else {None}}, vec![])]
    fn test_filter_map_into_iter<P: Fn(i32) -> Option<i32> + Sync + Send>(filter_op: P, expected_result: Vec<i32>) {
        assert_eq!(into_iterator![vec![1, 2, 3, 4]].filter_map(filter_op).collect::<Vec<_>>(), expected_result);
    }
}
