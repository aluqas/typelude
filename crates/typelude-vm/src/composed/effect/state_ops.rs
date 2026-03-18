use crate::composed::core::{Bind, MonadState, Pure};

pub type GetVm<F, State> = <F as MonadState<State>>::Get;
pub type PutVm<F, State, NewState> = <F as MonadState<State>>::Put<NewState>;
pub type ModifyVm<F, State, Func> = <F as MonadState<State>>::Modify<Func>;
pub type ReturnVm<F, A> = Pure<F, A>;
pub type Then<F, MA, MB> = Bind<F, MA, crate::composed::core::traits::LConst<MB>>;
