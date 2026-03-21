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

trait NandHelper2<Lhs> {
    type Output;
}

impl NandHelper2<True> for True {
    type Output = False;
}

impl NandHelper2<False> for True {
    type Output = True;
}

impl NandHelper2<True> for False {
    type Output = True;
}

impl NandHelper2<False> for False {
    type Output = True;
}

type If<Cond: Bool, Then, Else> = <() as IfHelper<Cond, Then, Else>>::Output;

type Nand<Lhs: Bool, Rhs: Bool> = <() as NandHelper<Lhs, Rhs>>::Output;

type Not<T: Bool> = Nand<T, T>;
type And<Lhs: Bool, Rhs: Bool> = Not<Nand<Lhs, Rhs>>;
type Or<Lhs: Bool, Rhs: Bool> = Nand<Not<Lhs>, Not<Rhs>>;
type Nor<Lhs: Bool, Rhs: Bool> = Not<Or<Lhs, Rhs>>;

type Xor<Lhs: Bool, Rhs: Bool> = Or<And<Lhs, Not<Rhs>>, And<Not<Lhs>, Rhs>>;
type Xnor<Lhs: Bool, Rhs: Bool> = Not<Xor<Lhs, Rhs>>;

type HalfAdder<Lhs: Bool, Rhs: Bool> = (Xor<Lhs, Rhs>, And<Lhs, Rhs>);
type FullAdder<Lhs: Bool, Rhs: Bool, CarryIn: Bool> =
    (Xor<Xor<Lhs, Rhs>, CarryIn>, Or<And<Lhs, Rhs>, And<Or<Lhs, Rhs>, CarryIn>>);

type ExampleWHATTHEFUCKFn<Arg1, Arg2, Arg3, Arg4> = If<And<Not<Arg1>, Or<Arg2, Arg3>>, Arg4, Arg2>;

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
    assert_type_eq_all!(ExampleWHATTHEFUCKFn<True, True, True, False>, True);

    println!("{}", std::any::type_name::<ExampleWHATTHEFUCKFn<True, True, True, False>>());
}
