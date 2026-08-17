#![no_std]
use super::abi::{ERR_INVALID_ARGUMENT, ERR_OK, ERR_PERMISSION, Syscall, SyscallResult};
use crate::process::{ProcessDescriptor, ProcessState};
pub struct SyscallContext<'a> {
    pub process: &'a mut ProcessDescriptor,
}
impl<'a> SyscallContext<'a> {
    pub fn dispatch(&mut self, number: u64, args: [u64; 6]) -> SyscallResult {
        let call = match number {
            0 => Syscall::Yield,
            1 => Syscall::Exit,
            2 => Syscall::Spawn,
            3 => Syscall::IpcSend,
            4 => Syscall::IpcRecv,
            5 => Syscall::Map,
            6 => Syscall::Unmap,
            7 => Syscall::Read,
            8 => Syscall::Write,
            _ => return err(ERR_INVALID_ARGUMENT),
        };
        match call {
            Syscall::Yield => {
                self.process.state = ProcessState::Runnable;
                ok(0)
            }
            Syscall::Exit => {
                self.process.state = ProcessState::Exited;
                ok(0)
            }
            Syscall::Spawn => err(ERR_PERMISSION),
            Syscall::IpcSend | Syscall::IpcRecv => {
                if !self.process.budget.consume_ipc(1) {
                    return err(ERR_PERMISSION);
                }
                ok(args[0])
            }
            Syscall::Map => {
                if !self.process.budget.permits_memory(args[0]) {
                    return err(ERR_PERMISSION);
                }
                ok(args[0])
            }
            Syscall::Unmap => ok(args[0]),
            Syscall::Read | Syscall::Write => {
                if args[1] == 0 {
                    return err(ERR_INVALID_ARGUMENT);
                }
                ok(args[1])
            }
        }
    }
}
const fn ok(value: u64) -> SyscallResult {
    SyscallResult {
        value,
        error: ERR_OK,
    }
}
const fn err(error: u64) -> SyscallResult {
    SyscallResult { value: 0, error }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{ProcessId, ResourceBudget};
    fn ctx() -> SyscallContext<'static> {
        static mut PROCESS: ProcessDescriptor = ProcessDescriptor {
            id: ProcessId(1),
            state: ProcessState::Running,
            budget: ResourceBudget {
                cpu_ticks: 10,
                memory_bytes: 4096,
                ipc_messages: 2,
            },
        };
        let ptr = &raw mut PROCESS;
        unsafe { SyscallContext { process: &mut *ptr } }
    }
    #[test]
    fn invalid_call_is_rejected() {
        let mut c = ctx();
        assert_eq!(c.dispatch(99, [0; 6]).error, ERR_INVALID_ARGUMENT)
    }
    #[test]
    fn exit_changes_state() {
        let mut c = ctx();
        assert_eq!(c.dispatch(Syscall::Exit as u64, [0; 6]).error, ERR_OK);
        assert_eq!(c.process.state, ProcessState::Exited)
    }
    #[test]
    fn ipc_consumes_budget() {
        let mut c = ctx();
        assert_eq!(
            c.dispatch(Syscall::IpcSend as u64, [7, 0, 0, 0, 0, 0])
                .error,
            ERR_OK
        );
        assert_eq!(
            c.dispatch(Syscall::IpcSend as u64, [8, 0, 0, 0, 0, 0])
                .error,
            ERR_OK
        );
        assert_eq!(
            c.dispatch(Syscall::IpcSend as u64, [9, 0, 0, 0, 0, 0])
                .error,
            ERR_PERMISSION
        )
    }
}
