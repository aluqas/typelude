use typelude::eval::{Eval, Evaluate, EIf, ECall, ELit};
use typelude::lambda::Apply;
use typelude::kernel::bool::{TyTrue, TyFalse};
use typelude::lambda::list::{Cons, Nil, HeadOr, K};

// =========================================================================
// Tests using src/eval/bridge.rs
// =========================================================================

#[test]
fn test_bridge_simple_apply() {
    struct AddOne;
    struct Zero;
    struct One;
    impl Apply<Zero> for AddOne { type Output = One; }

    // Evaluate< AddOne(Zero) >
    // Using ECall (Call-by-Value)
    type Res = Evaluate<ECall<ELit<AddOne>, ELit<Zero>>>;
    static_assertions::assert_type_eq_all!(Res, One);
}

#[test]
fn test_bridge_lazy_eval() {
    struct V1;
    type L = Cons<V1, Nil>;

    struct HeadOp;

    impl<L> Apply<L> for HeadOp
    where
        L: Apply<K>,
        <L as Apply<K>>::Output: Apply<()>,
    {
        type Output = HeadOr<L, ()>;
    }

    // Case True: Execute HeadOp via ECall
    type ExprTrue = EIf<
        ELit<TyTrue>,
        ECall<ELit<HeadOp>, ELit<L>>,
        ELit<()>
    >;

    type ResTrue = Evaluate<ExprTrue>;
    static_assertions::assert_type_eq_all!(ResTrue, V1);

    // Case False: Return Unit
    type ExprFalse = EIf<
        ELit<TyFalse>,
        ECall<ELit<HeadOp>, ELit<L>>,
        ELit<()>
    >;

    type ResFalse = Evaluate<ExprFalse>;
    static_assertions::assert_type_eq_all!(ResFalse, ());
}
