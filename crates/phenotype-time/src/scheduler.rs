//! Simple task scheduler for recurring work

use crate::{Clock, Duration, SystemClock, Timestamp};
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    id: u32,
    name: String,
    interval: Duration,
    last_run: Option<Timestamp>,
    is_running: Arc<AtomicBool>,
    execution_count: Arc<AtomicU32>,
}

impl ScheduledTask {
    #[must_use]
    pub fn new(id: u32, name: String, interval: Duration) -> Self {
        Self {
            id,
            name,
            interval,
            last_run: None,
            is_running: Arc::new(AtomicBool::new(false)),
            execution_count: Arc::new(AtomicU32::new(0)),
        }
    }
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn interval(&self) -> Duration {
        self.interval
    }
    #[must_use]
    pub fn last_run(&self) -> Option<Timestamp> {
        self.last_run
    }
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
    #[must_use]
    pub fn execution_count(&self) -> u32 {
        self.execution_count.load(Ordering::Relaxed)
    }
    #[must_use]
    pub fn is_due(&self, now: Timestamp) -> bool {
        match self.last_run {
            None => true,
            Some(last) => {
                let elapsed_ms = (now.inner() - last.inner())
                    .num_milliseconds()
                    .unsigned_abs();
                elapsed_ms >= self.interval.as_millis()
            }
        }
    }
    pub fn mark_started(&mut self) {
        self.is_running.store(true, Ordering::Relaxed);
    }
    pub fn mark_completed(&mut self, now: Timestamp) {
        self.is_running.store(false, Ordering::Relaxed);
        self.last_run = Some(now);
        self.execution_count.fetch_add(1, Ordering::Relaxed);
    }
}

impl fmt::Display for ScheduledTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task[id={}, name={}, count={}]",
            self.id,
            self.name,
            self.execution_count()
        )
    }
}

#[derive(Debug)]
pub struct Scheduler<C: Clock = SystemClock> {
    clock: C,
    tasks: Vec<ScheduledTask>,
    next_id: u32,
}

impl Scheduler<SystemClock> {
    #[must_use]
    pub fn new() -> Self {
        Self::with_clock(SystemClock)
    }
}

impl Default for Scheduler<SystemClock> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Clock> Scheduler<C> {
    #[must_use]
    pub fn with_clock(clock: C) -> Self {
        Self {
            clock,
            tasks: Vec::new(),
            next_id: 1,
        }
    }
    pub fn schedule(&mut self, name: String, interval: Duration) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.tasks.push(ScheduledTask::new(id, name, interval));
        id
    }
    #[must_use]
    pub fn get_task(&self, id: u32) -> Option<&ScheduledTask> {
        self.tasks.iter().find(|t| t.id == id)
    }
    pub fn get_task_mut(&mut self, id: u32) -> Option<&mut ScheduledTask> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }
    #[must_use]
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
    #[must_use]
    pub fn now(&self) -> Timestamp {
        self.clock.now()
    }
    #[must_use]
    pub fn due_task_ids(&self) -> Vec<u32> {
        let now = self.now();
        self.tasks
            .iter()
            .filter(|t| t.is_due(now))
            .map(|t| t.id)
            .collect()
    }
    pub fn remove(&mut self, id: u32) -> Option<ScheduledTask> {
        self.tasks
            .iter()
            .position(|t| t.id == id)
            .map(|pos| self.tasks.remove(pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockClock;

    #[test]
    fn task_new() {
        let task = ScheduledTask::new(1, "test".to_string(), Duration::from_secs(10));
        assert_eq!(task.id(), 1);
    }
    #[test]
    fn task_is_due_never_run() {
        let task = ScheduledTask::new(1, "test".to_string(), Duration::from_secs(10));
        let now = Timestamp::from_millis(1000).unwrap();
        assert!(task.is_due(now));
    }
    #[test]
    fn scheduler_new() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.task_count(), 0);
    }
    #[test]
    fn scheduler_schedule() {
        let mut scheduler = Scheduler::new();
        let id = scheduler.schedule("task".to_string(), Duration::from_secs(10));
        assert_eq!(id, 1);
    }
    #[test]
    fn scheduler_remove() {
        let mut scheduler = Scheduler::new();
        let id = scheduler.schedule("test".to_string(), Duration::from_secs(10));
        assert!(scheduler.remove(id).is_some());
    }
}
