use typelude_std::{std::debug::trace::Trace, tyarray};
use typenum::*;

#[test]
fn test_trace_bool() {
    assert_eq!(B0::fmt(), "false");
    assert_eq!(B1::fmt(), "true");
}

macro_rules! test_uint_trace {
    ($($N:literal),*) => {
        $(
            paste::paste! {
                #[test]
                fn [<test_trace_uint_ $N>]() {
                    let expected = $N.to_string();
                    let actual = <typenum::consts::[<U $N>] as Trace>::fmt();
                    assert_eq!(actual, expected);
                }
            }
        )*
    }
}

test_uint_trace!(0, 1, 2, 3, 4, 5, 8, 10, 16, 32, 42, 64, 100, 128);

#[test]
fn test_trace_signed_int() {
    // Assert Z0 (Zero)
    assert_eq!(Z0::fmt(), "0", "Trace for Z0 failed");

    // Assert PInt (Positive)
    // Note: PInt<U> corresponds to +U
    assert_eq!(<PInt<U1> as Trace>::fmt(), "1", "Trace for PInt<U1> failed");
    assert_eq!(<PInt<U5> as Trace>::fmt(), "5", "Trace for PInt<U5> failed");
    assert_eq!(<PInt<U100> as Trace>::fmt(), "100", "Trace for PInt<U100> failed");

    // Assert NInt (Negative)
    // Note: NInt<U> corresponds to -U
    assert_eq!(<NInt<U1> as Trace>::fmt(), "-1", "Trace for NInt<U1> failed");
    assert_eq!(<NInt<U5> as Trace>::fmt(), "-5", "Trace for NInt<U5> failed");
    assert_eq!(<NInt<U100> as Trace>::fmt(), "-100", "Trace for NInt<U100> failed");
}

#[test]
fn test_trace_list_simple() {
    type L = tyarray![U1, U2, U3];
    assert_eq!(<L as Trace>::fmt(), "[1, 2, 3]");
}

#[test]
fn test_trace_list_empty() {
    type L = tyarray![];
    assert_eq!(<L as Trace>::fmt(), "[]");

    // Explicit check for Nil::fmt_list via TraceList internal usage not
    // possible directly as TraceList is private? Wait, TraceList is private
    // trait in module, or public? It's private in `src/std/debug/trace.rs`:
    // `trait TraceList`. So we can only test it indirectly via `Trace::fmt`
    // on List types. The mutant `TraceList for Nil::fmt_list -> ""` vs
    // `String::new()` is essentially the same. If it returns something
    // else, `Trace for Nil` which calls `fmt_list` might fail?
    // Trace for Nil implementation:
    // impl Trace for Nil { fn fmt() -> String { "[]".to_string() } }
    // It does NOT use `TraceList::fmt_list`.
    //
    // Wait, `impl<Head, Tail> Trace for Array<Head, Tail>` uses
    // `TraceList::fmt_list`. The base case for recursion in `TraceList` is
    // `impl TraceList for Nil`. It returns empty string "".
    // If a list has elements, `Array::fmt_list` calls `Tail::fmt_list`.
    // Eventually Tail is Nil. `fmt_list` checks `tail.is_empty()`.
    // IF `Nil::fmt_list` returned "xyzzy", then `tail` would not be empty?
    // `Nil::fmt_list` returns `""`.
    // Where is `TraceList` used?
    // `Array::fmt_list` calls `Tail::fmt_list()`.
    // If Tail is Nil, it returns `""`.
    // Then in `Array::fmt_list`: `let tail = Tail::fmt_list(); if
    // tail.is_empty() { head } else { ... }` If `Nil::fmt_list` returns
    // "xyzzy", `tail` is NOT empty. So it returns "head, xyzzy".
    // Then `Trace::fmt` returns "[head, xyzzy]".
    // This should fail `test_trace_list_simple`.
    //
    // Why did it NOT fail?
    // Perhaps `tyarray![U1]` -> `Array<U1, Nil>`.
    // `TraceList for Array<U1, Nil>`:
    // head = "1"
    // tail = Nil::fmt_list() = "" (or "xyzzy" if mutated)
    // if tail.is_empty() -> returns "1".
    // If mutated to "xyzzy" -> not empty -> returns "1, xyzzy".
    // `Trace::fmt` -> "[1, xyzzy]".
    // `test_trace_list_simple` asserts "[1, 2, 3]".
    // It SHOULD fail.
    // Maybe `test_trace_list_simple` was not running in the mutant checking
    // process? Ah, `cargo mutants` only runs tests in the package being
    // mutated by default? `Trace` is in `typelude-std`.
    // `trace_proptests.rs` is in `typelude-std`. It should have run.
    //
    // Let's ensure we have a test for a single element list too, just in case.
}

#[test]
fn test_trace_single_element() {
    type L = tyarray![U1];
    assert_eq!(<L as Trace>::fmt(), "[1]");
}

#[test]
fn test_trace_nested() {
    // Nested lists: [[1, 2], [3, 4]]
    type L1 = tyarray![U1, U2];
    type L2 = tyarray![U3, U4];
    type Nested = tyarray![L1, L2];
    assert_eq!(<Nested as Trace>::fmt(), "[[1, 2], [3, 4]]");
}

#[test]
fn test_trace_mixed() {
    // [true, 0, [1]]
    type L = tyarray![B1, U0, tyarray![U1]];
    assert_eq!(<L as Trace>::fmt(), "[true, 0, [1]]");
}
