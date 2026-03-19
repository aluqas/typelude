pub type CallFrame<ReturnProgram, ReturnLocals, ReturnLabels> =
    crate::shared::frame::CallFrame<ReturnProgram, ReturnLocals, ReturnLabels>;

pub type LabelFrame<Continuation, Arity> = crate::shared::frame::LabelFrame<Continuation, Arity>;

pub type ReturnFrame<ReturnLocals, ReturnProgram> =
    crate::shared::frame::ReturnFrame<ReturnLocals, ReturnProgram>;
