use typelude_std::std::reify::Reify;

#[test]
fn test_reify_array_unsafe_copy() {
    // This targets the specific unsafe logic in Reify for Array
    // N > 1 case
    use typelude_std::{
        tyarray,
        typenum::{U1, U2, U3},
    };

    type Arr = tyarray![U1, U2, U3];
    let reified: [usize; 3] = <Arr as Reify<[usize; 3]>>::reify();
    assert_eq!(reified, [1, 2, 3]);

    // N = 1 case (boundary condition for the loop/copy logic)
    type Arr1 = tyarray![U1];
    let reified1: [usize; 1] = <Arr1 as Reify<[usize; 1]>>::reify();
    assert_eq!(reified1, [1]);

    // If > became >= in `if N > 1`, it would copy for N=1?
    // `ptr::copy_nonoverlapping(tail_ptr, out_ptr.add(1), N - 1)`
    // If N=1, N-1 = 0. Copy 0 bytes. Safe.
    // If the condition `if N > 1` is removed or changed to `if N >= 1`:
    // For N=1, `copy(..., ..., 0)` is a no-op.
    // So `replace > with >=` might be semantically equivalent?
    // If so, we can't kill the mutant easily unless the logic *fails* or
    // *panics* or produces wrong result. But `reify` logic is correct
    // either way for N=1 if copy count is 0.
    //
    // The mutant might be "unviable" or "equivalent".
    // If the code says `if N > 1`, replacing with `if N >= 1` logic:
    // Case N=1: `1 >= 1` is true. Executes copy 0 bytes.
    // Case N=1: `1 > 1` is false. Skips copy.
    // Both define correct behavior.
    // However, mutation testing tools try to find redundancy.
    // If `if N > 1` can be `if N >= 1` without change, maybe the check is
    // redundant? But `copy_nonoverlapping` with count 0 is valid.
    //
    // Let's create a test that specifically stresses this if possible?
    // Unlikely to differentiate.
    // We can mark it as allowed skip if we confirm it's equivalent.
}
