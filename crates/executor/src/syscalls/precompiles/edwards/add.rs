use std::marker::PhantomData;

use sp1_curves::{edwards::EdwardsParameters, EllipticCurve};

use crate::{
    events::create_ec_add_event,
    syscalls::{Syscall, SyscallCode, SyscallContext},
};

pub(crate) struct EdwardsAddAssignSyscall<E: EllipticCurve + EdwardsParameters> {
    _phantom: PhantomData<E>,
}

impl<E: EllipticCurve + EdwardsParameters> EdwardsAddAssignSyscall<E> {
    /// Create a new instance of the [`EdwardsAddAssignSyscall`].
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<E: EllipticCurve + EdwardsParameters> Syscall for EdwardsAddAssignSyscall<E> {
    fn num_extra_cycles(&self) -> u32 {
        1
    }

    fn execute(
        &self,
        rt: &mut SyscallContext,
        _: SyscallCode,
        arg1: u32,
        arg2: u32,
    ) -> Option<u32> {
        let _ = create_ec_add_event::<E>(rt, arg1, arg2);
        None
    }
}
