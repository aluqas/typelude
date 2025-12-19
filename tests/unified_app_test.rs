use static_assertions::assert_type_eq_all;
use typelude::{Apply, Eval, Evaluate};
use typelude::std::int::OpAdd;
use typelude::eval::App;
use typelude::typenum::{U1, U2, U3};

#[test]
fn test_unified_app() {
    // 1. Specific AST (EAdd) via Apply trait
    // OpAdd + (U1, U2) -> EAdd<U1, U2>
    type Ast = <OpAdd as Apply<(U1, U2)>>::Output;

    // 2. Generic App (App<Op, Args>)
    // App<OpAdd, (U1, U2)>
    // When Evaluated, it should delegate to OpAdd::Apply -> EAdd -> 3.
    type Generic = App<OpAdd, (U1, U2)>;

    // Verify Evaluation
    assert_type_eq_all!(Evaluate<Ast>, U3);
    assert_type_eq_all!(Evaluate<Generic>, U3);
}
