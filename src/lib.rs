/*!
A declarative macro for "list comprehension"-style syntax for creating Rust arrays.

In contrast to most Rust packages of this sort, arrcomp exclusively creates fixed size
arrays without an intermediate heap allocation. This is more performant than standard
Vector-based approaches, but places a few additional restrictions on the allowed syntax.

`arrcomp` Syntax
----------------

```rust
use arrcomp::arr;

# fn main() {
let incremented = arr![x + 1, for x in 0..10; len 10];
// [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

let incremented_if_odd = arr![x + 1, for x in 0..10, if x % 2 == 1; len 10];
// [None, Some(2), None, Some(4), None, Some(6), None, Some(8), None, Some(10)]
# }
```
This Rust adaption provides a familiar and performant interface for creating and
modifying fixed-size arrays. `Option` types allow the use of filters even in cases where
the number of unfiltered outputs is unknown at compile time -- without any dynamic
allocations!

The `arr!` pattern is generally expressed as `f(x), for x in interable, if condition; len N`,
where `f(x)` and `iterable` are any [statement](https://doc.rust-lang.org/reference/statements.html),
`condition` is any statement that evaluates to a `bool`, and and `x` is any [pattern](https://doc.rust-lang.org/reference/patterns.html). Unlike Python, we must also provide a const `N` matching
the length of the provided iterable in order to ensure the output can be sized at compile time.

<div class="warning">
Note that the extended <code>arr![f(x), for x in a, for a in b, ... for c in iterable]</code>
syntax that Python supports is not yet supported. Nested comprehensions like
<code>arr![arr![f(x) for x in outer] for outer in iterable]</code> work as expected.
</div>

Why this crate?
===============

When working with vectors, list comprehensions are most naturally expressed as
`filter/flat_map`s in Rust. Consider the following example, which uses `filter` and
`map` for clarity:

```rust
use arrcomp::arr;
# fn main() {
let incremented_vec: Vec<_> = (0..10).filter(|x| x % 2 == 1).map(|x| x + 1).collect();

// Converting to an array is simple. Note we have to provide the correct array length
let incremented_arr: [i32; 5] = incremented_vec.clone().try_into().unwrap();
assert_eq!(incremented_arr.to_vec(), incremented_vec);
# }
```

Note that, in the example above, we dynamically allocate a vector that gets converted to
an array. In performance-critical contexts this is undesirable. Fortunately, there is
another way:

```rust
# use arrcomp::arr;
# fn main() {
let mut incremented_iter = (0..10)
    .into_iter()
    .map(|x| if x % 2 == 1 { Some(x+1) } else { None });

let iter_copy = incremented_iter.clone();

// std::array::from_fn lets us generate the array without collecting into a vector
let arr_without_allocation: [Option<i32>; 10] = std::array::from_fn(
    |_| incremented_iter.next().unwrap()
);

assert_eq!(arr_without_allocation.to_vec(), iter_copy.collect::<Vec<_>>());
# }
```

The default syntax is a little clunky, and the inputs to `std::array::from_fn` are
limited. Furthermore, an additional variable must be created outside the function call,
as cloning the iterator inside `from_fn` resets the iterator to the beginning.

An array comprehension provides an attractive to this pattern, with a simplified syntax
that allows for arbitrary expressions for our input iterable.

```rust
# use arrcomp::arr;
# fn main() {
let incremented_vec = (0..10).map(|x| if x % 2 == 1 { Some(x+1) } else { None });

let arr_comprehension = arr![x+1, for x in 0..10, if x % 2 == 1; len 10];

assert_eq!(arr_comprehension.to_vec(), incremented_vec.collect::<Vec<_>>());
# }
```

*/

#[macro_export]
macro_rules! arr {
    ($ex:stmt, for $x:pat in $input:expr $(, if $cond:expr)+; len $len:expr) => {{
        let mut iter = $input.into_iter();

        if $input.len() != $len {
            let msg = &format!("Expected {} elements, got {}.", $len, $input.len());
            panic!("{}", msg);
        }

        std::array::from_fn::<_, $len, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            (true $(&& $cond)*).then(|| {$ex})
        })
        // let mut count = 0;
        // let results = std::array::from_fn::<_, $len, _>(|_| {
        //         let $x = iter.next().unwrap_or_default();
        //         (true $(&& $cond)*).then(|| {
        //             count += 1;
        //             $ex
        //         })
        //     });

        // // Return only the number of valid results
        // Iterate through and apply the condition
        // let mut final_array: [<typeof({$ex})>; 0] = []; // Placeholder for the array
        // for i in 0..$len {
        //     let $x = iter.next().unwrap_or_default();
        //     if true $(&& $cond)* {
        //         final_array[i] = Some({$ex});
        //         count += 1;
        //     }
        // }


        // final_array
        // results[..count]
    }};

    ($ex:stmt, for $x:pat in $input:expr; len $len:expr) => {{
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
    /*
        The tests here aim to cover a wide range of possible syntax statements that
        users may wish to include in their list comprehensions. Our comprehension syntax
        is tested against map/filter statements as one would normally use, with the
        additional understanding that such patterns do not translate easily to arrays.
    */
    use super::*;
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
    fn test_nums_statement(nums: [i32; 5]) {
        assert_eq!(arr![{let _ = x + 1;}, for x in nums; len 5], [(); 5]);
    }

    #[rstest]
    fn test_nums_incremented(nums: [i32; 5], nums_plus_one: [i32; 5]) {
        assert_eq!(arr![x + 1, for x in nums; len 5], nums_plus_one);
    }

    #[rstest]
    fn test_nums_with_fn(nums: [i32; 5]) {
        assert_eq!(
            vec![arr![x.abs(), for x in nums; len 5]],
            vec![nums.map(|x| x.abs())]
        );
    }

    #[rstest]
    fn test_nums_constant_value(nums: [i32; 5]) {
        assert_eq!(arr![12.3, for _ in nums; len 5], [12.3; 5]);
    }

    #[rstest]
    fn test_conditional_expressions(nums: [i32; 5]) {
        assert_eq!(
            vec![arr![if x > 0 { 1 } else { 0 }, for x in nums; len 5]],
            vec![nums.map(|x| if x > 0 { 1 } else { 0 })]
        );
    }

    #[rstest]
    fn test_pairs_first_element(pairs: [(i32, f64); 5]) {
        assert_eq!(arr![x, for (x, _) in pairs; len 5], [0, 99, -2, 5, 9]);
    }

    #[rstest]
    fn test_pairs_nested(pairs: [(i32, f64); 5]) {
        assert_eq!(
            vec![arr![arr![y, for (_, _, y) in [(1, 33, x)]; len 1], for (x, _) in pairs; len 5]],
            vec![pairs.map(|p| [p.0])]
        );
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
    fn test_pairs_swapped_and_scaled(pairs: [(i32, f64); 5]) {
        assert_eq!(
            vec![arr![(y * 2.0, x + 10), for (x, y) in pairs; len 5]],
            vec![pairs.map(|(x, y)| (y * 2.0, x + 10))]
        );
    }

    #[rstest]
    fn test_pairs_to_arr(pairs: [(i32, f64); 5]) {
        let other_variable = 43;
        assert_eq!(
            vec![arr![[x - other_variable, y as i32], for (x, y) in pairs; len 5]],
            vec![pairs.map(|(x, y)| [x - other_variable, y as i32])]
        );
    }

    #[rstest]
    fn test_pairs_zipped_product(nums_plus_one: [i32; 5], pairs: [(i32, f64); 5]) {
        assert_eq!(
            arr![(x * z) as f64, for ((x, _), z) in pairs.into_iter().zip(nums_plus_one); len 5],
            arr![y, for (_, y) in pairs; len 5]
        );
    }

    #[rstest]
    fn test_nums_identity_with_cond(nums: [i32; 5]) {
        assert_eq!(
            arr![x, for x in nums, if x % 2 == 0; len 5],
            nums.map(|x| if x % 2 == 0 { Some(x) } else { None })
        );
    }

    #[rstest]
    fn test_nums_statement_with_cond(nums: [i32; 5]) {
        assert_eq!(
            arr![{let _ = x + 1;}, for x in nums, if x > 0; len 5],
            nums.map(|x| if x > 0 { Some(()) } else { None })
        );
    }

    #[rstest]
    fn test_nums_incremented_with_cond(nums: [i32; 5]) {
        assert_eq!(
            arr![x + 1, for x in nums, if x >= 0; len 5],
            nums.map(|x| if x >= 0 { Some(x + 1) } else { None })
        );
    }

    #[rstest]
    fn test_nums_with_fn_with_cond(nums: [i32; 5]) {
        assert_eq!(
            vec![arr![x.abs(), for x in nums, if x != 0; len 5]],
            vec![nums.map(|x| if x != 0 { Some(x.abs()) } else { None })]
        );
    }

    #[rstest]
    #[case::is_odd(|x: i32| x % 2 == 1)]
    #[case::greater_than_5(|x: i32| x > 5)]
    #[case::is_negative(|x: i32| x < 0)]
    #[case::is_zero(|x: i32| x == 0)]
    fn test_nums_constant_value_with_cond(nums: [i32; 5], #[case] cond: fn(i32) -> bool) {
        assert_eq!(
            vec![arr![12.3, for x in nums, if cond(x); len 5]],
            vec![nums.map(|x| if cond(x) { Some(12.3) } else { None })]
        );
    }

    #[rstest]
    #[case::is_odd(|x: i32| x % 2 == 1)]
    #[case::greater_than_5(|x: i32| x > 5)]
    #[case::is_negative(|x: i32| x < 0)]
    #[case::is_zero(|x: i32| x == 0)]
    fn test_pairs_first_element_with_cond(pairs: [(i32, f64); 5], #[case] cond: fn(i32) -> bool) {
        assert_eq!(
            arr![x, for (x, _) in pairs, if cond(x); len 5],
            pairs.map(|(x, _)| if cond(x) { Some(x) } else { None })
        );
    }

    #[rstest]
    fn test_pairs_nested_with_cond(pairs: [(i32, f64); 5]) {
        assert_eq!(
            vec![arr![arr![y, for (_, _, y) in [(1, 33, x)]; len 1],
                for (x, _) in pairs, if x % 2 == 0; len 5]],
            vec![pairs.map(|p| if p.0 % 2 == 0 { Some([p.0]) } else { None })]
        );
    }

    #[rstest]
    fn test_pairs_nested_with_nested_cond(pairs: [(i32, f64); 5]) {
        assert_eq!(
            vec![
                arr![arr![y, for (_, _, y) in [(1, 33, x)], if y > 0; len 1],
                for (x, _) in pairs, if x % 2 == 0; len 5]
            ],
            vec![pairs.map(|p| if p.0 % 2 == 0 {
                Some([if p.0 > 0 { Some(p.0) } else { None }])
            } else {
                None
            })]
        );
    }

    #[rstest]
    fn test_pairs_second_element_zeroed_with_cond(pairs: [(i32, f64); 5]) {
        assert_eq!(
            arr![y * 0.0, for (_, y) in pairs, if true; len 5],
            [Some(0.0); 5]
        );
    }

    #[rstest]
    fn test_pairs_constant_tuple_with_cond(pairs: [(i32, f64); 5]) {
        assert_eq!(arr![(), for _ in pairs, if true; len 5], [Some(()); 5]);
    }

    #[rstest]
    fn test_pairs_swapped_elements_with_cond(pairs: [(i32, f64); 5]) {
        assert_eq!(
            arr![(y, x), for (x, y) in pairs, if x > 0; len 5],
            arr![(pair.1, pair.0), for pair in pairs, if pair.0 > 0; len 5]
        );
    }

    #[rstest]
    fn test_pairs_swapped_and_scaled_with_cond(pairs: [(i32, f64); 5]) {
        assert_eq!(
            vec![arr![(y * 2.0, x + 10), for (x, y) in pairs, if x as f64 + y > 10.0; len 5]],
            vec![pairs.map(|(x, y)| if x as f64 + y > 10.0 {
                Some((y * 2.0, x + 10))
            } else {
                None
            })]
        );
    }

    #[rstest]
    fn test_pairs_to_arr_with_cond(pairs: [(i32, f64); 5]) {
        let other_variable = 43;
        assert_eq!(
            vec![arr![
                [x - other_variable, y as i32],
                for (x, y) in pairs,
                if x > y as i32; len 5
            ]],
            vec![pairs.map(|(x, y)| if x > y as i32 {
                Some([x - other_variable, y as i32])
            } else {
                None
            })]
        );
    }

    #[rstest]
    fn test_pairs_zipped_product_with_cond(nums_plus_one: [i32; 5], pairs: [(i32, f64); 5]) {
        assert_eq!(
            arr![
                (x * z) as f64,
                for ((x, _), z) in pairs.into_iter().zip(nums_plus_one),
                if x > z; len 5
            ],
            arr![y, for (x, y) in pairs, if x as f64 > y + 1.0; len 5]
        );
    }
}
