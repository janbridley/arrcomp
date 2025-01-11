#[macro_export]
macro_rules! arr {
    ($ex:expr, for $x:pat in $input:expr $(, if $cond:expr)+; len $len:expr) => {{
        let mut iter = $input.into_iter();

        if $input.len() != $len {
            let msg = &format!("Expected {} elements, got {}.", $len, $input.len());
            panic!("{}", msg);
        }

        std::array::from_fn::<_, $len, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            (true $(&& $cond)*).then(|| $ex)
        })
    }};

    ($ex:expr, for $x:pat in $input:expr; len $len:expr) => {{
        let mut iter = $input.into_iter();

        if $input.len() != $len {
            let msg = &format!("Expected {} elements, got {}.", $len, $input.len());
            panic!("{}", msg);
        }

        std::array::from_fn::<_, $len, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            $ex
        })
    }};

    // Panic if no expression is provided - otherwise the iteration does nothing.
    (_, for $x:pat in $input:expr $(, if $cond:expr)*; len $len:expr) => {{
        let msg = &format!("Comprehension cannot start with a placeholder ``_``");
        panic!("{}", msg);
    }};

}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rstest::fixture;
    use rstest::rstest;

    #[fixture]
    fn nums() -> [i32; 5] {
        [0, 99, -2, 5, 9]
    }

    #[fixture]
    fn nums_plus_one() -> [i32; 5] {
        [1, 100, -1, 6, 10]
    }

    #[fixture]
    fn pairs() -> [(i32, f64); 5] {
        [(0, 0.0), (99, 9900.0), (-2, 2.0), (5, 30.0), (9, 90.0)]
    }

    #[rstest]
    fn test_nums_identity(nums: [i32; 5]) {
        assert_eq!(arr![x, for x in nums; len 5], nums);
    }

    #[rstest]
    fn test_nums_incremented(nums: [i32; 5], nums_plus_one: [i32; 5]) {
        assert_eq!(arr![x + 1, for x in nums; len 5], nums_plus_one);
    }

    #[rstest]
    fn test_nums_constant_value(nums: [i32; 5]) {
        assert_eq!(arr![12.3, for _ in nums; len 5], [12.3; 5]);
    }

    #[rstest]
    fn test_pairs_first_element(pairs: [(i32, f64); 5]) {
        assert_eq!(arr![x, for (x, _) in pairs; len 5], [0, 99, -2, 5, 9]);
    }

    #[rstest]
    fn test_pairs_second_element_zeroed(pairs: [(i32, f64); 5]) {
        assert_eq!(arr![y * 0.0, for (_, y) in pairs; len 5], [0.0; 5]);
    }

    #[rstest]
    fn test_pairs_constant_tuple(pairs: [(i32, f64); 5]) {
        assert_eq!(arr![(), for _ in pairs; len 5], [(); 5]);
    }

    #[rstest]
    fn test_pairs_swapped_elements(pairs: [(i32, f64); 5]) {
        assert_eq!(
            arr![(y, x), for (x, y) in pairs; len 5],
            arr![(pair.1, pair.0), for pair in pairs; len 5]
        );
    }

    #[rstest]
    fn test_pairs_zipped_product(nums_plus_one: [i32; 5], pairs: [(i32, f64); 5]) {
        assert_eq!(
            arr![(x * z) as f64, for ((x, y), z) in pairs.into_iter().zip(nums_plus_one); len 5],
            arr![y, for (_, y) in pairs; len 5]
        );
    }

    #[rstest]
    #[should_panic]
    fn test_placeholder(nums: [i32; 5]) {
        arr![_, for _ in nums; len 5];
    }
}
