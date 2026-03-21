use std::marker::PhantomData;

use static_assertions::assert_type_eq_all;
use typelude_std::core::{Eval, Evaluate};

struct True;
struct False;

trait Bool {}
impl Bool for True {}
impl Bool for False {}

impl Eval for True {
    type Output = True;
}

impl Eval for False {
    type Output = False;
}

trait NotValue {
    type Output: Bool;
}

impl NotValue for True {
    type Output = False;
}

impl NotValue for False {
    type Output = True;
}

trait NandValue<Rhs: Bool> {
    type Output: Bool;
}

impl NandValue<True> for True {
    type Output = False;
}

impl NandValue<False> for True {
    type Output = True;
}

impl NandValue<True> for False {
    type Output = True;
}

impl NandValue<False> for False {
    type Output = True;
}

trait IfValue<Then, Else> {
    type Output;
}

impl<Then, Else> IfValue<Then, Else> for True {
    type Output = Then;
}

impl<Then, Else> IfValue<Then, Else> for False {
    type Output = Else;
}

struct EIf<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);
struct ENand<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);
struct ENot<T>(PhantomData<T>);
struct EAnd<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);
struct EOr<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

trait EIfBranch<Then, Else> {
    type Output;
}

impl<Then, Else> EIfBranch<Then, Else> for True
where
    Then: Eval,
{
    type Output = Evaluate<Then>;
}

impl<Then, Else> EIfBranch<Then, Else> for False
where
    Else: Eval,
{
    type Output = Evaluate<Else>;
}

impl<Cond, Then, Else> Eval for EIf<Cond, Then, Else>
where
    Cond: Eval,
    Evaluate<Cond>: EIfBranch<Then, Else>,
{
    type Output = <Evaluate<Cond> as EIfBranch<Then, Else>>::Output;
}

impl<Lhs, Rhs> Eval for ENand<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Rhs>: Bool,
    Evaluate<Lhs>: NandValue<Evaluate<Rhs>>,
{
    type Output = <Evaluate<Lhs> as NandValue<Evaluate<Rhs>>>::Output;
}

impl<T> Eval for ENot<T>
where
    T: Eval,
    Evaluate<T>: NotValue,
{
    type Output = <Evaluate<T> as NotValue>::Output;
}

impl<Lhs, Rhs> Eval for EAnd<Lhs, Rhs>
where
    ENand<Lhs, Rhs>: Eval,
    ENot<ENand<Lhs, Rhs>>: Eval,
{
    type Output = Evaluate<ENot<ENand<Lhs, Rhs>>>;
}

impl<Lhs, Rhs> Eval for EOr<Lhs, Rhs>
where
    ENot<Lhs>: Eval,
    ENot<Rhs>: Eval,
    ENand<ENot<Lhs>, ENot<Rhs>>: Eval,
{
    type Output = Evaluate<ENand<ENot<Lhs>, ENot<Rhs>>>;
}

struct EExampleWHATTHEFUCKFn<Arg1, Arg2, Arg3, Arg4>(PhantomData<(Arg1, Arg2, Arg3, Arg4)>);

struct Lit<T>(PhantomData<T>);

impl<T> Eval for Lit<T> {
    type Output = T;
}

impl<Arg1, Arg2, Arg3, Arg4> Eval for EExampleWHATTHEFUCKFn<Arg1, Arg2, Arg3, Arg4>
where
    Arg1: Eval,
    Arg2: Eval,
    Arg3: Eval,
    Arg4: Eval,
    EIf<EAnd<ENot<Arg1>, EOr<Arg2, Arg3>>, Arg4, Arg2>: Eval,
{
    type Output = Evaluate<EIf<EAnd<ENot<Arg1>, EOr<Arg2, Arg3>>, Arg4, Arg2>>;
}

#[test]
fn test_if() {
    assert_type_eq_all!(Evaluate<EIf<True, Lit<typenum::U1>, Lit<typenum::U2>>>, typenum::U1);
    assert_type_eq_all!(Evaluate<EIf<False, Lit<typenum::U1>, Lit<typenum::U2>>>, typenum::U2);
}

#[test]
fn test_nand() {
    assert_type_eq_all!(Evaluate<ENand<True, True>>, False);
    assert_type_eq_all!(Evaluate<ENand<True, False>>, True);
    assert_type_eq_all!(Evaluate<ENand<False, True>>, True);
    assert_type_eq_all!(Evaluate<ENand<False, False>>, True);
}

#[test]
fn test_not() {
    assert_type_eq_all!(Evaluate<ENot<True>>, False);
    assert_type_eq_all!(Evaluate<ENot<False>>, True);
}

#[test]
fn test_and() {
    assert_type_eq_all!(Evaluate<EAnd<True, True>>, True);
    assert_type_eq_all!(Evaluate<EAnd<True, False>>, False);
    assert_type_eq_all!(Evaluate<EAnd<False, True>>, False);
    assert_type_eq_all!(Evaluate<EAnd<False, False>>, False);
}

#[test]
fn test_or() {
    assert_type_eq_all!(Evaluate<EOr<True, True>>, True);
    assert_type_eq_all!(Evaluate<EOr<True, False>>, True);
    assert_type_eq_all!(Evaluate<EOr<False, True>>, True);
    assert_type_eq_all!(Evaluate<EOr<False, False>>, False);
}

#[test]
fn test() {
    type Result = Evaluate<EIf<EAnd<True, False>, Lit<typenum::U32>, Lit<typenum::U64>>>;
    assert_type_eq_all!(Result, typenum::U64);
}

#[test]
fn test_example_fn() {
    type Result = Evaluate<EExampleWHATTHEFUCKFn<False, True, False, Lit<typenum::U42>>>;
    assert_type_eq_all!(Result, typenum::U42);
}
