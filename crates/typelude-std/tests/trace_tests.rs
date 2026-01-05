use typelude_std::{std::debug::trace::Trace, tyarray};
use typenum::{B0, B1, U0, U1, U2};

#[test]
fn test_trace_fmt() {
    assert_eq!(B0::fmt(), "false");
    assert_eq!(B1::fmt(), "true");
    assert_eq!(U0::fmt(), "0");
    assert_eq!(U1::fmt(), "1");

    type List = tyarray![U1, U2];
    assert_eq!(<List as Trace>::fmt(), "[1, 2]");

    type Empty = typelude_std::std::col::array::Nil;
    assert_eq!(<Empty as Trace>::fmt(), "[]");
}
