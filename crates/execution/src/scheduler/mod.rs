//! Scheduler for strategy execution timing.
//!
//! Provides flexible scheduling for:
//! - Periodic evaluations
//! - Time-based triggers
//! - Cron-like scheduling

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{Instant, interval};
use tracing::{debug, info, warn};

/// Schedule type for task execution.
#[derive(Debug, Clone)]
pub enum Schedule {
    /// Run at fixed intervals.
    Interval(Duration),
    /// Run at specific times (hour, minute).
    Daily(Vec<(u8, u8)>),
    /// Run once after delay.
    Once(Duration),
    /// Custom schedule with cron-like expression.
    Cron(String),
}

/// A scheduled task.
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    /// Task name.
    pub name: String,
    /// Schedule.
    pub schedule: Schedule,
    /// Whether task is enabled.
    pub enabled: bool,
    /// Last run time.
    pub last_run: Option<Instant>,
    /// Next scheduled run.
    pub next_run: Option<Instant>,
}

impl ScheduledTask {
    /// Creates a new scheduled task.
    pub fn new(name: impl Into<String>, schedule: Schedule) -> Self {
        Self {
            name: name.into(),
            schedule,
            enabled: true,
            last_run: None,
            next_run: None,
        }
    }

    /// Disables the task.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Event sent when a task should run.
#[derive(Debug, Clone)]
pub struct TaskEvent {
    /// Task name.
    pub task_name: String,
    /// Scheduled time.
    pub scheduled_at: Instant,
    /// Actual trigger time.
    pub triggered_at: Instant,
}

/// Scheduler for managing task execution timing.
pub struct Scheduler {
    /// Scheduled tasks.
    tasks: Vec<ScheduledTask>,
    /// Event sender.
    event_tx: mpsc::Sender<TaskEvent>,
    /// Event receiver.
    event_rx: Option<mpsc::Receiver<TaskEvent>>,
    /// Running flag.
    running: Arc<AtomicBool>,
}

impl Scheduler {
    /// Creates a new scheduler.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            tasks: Vec::new(),
            event_tx: tx,
            event_rx: Some(rx),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Adds a task to the scheduler.
    pub fn add_task(&mut self, task: ScheduledTask) {
        info!(task = %task.name, "Adding task to scheduler");
        self.tasks.push(task);
    }

    /// Removes a task by name.
    pub fn remove_task(&mut self, name: &str) {
        self.tasks.retain(|t| t.name != name);
    }

    /// Enables a task by name.
    pub fn enable_task(&mut self, name: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.name == name) {
            task.enabled = true;
        }
    }

    /// Disables a task by name.
    pub fn disable_task(&mut self, name: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.name == name) {
            task.enabled = false;
        }
    }

    /// Takes the event receiver for processing events.
    pub fn take_receiver(&mut self) -> Option<mpsc::Receiver<TaskEvent>> {
        self.event_rx.take()
    }

    /// Starts the scheduler.
    pub async fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);

        info!(tasks = self.tasks.len(), "Starting scheduler");

        // Initialize next run times
        let now = Instant::now();
        for task in &mut self.tasks {
            let next = Self::calculate_next_run_static(&task.schedule, now);
            task.next_run = Some(next);
        }

        // Main scheduler loop
        let mut check_interval = interval(Duration::from_secs(1));

        while self.running.load(Ordering::SeqCst) {
            check_interval.tick().await;

            let now = Instant::now();

            // Collect events to send
            let mut events_to_send = Vec::new();

            for task in &mut self.tasks {
                if !task.enabled {
                    continue;
                }

                if let Some(next_run) = task.next_run
                    && now >= next_run
                {
                    // Task should run
                    let event = TaskEvent {
                        task_name: task.name.clone(),
                        scheduled_at: next_run,
                        triggered_at: now,
                    };

                    events_to_send.push((task.name.clone(), event));

                    task.last_run = Some(now);
                    let next = Self::calculate_next_run_static(&task.schedule, now);
                    task.next_run = Some(next);

                    debug!(
                        task = %task.name,
                        next_run = ?task.next_run,
                        "Task triggered"
                    );
                }
            }

            // Send events outside the mutable borrow
            for (task_name, event) in events_to_send {
                if let Err(e) = self.event_tx.send(event).await {
                    warn!(task = %task_name, error = %e, "Failed to send task event");
                }
            }
        }

        info!("Scheduler stopped");
    }

    /// Stops the scheduler.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Calculates the next run time for a schedule (static version).
    fn calculate_next_run_static(schedule: &Schedule, from: Instant) -> Instant {
        match schedule {
            Schedule::Interval(duration) => from + *duration,
            Schedule::Once(delay) => from + *delay,
            Schedule::Daily(_times) => {
                // Simplified: just run in 24 hours
                // A real implementation would calculate based on wall clock time
                from + Duration::from_secs(24 * 60 * 60)
            }
            Schedule::Cron(_expr) => {
                // Simplified: just run in 1 hour
                // A real implementation would parse the cron expression
                from + Duration::from_secs(60 * 60)
            }
        }
    }

    /// Gets all tasks.
    pub fn tasks(&self) -> &[ScheduledTask] {
        &self.tasks
    }

    /// Checks if the scheduler is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating common schedules.
pub struct ScheduleBuilder;

impl ScheduleBuilder {
    /// Creates an interval schedule.
    pub fn every(duration: Duration) -> Schedule {
        Schedule::Interval(duration)
    }

    /// Creates a schedule that runs every N seconds.
    pub fn every_secs(secs: u64) -> Schedule {
        Schedule::Interval(Duration::from_secs(secs))
    }

    /// Creates a schedule that runs every N minutes.
    pub fn every_mins(mins: u64) -> Schedule {
        Schedule::Interval(Duration::from_secs(mins * 60))
    }

    /// Creates a schedule that runs every N hours.
    pub fn every_hours(hours: u64) -> Schedule {
        Schedule::Interval(Duration::from_secs(hours * 60 * 60))
    }

    /// Creates a one-time schedule.
    pub fn once_after(delay: Duration) -> Schedule {
        Schedule::Once(delay)
    }

    /// Creates a daily schedule at specific times.
    pub fn daily_at(times: Vec<(u8, u8)>) -> Schedule {
        Schedule::Daily(times)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_builder() {
        let schedule = ScheduleBuilder::every_mins(5);
        assert!(matches!(schedule, Schedule::Interval(_)));

        if let Schedule::Interval(d) = schedule {
            assert_eq!(d, Duration::from_secs(300));
        }
    }

    #[test]
    fn test_scheduled_task() {
        let task = ScheduledTask::new("test", ScheduleBuilder::every_secs(60));
        assert!(task.enabled);
        assert_eq!(task.name, "test");
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        let mut scheduler = Scheduler::new();
        scheduler.add_task(ScheduledTask::new("test", ScheduleBuilder::every_secs(1)));

        assert_eq!(scheduler.tasks().len(), 1);
    }
}
