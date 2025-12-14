use typenum::op;
use typenum::U5;

fn main() {
    type X = op!(5);
    // Verify compatibility
    let _: X = U5::default();
}
