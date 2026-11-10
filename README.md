<img src="src/ims/arrcomp_anim.svg" width="600">

[![Crates.io][crates-badge]][crates-url]
![docs.rs](https://img.shields.io/docsrs/arrcomp)
[![BSD-3 licensed][bsd-badge]][bsd-url]

[crates-badge]: https://img.shields.io/crates/v/arrcomp.svg
[crates-url]: https://crates.io/crates/arrcomp
[bsd-badge]: https://img.shields.io/badge/license-BSD3-blue.svg
[bsd-url]: https://github.com/janbridley/arrcomp/blob/main/LICENSE

Python-style list comprehension syntax for contiguous memory Rust arrays.

In contrast to most Rust packages of this sort, arrcomp exclusively creates fixed size
arrays without an intermediate heap allocation. This is more performant than standard
Vector-based approaches, but places a few additional restrictions on the allowed syntax.

## `arrcomp` Syntax

```rust
use arrcomp::arr;

let incremented: [i32; 10] = arr![x + 1, for x in 0..10];
assert_eq!(incremented, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

let incremented_if_odd: [Option<i32>; 10] = arr![x + 1, for x in 0..10, if x % 2 == 1];
assert_eq!(
    incremented_if_odd,
    [None, Some(2), None, Some(4), None, Some(6), None, Some(8), None, Some(10)]
);
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
whose type carries its length (arrays and most iterators built from them, see `KnownLength`).
Iterables whose length is only known at runtime (`0..n` for a runtime `n`, `Vec`s, slice
iterators, `filter`/`chain`/`flatten` adapters) instead take a const `N` after a semicolon:
`arr![x + 1, for x in 0..n; len 10]`.

Multiple `for` clauses
----------------------

Multiple `for` clauses iterate the Cartesian product Python-style, with each clause's
iterable and conditions able to reference the bindings of all previous clauses:

```rust
use arrcomp::arr;

let products: [i32; 6] = arr![x * 10 + y, for x in 0..2, for y in 0..3];
assert_eq!(products, [0, 1, 2, 10, 11, 12]);

let evens_only: [Option<i32>; 4] = arr![x * y, for x in 1..3, if x % 2 == 0, for y in 0..2];
assert_eq!(evens_only, [Some(0), Some(2), None, None]);
```

The output length is the product of the clause lengths, inferred whenever every clause is
a constant range and otherwise given with `; len N`. The product of type-inferred lengths
cannot be expressed on stable Rust, so `arr![f, for x in nums, for y in nums]` does not
compile. Supply `; len N` for those cases instead:

```rust,compile_fail
let nums = [1, 2];
let products = arr![x * y, for x in nums, for y in nums];
```

Conditions on outer clauses skip the entire sub-iteration (as in Python), so filtered
results compress towards the front and the tail of the array is padded with `None` --
unlike single-`for` comprehensions, where each filtered element becomes `None` in place.
Inner iterables are re-evaluated for each outer element, so they must be re-creatable
expressions (ranges, literals, `Copy` arrays, etc) rather than moved bindings.
Nested comprehensions like `arr![arr![f(x), for x in outer], for outer in iterable]`
continue to work as expected.

Patterns, bindings, and local state
-----------------------------------

`arrcomp::arr` is a superset of the functionality provided by Python list
comprehensions, supporting a greater array of allowed expressions and mutable state.

The loop variable may be any irrefutable pattern, including destructuring (with `mut` if
the element itself should be modified), and the comprehension body is an arbitrary
Rust statement, so it can bind named intermediates inside a block:

```rust
use arrcomp::arr;

let hypotenuses: [f64; 3] = arr![
    { let sq = x * x + y * y; (sq as f64).sqrt() },
    for (x, y) in [(3.0, 4.0), (5.0, 12.0), (8.0, 15.0)]
];
assert_eq!(hypotenuses, [5.0, 13.0, 17.0]);
```

The body is an `FnMut` closure, so mutable outer bindings are updated as elements are
produced. This allows for cumulative sums, for example:

```rust
use arrcomp::arr;

let mut total = 0;
let cumulative: [i32; 4] = arr![{ total += x; total }, for x in [1, 2, 3, 4]];
assert_eq!(cumulative, [1, 3, 6, 10]);
```

Inner clauses see every binding of the outer clauses, so an inner iterable's bounds may
depend on the outer value. Here the dependent inner ranges yield exactly
`0 + 1 + 2 + 3` elements:

```rust
use arrcomp::arr;

let triangle: [i32; 6] = arr![y, for x in 0..4, for y in 0..x; len 6];
assert_eq!(triangle, [0, 0, 1, 0, 1, 2]);
```

Mutable bindings do not carry *into* multiple-`for` clauses: each inner clause captures
its own copy, so updates reset per outer element. Share state across loops through a
[`Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html) behind a reference instead:

```rust
use arrcomp::arr;

let next = std::cell::Cell::new(0);
let id = &next;
let labelled: [(i32, i32); 6] = arr![
    { id.set(id.get() + 1); (id.get(), x * 10 + y) },
    for x in 0..2,
    for y in 0..3
];
assert_eq!(labelled, [(1, 0), (2, 1), (3, 2), (4, 10), (5, 11), (6, 12)]);
```

# Why this crate?

When working with vectors, list comprehensions are most naturally expressed as
`filter/flat_map`s in Rust. Consider the following example, which uses `filter` and
`map` for clarity:

```rust
use arrcomp::arr;
let incremented_vec: Vec<_> = (0..10).filter(|x| x % 2 == 1).map(|x| x + 1).collect();

// Converting to an array is simple. Note we have to provide the correct array length
let incremented_arr: [i32; 5] = incremented_vec.clone().try_into().unwrap();
assert_eq!(incremented_arr.to_vec(), incremented_vec);
```

Note that, in the example above, we dynamically allocate a vector that gets converted to
an array. In performance-critical contexts this is undesirable. Fortunately, there is
another way:

```rust
let mut incremented_iter = (0..10)
    .into_iter()
    .map(|x| if x % 2 == 1 { Some(x+1) } else { None });

let iter_copy = incremented_iter.clone();

// std::array::from_fn lets us generate the array without collecting into a vector
let arr_without_allocation: [Option<i32>; 10] = std::array::from_fn(
    |_| incremented_iter.next().unwrap()
);

assert_eq!(arr_without_allocation.to_vec(), iter_copy.collect::<Vec<_>>());
```

The default Rust syntax is a little clunky, and the inputs to `std::array::from_fn` are
limited. Furthermore, an additional variable must be created outside the function call,
as cloning the iterator inside `from_fn` resets the iterator to the beginning.

An array comprehension provides an attractive alternative to this pattern, with a
simplified syntax that allows for arbitrary expressions for our input iterable.

```rust
let incremented_vec = (0..10).map(|x| if x % 2 == 1 { Some(x+1) } else { None });

let arr_comprehension = arr![x+1, for x in 0..10, if x % 2 == 1];
assert_eq!(arr_comprehension.to_vec(), incremented_vec.collect::<Vec<_>>());
```
