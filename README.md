<img src="src/ims/arrcomp_anim.svg" width="600">

List comprehension-style syntax for creating Rust array using declarative macros.

In contrast to most Rust packages of this sort, arrcomp exclusively creates fixed size
arrays without an intermediate heap allocation. This is more performant than standard
Vector-based approaches, but places a few additional restrictions on the allowed syntax.

`arrcomp` Syntax
----------------

```rust
use arrcomp::arr;

let incremented = arr![x + 1, for x in 0..10; len 10];
// [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

let incremented_if_odd = arr![x + 1, for x in 0..10, if x % 2 == 1; len 10];
// [None, Some(2), None, Some(4), None, Some(6), None, Some(8), None, Some(10)]
```

This Rust adaption provides a familiar and performant interface for creating and
modifying fixed-size arrays. `Option` types allow the use of filters even in cases where
the number of unfiltered outputs is unknown at compile time -- without any dynamic
allocations!

The `arr!` pattern is generally expressed as `f(x), for x in iterable, if condition; len N`,
where `f(x)` and `iterable` are any [statement](https://doc.rust-lang.org/reference/statements.html),
`condition` is any statement that evaluates to a `bool`, and and `x` is any [pattern](https://doc.rust-lang.org/reference/patterns.html). Unlike Python, we must also provide a const `N` matching
the length of the provided iterable in order to ensure the output can be sized at compile time.

Why this crate?
===============

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
# use arrcomp::arr;
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
# use arrcomp::arr;
let incremented_vec = (0..10).map(|x| if x % 2 == 1 { Some(x+1) } else { None });

let arr_comprehension = arr![x+1, for x in 0..10, if x % 2 == 1; len 10];
assert_eq!(arr_comprehension.to_vec(), incremented_vec.collect::<Vec<_>>());
```
