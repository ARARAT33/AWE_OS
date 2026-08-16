#![no_std]

use super::context::ProcessContext;
use super::context_switch::SwitchFrame;
use super::dispatch::{prepare_dispatch, DispatchError, DispatchTarget};
use super::scheduler::{Scheduler, SchedulerError};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuntimeDispatchError { Scheduler(SchedulerError), Dispatch(DispatchError) }
pub fn prepare_runtime_switch<const N: usize>(scheduler:&mut Scheduler<N>,current:&SwitchFrame,contexts:&[ProcessContext;N])->Result<DispatchTarget,RuntimeDispatchError>{let target=scheduler.prepare_next(contexts).map_err(RuntimeDispatchError::Scheduler)?;prepare_dispatch(current,&target).map_err(RuntimeDispatchError::Dispatch)?;Ok(target)}
