//! Crash-safe storage transaction state machine.
//! This is the policy layer used by journaling/recovery services; it never
//! assumes that a write reached durable media until `Committed` is observed.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JournalState {
    Empty,
    Prepared,
    Committing,
    Committed,
    Aborted,
    NeedsRecovery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JournalError {
    InvalidTransition,
    SequenceOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JournalTxn {
    pub id: u64,
    pub state: JournalState,
    pub records: u16,
}

impl JournalTxn {
    pub const MAX_RECORDS: u16 = 1024;

    pub const fn new(id: u64) -> Self {
        Self {
            id,
            state: JournalState::Empty,
            records: 0,
        }
    }

    pub fn append(&mut self) -> Result<(), JournalError> {
        if self.records == Self::MAX_RECORDS {
            return Err(JournalError::SequenceOverflow);
        }
        if !matches!(self.state, JournalState::Empty | JournalState::Prepared) {
            return Err(JournalError::InvalidTransition);
        }
        self.records += 1;
        self.state = JournalState::Prepared;
        Ok(())
    }

    pub fn begin_commit(&mut self) -> Result<(), JournalError> {
        if self.records == 0 || self.state != JournalState::Prepared {
            return Err(JournalError::InvalidTransition);
        }
        self.state = JournalState::Committing;
        Ok(())
    }

    pub fn durable_commit(&mut self) -> Result<(), JournalError> {
        if self.state != JournalState::Committing {
            return Err(JournalError::InvalidTransition);
        }
        self.state = JournalState::Committed;
        Ok(())
    }

    pub fn abort(&mut self) -> Result<(), JournalError> {
        if matches!(self.state, JournalState::Committed | JournalState::Aborted) {
            return Err(JournalError::InvalidTransition);
        }
        self.state = JournalState::Aborted;
        Ok(())
    }

    pub const fn recover_after_crash(self) -> JournalState {
        match self.state {
            JournalState::Committing | JournalState::Prepared => JournalState::NeedsRecovery,
            state => state,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryDecision {
    pub state: JournalState,
    pub replay: bool,
}

pub const fn decide_recovery(txn: JournalTxn) -> RecoveryDecision {
    match txn.recover_after_crash() {
        JournalState::NeedsRecovery => RecoveryDecision {
            state: JournalState::NeedsRecovery,
            replay: true,
        },
        state => RecoveryDecision {
            state,
            replay: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_is_not_observed_before_durable_commit() {
        let mut txn = JournalTxn::new(7);
        txn.append().unwrap();
        txn.begin_commit().unwrap();
        assert_eq!(txn.state, JournalState::Committing);
        assert_eq!(decide_recovery(txn).replay, true);
        txn.durable_commit().unwrap();
        assert_eq!(decide_recovery(txn).replay, false);
    }

    #[test]
    fn record_capacity_and_illegal_transitions_fail_closed() {
        let mut txn = JournalTxn::new(1);
        assert_eq!(txn.begin_commit(), Err(JournalError::InvalidTransition));
        for _ in 0..JournalTxn::MAX_RECORDS {
            txn.append().unwrap();
        }
        assert_eq!(txn.append(), Err(JournalError::SequenceOverflow));
    }
}
