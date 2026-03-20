use static_assertions::assert_type_eq_all;

struct True;
struct False;

trait Bool {}
impl Bool for True {}
impl Bool for False {}

trait IfHelper<Cond: Bool, Then, Else> {
    type Output;
}

impl<Then, Else> IfHelper<True, Then, Else> for () {
    type Output = Then;
}

impl<Then, Else> IfHelper<False, Then, Else> for () {
    type Output = Else;
}

trait NandHelper<Lhs: Bool, Rhs: Bool> {
    type Output;
}

impl NandHelper<True, True> for () {
    type Output = False;
}

impl NandHelper<True, False> for () {
    type Output = True;
}

impl NandHelper<False, True> for () {
    type Output = True;
}

impl NandHelper<False, False> for () {
    type Output = True;
}

trait NotHelper<T: Bool> {
    type Output;
}

impl<T: Bool> NotHelper<T> for ()
where
    (): NandHelper<T, T>,
{
    type Output = <() as NandHelper<T, T>>::Output;
}

trait AndHelper<Lhs: Bool, Rhs: Bool> {
    type Output;
}

impl<Lhs: Bool, Rhs: Bool> AndHelper<Lhs, Rhs> for ()
where
    (): NandHelper<Lhs, Rhs> + NotHelper<Nand<Lhs, Rhs>>,
    Nand<Lhs, Rhs>: Bool,
    Not<Nand<Lhs, Rhs>>: Bool,
{
    type Output = Not<Nand<Lhs, Rhs>>;
}

trait OrHelper<Lhs: Bool, Rhs: Bool> {
    type Output;
}

impl<Lhs: Bool, Rhs: Bool> OrHelper<Lhs, Rhs> for ()
where
    (): NotHelper<Lhs> + NotHelper<Rhs> + NandHelper<Not<Lhs>, Not<Rhs>>,
    Not<Lhs>: Bool,
    Not<Rhs>: Bool,
    Nand<Not<Lhs>, Not<Rhs>>: Bool,
{
    type Output = Nand<Not<Lhs>, Not<Rhs>>;
}

type If<Cond: Bool, Then, Else> = <() as IfHelper<Cond, Then, Else>>::Output;

type Nand<Lhs: Bool, Rhs: Bool> = <() as NandHelper<Lhs, Rhs>>::Output;
type Not<T: Bool> = <() as NotHelper<T>>::Output;
type And<Lhs: Bool, Rhs: Bool> = <() as AndHelper<Lhs, Rhs>>::Output;
type Or<Lhs: Bool, Rhs: Bool> = <() as OrHelper<Lhs, Rhs>>::Output;

#[test]
fn test_if() {
    assert_type_eq_all!(If<True, u32, u64>, u32);
    assert_type_eq_all!(If<False, u32, u64>, u64);
}

#[test]
fn test_nand() {
    assert_type_eq_all!(Nand<True, True>, False);
    assert_type_eq_all!(Nand<True, False>, True);
    assert_type_eq_all!(Nand<False, True>, True);
    assert_type_eq_all!(Nand<False, False>, True);
}

#[test]
fn test_not() {
    assert_type_eq_all!(Not<True>, False);
    assert_type_eq_all!(Not<False>, True);
}

#[test]
fn test_and() {
    assert_type_eq_all!(And<True, True>, True);
    assert_type_eq_all!(And<True, False>, False);
    assert_type_eq_all!(And<False, True>, False);
    assert_type_eq_all!(And<False, False>, False);
}

#[test]
fn test_or() {
    assert_type_eq_all!(Or<True, True>, True);
    assert_type_eq_all!(Or<True, False>, True);
    assert_type_eq_all!(Or<False, True>, True);
    assert_type_eq_all!(Or<False, False>, False);
}

#[test]
fn test() {
    assert_type_eq_all!(If<And<True, False>, u32, u64>, u64);
}
