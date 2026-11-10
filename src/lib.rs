#![doc(html_logo_url = "https://github.com/janbridley/arrcomp/blob/main/src/ims/arrcomp.svg")]

/*!
List comprehension-style syntax for creating Rust arrays using declarative macros.
Python-style list comprehension syntax for contiguous memory Rust arrays.

In contrast to most Rust packages of this sort, arrcomp exclusively creates fixed size
arrays without an intermediate heap allocation. This is more performant than standard
Vector-based approaches, but places a few additional restrictions on the allowed syntax.

`arrcomp` Syntax
----------------

```rust
use arrcomp::arr;

# fn main() {
let incremented = arr![x + 1, for x in 0..10];
// [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

let incremented_if_odd = arr![x + 1, for x in 0..10, if x % 2 == 1];
// [None, Some(2), None, Some(4), None, Some(6), None, Some(8), None, Some(10)]
# }
```
This Rust adaption provides a familiar and performant interface for creating and
modifying fixed-size arrays. `Option` types allow the use of filters even in cases where
the number of unfiltered outputs is unknown at compile time -- without any dynamic
allocations!

The `arr!` pattern is generally expressed as `f(x), for x in iterable, if condition`, where
`f(x)` and `iterable` are any [statement](https://doc.rust-lang.org/reference/statements.html),
`condition` is any statement that evaluates to a `bool`, and `x` is any [pattern](https://doc.rust-lang.org/reference/patterns.html).
The output length is inferred at compile time whenever it is statically knowable: constant
ranges (`0..10`, `5..12`, `0..=9`, `0..N`), and any iterable
whose type carries its length (arrays and most iterators built from them, see [`KnownLength`]).
Iterables whose length is only known at runtime (`0..n` for a runtime `n`, `Vec`s, slice
iterators, `filter`/`chain`/`flatten` adapters) instead take a const `N` after a semicolon:
`arr![x + 1, for x in 0..n; len 10]`.

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

The default Rust syntax is a little clunky, and the inputs to `std::array::from_fn` are
limited. Furthermore, an additional variable must be created outside the function call,
as cloning the iterator inside `from_fn` resets the iterator to the beginning.

An array comprehension provides an attractive alternative to this pattern, with a
simplified syntax that allows for arbitrary expressions for our input iterable.

```rust
# use arrcomp::arr;
# fn main() {
let incremented_vec = (0..10).map(|x| if x % 2 == 1 { Some(x+1) } else { None });

let arr_comprehension = arr![x+1, for x in 0..10, if x % 2 == 1; len 10];

assert_eq!(arr_comprehension.to_vec(), incremented_vec.collect::<Vec<_>>());
# }
```

*/

/**
Marker trait for iterables whose exact element count `N` is known at compile time.

Because a given type implements `KnownLength<N>` for exactly one `N`, the compiler can
*infer* the const generic from the iterable's type alone, allowing users to omit `len`.

Implementations are provided for `[T; N]`, `std::array::IntoIter<T, N>`, `std::iter::Empty`
(length 0), `std::iter::Once` (length 1), and the length-preserving iterator adapters
`map`, `zip` (both sides must have equal length), `enumerate`, `rev`, `copied`, `cloned`,
`fuse`, and `peekable`.

Most iterators do *not* have a statically known length: ranges over runtime bounds (`0..n`),
`Vec`s and slice iterators, `filter`/`chain`/`flatten`/`take`/`skip`, and anything built on
I/O. Feed those to [`arr!`](crate::arr) with an explicit `; len N` instead. Constant-bounded
ranges (`0..10`, `0..N`) are sized them directly from their bounds:

```rust,compile_fail
use arrcomp::arr;

# fn main() {
// `n` is a runtime value, so `0..n` has no compile-time length
let n = 10;
let range_comprehension = arr![x + 1, for x in 0..n];
# }
```
**/
pub trait KnownLength<const N: usize> {}

impl<T, const N: usize> KnownLength<N> for [T; N] {}
impl<T, const N: usize> KnownLength<N> for std::array::IntoIter<T, N> {}
impl<T> KnownLength<0> for std::iter::Empty<T> {}
impl<T> KnownLength<1> for std::iter::Once<T> {}

impl<I, F, const N: usize> KnownLength<N> for std::iter::Map<I, F> where I: KnownLength<N> {}
impl<A, B, const N: usize> KnownLength<N> for std::iter::Zip<A, B>
where
    A: KnownLength<N>,
    B: KnownLength<N>,
{
}
impl<I, const N: usize> KnownLength<N> for std::iter::Enumerate<I> where I: KnownLength<N> {}
impl<I, const N: usize> KnownLength<N> for std::iter::Rev<I> where I: KnownLength<N> {}
impl<I, const N: usize> KnownLength<N> for std::iter::Copied<I> where I: KnownLength<N> {}
impl<I, const N: usize> KnownLength<N> for std::iter::Cloned<I> where I: KnownLength<N> {}
impl<I, const N: usize> KnownLength<N> for std::iter::Fuse<I> where I: KnownLength<N> {}
impl<I, const N: usize> KnownLength<N> for std::iter::Peekable<I> where I: Iterator + KnownLength<N> {}

// Support function for the `arr!` macro's inferred-length branches. It exists because
// only a generic function can have a const generic *inferred* from a trait bound: with
// `I: KnownLength<N>`, the sole `N` satisfied by the iterable's type becomes the array
// length. It must remain `pub` -- the exported macro calls it as `$crate::collect_array`
// inside downstream crates -- but it is an implementation detail, so it is hidden from
// the documentation. `f` is the macro-built per-element closure: it binds the `for`
// pattern and evaluates the comprehension expression (wrapped in `Option` for filtered
// comprehensions). Short iterators fall back to `Default` values for the remaining
// slots, which cannot happen for a correctly implemented `KnownLength`.
#[doc(hidden)]
pub fn collect_array<I, T, F, const N: usize>(iter: I, mut f: F) -> [T; N]
where
    I: IntoIterator + KnownLength<N>,
    I::Item: Default,
    F: FnMut(I::Item) -> T,
{
    let mut iter = iter.into_iter();

    std::array::from_fn(|_| f(iter.next().unwrap_or_default()))
}

/**
The `arr!` macro has three length modes -- inferred from a constant-bounded range, inferred
from the iterable's type via `KnownLength`, or explicit `; len` -- each with filtered and
unfiltered branches.

Ranges of the form `0..$bound` are sized directly from the bound, which works even when the
bound is a const generic parameter: `arr![x, for x in 0..N]` inside `fn f<const N: usize>()`.
Other constant bounds (`5..12`, `0..=9`) are sized by const arithmetic on the bounds.

`arr![$ex, for $x in 0..$bound]`
`arr![$ex, for $x in $lo..$hi]`
`arr![$ex, for $x in $input]`
`arr![$ex, for $x in $input; len $len]`
**/
#[macro_export]
macro_rules! arr {
    ($ex:stmt, for $x:pat in $input:expr $(, if $cond:expr)+; len $len:expr) => {{
        let mut iter = $input.into_iter();

        if iter.len() != $len {
            panic!("Expected {} elements, got {}.", $len, iter.len());
        }
        std::array::from_fn::<_, $len, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            (true $(&& $cond)*).then(|| {$ex})
        })
    }};


    ($ex:stmt, for $x:pat in $input:expr; len $len:expr) => {{
        let mut iter = $input.into_iter();

        if iter.len() != $len {
            panic!("Expected {} elements, got {}.", $len, iter.len());
        }

        std::array::from_fn::<_, $len, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            $ex
        })
    }};

    // `0..bound`: the length *is* the bound, so it can be passed as a standalone const
    // argument. This is the only form that supports const generic bounds (`0..N`).
    ($ex:stmt, for $x:pat in 0 .. $hi:tt $(, if $cond:expr)+) => {{
        let mut iter = (0..$hi).into_iter();

        std::array::from_fn::<_, $hi, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            (true $(&& $cond)*).then(|| {$ex})
        })
    }};

    ($ex:stmt, for $x:pat in 0 .. $hi:tt) => {{
        let mut iter = (0..$hi).into_iter();

        std::array::from_fn::<_, $hi, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            $ex
        })
    }};

    // constant bounds with a non-zero start (or an inclusive end) require const
    // arithmetic, which excludes const generic bounds: use `; len` for those.
    ($ex:stmt, for $x:pat in $lo:tt ..= $hi:tt $(, if $cond:expr)+) => {{
        let mut iter = ($lo..=$hi).into_iter();

        std::array::from_fn::<_, { ($hi) - ($lo) + 1 }, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            (true $(&& $cond)*).then(|| {$ex})
        })
    }};

    ($ex:stmt, for $x:pat in $lo:tt ..= $hi:tt) => {{
        let mut iter = ($lo..=$hi).into_iter();

        std::array::from_fn::<_, { ($hi) - ($lo) + 1 }, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            $ex
        })
    }};

    ($ex:stmt, for $x:pat in $lo:tt .. $hi:tt $(, if $cond:expr)+) => {{
        let mut iter = ($lo..$hi).into_iter();

        std::array::from_fn::<_, { ($hi) - ($lo) }, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            (true $(&& $cond)*).then(|| {$ex})
        })
    }};

    ($ex:stmt, for $x:pat in $lo:tt .. $hi:tt) => {{
        let mut iter = ($lo..$hi).into_iter();

        std::array::from_fn::<_, { ($hi) - ($lo) }, _>(|_| {
            let $x = iter.next().unwrap_or_default();
            $ex
        })
    }};

    // any iterable whose type carries its length: see `KnownLength`
    ($ex:stmt, for $x:pat in $input:expr $(, if $cond:expr)+) => {{
        $crate::collect_array($input, |$x| {(true $(&& $cond)*).then(|| {$ex})})
    }};

    ($ex:stmt, for $x:pat in $input:expr) => {{
        $crate::collect_array($input, |$x| {$ex})
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

    /*
        Length inference, part one: constant-bounded ranges.
    */

    #[rstest]
    fn test_range_from_literals() {
        assert_eq!(arr![x + 1, for x in 0..10], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(arr![x, for x in 5..12], [5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(arr![x, for x in 0..=9], [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[rstest]
    fn test_range_empty() {
        let empty: [i32; 0] = arr![x, for x in 0..0];
        assert_eq!(empty, []);
    }

    #[rstest]
    fn test_range_with_cond() {
        assert_eq!(
            arr![x + 1, for x in 0..10, if x % 2 == 1],
            [
                None,
                Some(2),
                None,
                Some(4),
                None,
                Some(6),
                None,
                Some(8),
                None,
                Some(10)
            ]
        );
    }

    #[rstest]
    fn test_range_const_bound() {
        const LIMIT: usize = 4;
        assert_eq!(arr![x, for x in 0..LIMIT], [0, 1, 2, 3]);
        assert_eq!(arr![x, for x in 1..LIMIT], [1, 2, 3]);
    }

    fn doubled<const N: usize>() -> [usize; N] {
        arr![x * 2, for x in 0..N]
    }

    #[rstest]
    fn test_range_const_generic_bound() {
        assert_eq!(doubled::<5>(), [0, 2, 4, 6, 8]);
        assert_eq!(doubled::<0>(), []);
    }

    /*
        Length inference, part two: iterables whose *type* carries its length
    */

    #[rstest]
    fn test_inferred_identity(nums: [i32; 5]) {
        assert_eq!(arr![x, for x in nums], nums);
    }

    #[rstest]
    fn test_inferred_incremented(nums: [i32; 5], nums_plus_one: [i32; 5]) {
        assert_eq!(arr![x + 1, for x in nums], nums_plus_one);
    }

    #[rstest]
    fn test_inferred_with_cond(nums: [i32; 5]) {
        assert_eq!(
            arr![x, for x in nums, if x % 2 == 0],
            nums.map(|x| if x % 2 == 0 { Some(x) } else { None })
        );
    }

    #[rstest]
    fn test_inferred_array_literal() {
        assert_eq!(arr![x * 2, for x in [1, 2, 3]], [2, 4, 6]);
    }

    #[rstest]
    fn test_inferred_nested(pairs: [(i32, f64); 5]) {
        assert_eq!(
            vec![arr![arr![y, for (_, _, y) in [(1, 33, x)]], for (x, _) in pairs]],
            vec![pairs.map(|p| [p.0])]
        );
    }

    #[rstest]
    fn test_inferred_map_adapter(nums: [i32; 5]) {
        assert_eq!(
            arr![x + 1, for x in nums.into_iter().map(|v| v * 2)],
            nums.map(|v| v * 2 + 1)
        );
    }

    #[rstest]
    fn test_inferred_zip_adapter(pairs: [(i32, f64); 5], nums_plus_one: [i32; 5]) {
        assert_eq!(
            arr![(x * z) as f64, for ((x, _), z) in pairs.into_iter().zip(nums_plus_one)],
            arr![y, for (_, y) in pairs]
        );
    }

    #[rstest]
    fn test_inferred_adapter_chain(nums: [i32; 5]) {
        assert_eq!(
            arr![x, for x in nums.into_iter().enumerate().map(|(i, v)| i as i32 + v).rev()],
            [13, 8, 0, 100, 0]
        );
    }

    #[rstest]
    fn test_inferred_copied() {
        assert_eq!(
            arr![x + 1, for x in [&1, &2, &3].into_iter().copied()],
            [2, 3, 4]
        );
    }

    #[rstest]
    fn test_inferred_once_and_empty() {
        assert_eq!(arr![x + 1, for x in std::iter::once(41)], [42]);
        assert_eq!(arr![x, for x in std::iter::empty::<i32>()], []);
    }

    #[rstest]
    fn test_non_copy_elements() {
        let words = [String::from("hello"), String::from("arrcomp")];
        assert_eq!(arr![w.len(), for w in words], [5, 7]);
    }

    #[rstest]
    fn test_non_copy_elements_with_len() {
        let words = [String::from("hello"), String::from("arrcomp")];
        assert_eq!(arr![w.len(), for w in words; len 2], [5, 7]);
    }

    #[rstest]
    fn test_collect_array_direct(nums: [i32; 5], nums_plus_one: [i32; 5]) {
        assert_eq!(collect_array(nums, |x| x + 1), nums_plus_one);
        assert_eq!(
            collect_array(nums.into_iter().map(|v| v * 2), |x| x - 1),
            nums.map(|v| v * 2 - 1)
        );
    }
}
