use super::{RestartPolicy, ServiceId, ServiceSpec, ServiceState, validate_spec};

pub const MAX_DEPENDENCIES: usize = 8;
pub const MAX_RUNTIME_SERVICES: usize = 8;
pub const MAX_FAILURES: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeError { Full, Duplicate, InvalidSpec, InvalidDependency, DependencyCycle, MissingDependency, SpawnFailed, InvalidTransition, Quarantined }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceRuntimeSpec {
    pub spec: ServiceSpec,
    pub dependencies: [Option<ServiceId>; MAX_DEPENDENCIES],
    pub dependency_count: u8,
    pub entry: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeRecord { pub spec: ServiceRuntimeSpec, pub state: ServiceState, pub process_id: u64, pub failures: u8 }

pub type SpawnFn = fn(entry: usize, service: ServiceId, memory_pages: u32, cpu_budget: u32) -> Result<u64, ()>;

pub struct Supervisor {
    records: [Option<RuntimeRecord>; MAX_RUNTIME_SERVICES],
    count: usize,
    spawn: SpawnFn,
}
impl Supervisor {
    pub const fn new(spawn: SpawnFn) -> Self { Self { records: [None; MAX_RUNTIME_SERVICES], count: 0, spawn } }
    pub const fn len(&self) -> usize { self.count }
    pub fn register(&mut self, runtime: ServiceRuntimeSpec) -> Result<(), RuntimeError> {
        validate_spec(runtime.spec).map_err(|_| RuntimeError::InvalidSpec)?;
        if runtime.dependency_count as usize > MAX_DEPENDENCIES || runtime.entry == 0 { return Err(RuntimeError::InvalidSpec); }
        if self.find(runtime.spec.id).is_some() { return Err(RuntimeError::Duplicate); }
        for index in 0..runtime.dependency_count as usize { let dep = runtime.dependencies[index].ok_or(RuntimeError::InvalidDependency)?; if dep == runtime.spec.id { return Err(RuntimeError::DependencyCycle); } }
        let slot = self.records.iter().position(Option::is_none).ok_or(RuntimeError::Full)?;
        self.records[slot] = Some(RuntimeRecord { spec: runtime, state: ServiceState::Declared, process_id: 0, failures: 0 });
        self.count += 1; Ok(())
    }
    pub fn start(&mut self, id: ServiceId) -> Result<u64, RuntimeError> {
        let index = self.find(id).ok_or(RuntimeError::MissingDependency)?;
        let runtime = self.records[index].ok_or(RuntimeError::MissingDependency)?;
        if runtime.state == ServiceState::Quarantined { return Err(RuntimeError::Quarantined); }
        for dependency_index in 0..runtime.spec.dependency_count as usize {
            let dep = runtime.spec.dependencies[dependency_index].ok_or(RuntimeError::InvalidDependency)?;
            let dep_index = self.find(dep).ok_or(RuntimeError::MissingDependency)?;
            if self.records[dep_index].ok_or(RuntimeError::MissingDependency)?.state != ServiceState::Running { return Err(RuntimeError::MissingDependency); }
        }
        if !matches!(runtime.state, ServiceState::Declared | ServiceState::Failed | ServiceState::Stopped) { return Err(RuntimeError::InvalidTransition); }
        self.records[index].as_mut().unwrap().state = ServiceState::Starting;
        let pid = match (self.spawn)(runtime.spec.entry, id, runtime.spec.spec.memory_limit_pages, runtime.spec.spec.cpu_budget_ticks) {
            Ok(pid) if pid != 0 => pid,
            _ => { self.records[index].as_mut().unwrap().state = ServiceState::Failed; return Err(RuntimeError::SpawnFailed); }
        };
        let record = self.records[index].as_mut().unwrap();
        record.process_id = pid; record.failures = 0; record.state = ServiceState::Running; Ok(pid)
    }
    pub fn report_failure(&mut self, id: ServiceId) -> Result<ServiceState, RuntimeError> {
        let index = self.find(id).ok_or(RuntimeError::MissingDependency)?;
        let record = self.records[index].as_mut().unwrap();
        if record.state != ServiceState::Running { return Err(RuntimeError::InvalidTransition); }
        record.failures = record.failures.saturating_add(1);
        record.state = if record.failures > MAX_FAILURES { ServiceState::Quarantined } else { ServiceState::Failed };
        Ok(record.state)
    }
    pub fn restart(&mut self, id: ServiceId) -> Result<u64, RuntimeError> {
        let index = self.find(id).ok_or(RuntimeError::MissingDependency)?;
        let record = self.records[index].ok_or(RuntimeError::MissingDependency)?;
        if record.state == ServiceState::Quarantined || record.failures > MAX_FAILURES { return Err(RuntimeError::Quarantined); }
        match record.spec.spec.restart { RestartPolicy::Never => Err(RuntimeError::InvalidTransition), RestartPolicy::OnFailure | RestartPolicy::Always => self.start(id) }
    }
    pub fn state(&self, id: ServiceId) -> Option<ServiceState> { self.find(id).and_then(|i| self.records[i].map(|r| r.state)) }
    pub fn process_id(&self, id: ServiceId) -> Option<u64> { self.find(id).and_then(|i| self.records[i].map(|r| r.process_id)).filter(|pid| *pid != 0) }
    fn find(&self, id: ServiceId) -> Option<usize> { self.records.iter().position(|r| r.map(|record| record.spec.spec.id) == Some(id)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spawn(entry: usize, service: ServiceId, _mem: u32, _cpu: u32) -> Result<u64, ()> { if entry == 0 || service.0 == 0 { Err(()) } else { Ok((entry as u64) ^ service.0 as u64) } }
    fn spec(id: u16, deps: [Option<ServiceId>; MAX_DEPENDENCIES], count: u8) -> ServiceRuntimeSpec { ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(id), restart: RestartPolicy::OnFailure, capability_mask: u64::MAX, memory_limit_pages: 4, cpu_budget_ticks: 100 }, dependencies: deps, dependency_count: count, entry: id as usize + 1 } }
    #[test] fn dependencies_are_enforced() { let mut s=Supervisor::new(spawn); s.register(spec(1,[None;MAX_DEPENDENCIES],0)).unwrap(); s.register(spec(2,[Some(ServiceId(1)),None,None,None,None,None,None,None],1)).unwrap(); assert_eq!(s.start(ServiceId(2)),Err(RuntimeError::MissingDependency)); s.start(ServiceId(1)).unwrap(); assert!(s.start(ServiceId(2)).is_ok()); }
    #[test] fn failure_escalates_to_quarantine() { let mut s=Supervisor::new(spawn); s.register(spec(1,[None;MAX_DEPENDENCIES],0)).unwrap(); s.start(ServiceId(1)).unwrap(); for _ in 0..MAX_FAILURES { assert_eq!(s.report_failure(ServiceId(1)).unwrap(),ServiceState::Failed); s.start(ServiceId(1)).unwrap(); } assert_eq!(s.report_failure(ServiceId(1)).unwrap(),ServiceState::Quarantined); }
}