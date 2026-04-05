trait Eval {
    type Output;
}

// struct OpAdd<Lhs, Rhs>;
// impl<Lhs, Rhs> Eval for OpAdd<Lhs, Rhs> {
//     type Output = <Lhs as OpAdd<Rhs>>::Output;
// }
// でもそれ<Op, Lhs, Rhs> -> Op<Lhs, Rhs>はできないよｗ
// struct Add<Lhs, Rhs> = <Lhs as OpAdd<Rhs>>::Output;

trait Apply<Fn, Arg> {
    type Output;
}

impl<Fn, Arg> Eval for Apply<Fn, Arg> {
    type Output = <Fn as Apply<Arg>>::Output;
}
