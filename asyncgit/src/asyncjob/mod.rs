//! provides `AsyncJob` trait and `AsyncSingleJob` struct

#![deny(clippy::expect_used)]

use std::sync::{Arc, Mutex, RwLock};

use crossbeam_channel::Sender;

use crate::error::Result;

/// Passed to `AsyncJob::run` allowing sending intermediate progress
/// notifications
pub struct RunParams<T: Copy + Send, P: Clone + Send + Sync + PartialEq> {
    sender: Sender<T>,
    progress: Arc<RwLock<P>>,
}

impl<T: Copy + Send, P: Clone + Send + Sync + PartialEq> RunParams<T, P> {
    /// send an intermediate update notification.
    /// do not confuse this with the return value of `run`.
    /// `send` should only be used about progress notifications
    /// and not for the final notification indicating the end of the
    /// async job. see `run` for more info
    pub fn send(&self, notification: T) -> Result<()> {
        self.sender.send(notification)?;
        Ok(())
    }

    /// set the current progress
    pub fn set_progress(&self, p: P) -> Result<bool> {
        Ok(if *self.progress.read()? == p {
            false
        } else {
            *(self.progress.write()?) = p;
            true
        })
    }
}

/// trait that defines an async task we can run on a threadpool
pub trait AsyncJob: Send + Sync + Clone {
    /// defines what notification type is used to communicate outside
    type Notification: Copy + Send;
    /// type of progress
    type Progress: Clone + Default + Send + Sync + PartialEq;

    /// can run a synchronous time intensive task.
    /// the returned notification is used to tell interested parties
    /// that the job finished and the job can be access via
    /// `take_last`. prior to this final notification it is not safe
    /// to assume `take_last` will already return the correct job
    fn run(
        &mut self,
        params: RunParams<Self::Notification, Self::Progress>,
    ) -> Result<Self::Notification>;

    /// allows observers to get intermediate progress status if the
    /// job customizes it by default this will be returning
    /// `Self::Progress::default()`
    fn get_progress(&self) -> Self::Progress {
        Self::Progress::default()
    }
}

/// Abstraction for a FIFO task queue that will only queue up **one**
/// `next` job. It keeps overwriting the next job until it is actually
/// taken to be processed
#[derive(Debug, Clone)]
pub struct AsyncSingleJob<J: AsyncJob> {
    next: Arc<Mutex<Option<J>>>,
    last: Arc<Mutex<Option<J>>>,
    progress: Arc<RwLock<J::Progress>>,
    sender: Sender<J::Notification>,
    pending: Arc<Mutex<()>>,
}

impl<J: 'static + AsyncJob> AsyncSingleJob<J> {
    ///
    pub fn new(sender: Sender<J::Notification>) -> Self {
        Self {
            next: Arc::new(Mutex::new(None)),
            last: Arc::new(Mutex::new(None)),
            pending: Arc::new(Mutex::new(())),
            progress: Arc::new(RwLock::new(J::Progress::default())),
            sender,
        }
    }

    ///
    pub fn is_pending(&self) -> bool {
        self.pending.try_lock().is_err()
    }

    /// makes sure `next` is cleared and returns `true` if it actually
    /// canceled something
    pub fn cancel(&mut self) -> bool {
        if let Ok(mut next) = self.next.lock() {
            if next.is_some() {
                *next = None;
                return true;
            }
        }

        false
    }

    /// take out last finished job
    pub fn take_last(&self) -> Option<J> {
        self.last.lock().map_or(None, |mut last| last.take())
    }

    /// spawns `task` if nothing is running currently,
    /// otherwise schedules as `next` overwriting if `next` was set
    /// before. return `true` if the new task gets started right
    /// away.
    pub fn spawn(&mut self, task: J) -> bool {
        self.schedule_next(task);
        self.check_for_job()
    }

    ///
    pub fn progress(&self) -> Option<J::Progress> {
        self.progress.read().ok().map(|d| (*d).clone())
    }

    fn check_for_job(&self) -> bool {
        if self.is_pending() {
            return false;
        }

        if let Some(task) = self.take_next() {
            let self_clone = (*self).clone();
            rayon_core::spawn(move || {
                if let Err(e) = self_clone.run_job(task) {
                    log::error!("async job error: {}", e);
                }
            });

            return true;
        }

        false
    }

    fn run_job(&self, mut task: J) -> Result<()> {
        //limit the pending scope
        {
            let _pending = self.pending.lock()?;

            let notification = task.run(RunParams {
                progress: self.progress.clone(),
                sender: self.sender.clone(),
            })?;

            if let Ok(mut last) = self.last.lock() {
                *last = Some(task);
            }

            self.sender.send(notification)?;
        }

        self.check_for_job();

        Ok(())
    }

    fn schedule_next(&mut self, task: J) {
        if let Ok(mut next) = self.next.lock() {
            *next = Some(task);
        }
    }

    fn take_next(&self) -> Option<J> {
        self.next.lock().map_or(None, |mut next| next.take())
    }
}

#[cfg(test)]
mod test {
    use std::{
        sync::atomic::{AtomicBool, AtomicU32, Ordering},
        thread,
        time::Duration,
    };

    use crossbeam_channel::unbounded;
    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(Clone)]
    struct TestJob {
        v: Arc<AtomicU32>,
        finish: Arc<AtomicBool>,
        value_to_add: u32,
        notification_value: u32, // Added field
    }

    impl AsyncJob for TestJob {
        type Notification = u32; // Changed from ()
        type Progress = ();

        fn run(
            &mut self,
            params: RunParams<Self::Notification, Self::Progress>, // Use params
        ) -> Result<Self::Notification> {
            println!("[job] wait");

            while !self.finish.load(Ordering::SeqCst) {
                std::thread::yield_now();
            }

            println!("[job] sleep");

            thread::sleep(Duration::from_millis(100));

            println!("[job] done sleeping");

            // Send notification before adding value
            params.send(self.notification_value)?;

            let res = self.v.fetch_add(self.value_to_add, Ordering::SeqCst);

            println!("[job] value: {res}");

            Ok(self.notification_value) // Return the notification value
        }
    }

    #[test]
    #[ignore]
    fn test_overwrite() {
        let (sender, receiver) = unbounded();

        let mut job: AsyncSingleJob<TestJob> = AsyncSingleJob::new(sender);

        let shared_v = Arc::new(AtomicU32::new(1));
        let shared_finish = Arc::new(AtomicBool::new(false));

        let task_template = TestJob {
            v: Arc::clone(&shared_v),
            finish: Arc::clone(&shared_finish),
            value_to_add: 1,
            notification_value: 10,
        };

        // First job
        let first_task = task_template.clone();
        assert!(job.spawn(first_task));

        // Subsequent jobs (only the last one will run)
        for _ in 0..5 {
            println!("spawn");
            let next_task = task_template.clone();
            assert!(!job.spawn(next_task));
        }

        shared_finish.store(true, Ordering::SeqCst);

        println!("recv");
        assert_eq!(receiver.recv().unwrap(), 10);
        assert_eq!(receiver.recv().unwrap(), 10);
        assert!(receiver.is_empty());

        assert_eq!(shared_v.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    fn wait_for_job(job: &AsyncSingleJob<TestJob>) {
        while job.is_pending() {
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn test_cancel() {
        let (sender, receiver) = unbounded();

        let mut job: AsyncSingleJob<TestJob> = AsyncSingleJob::new(sender);

        // Define the first job
        let initial_task = TestJob {
            v: Arc::new(AtomicU32::new(1)),
            finish: Arc::new(AtomicBool::new(false)),
            value_to_add: 1,
            notification_value: 20,
        };

        // Spawn the first job
        assert!(job.spawn(initial_task.clone()));
        initial_task.finish.store(true, Ordering::SeqCst); // Signal the first job to finish
        thread::sleep(Duration::from_millis(10)); // Give it time to start

        // Schedule subsequent jobs (these will be cancelled)
        for _ in 0..5 {
            println!("spawn");
            // Use a new task definition for subsequent jobs, or ensure they are distinct if needed
            // For now, cloning the initial_task definition is fine as they are meant to be cancelled.
            assert!(!job.spawn(initial_task.clone()));
        }

        println!("cancel");
        assert!(job.cancel()); // Cancel the queued jobs

        // Wait for the first job to complete
        wait_for_job(&job);

        println!("recv");
        // Assert the received notification value from the first job
        assert_eq!(receiver.recv().unwrap(), 20);
        println!("received");

        // Retrieve the completed job from `last`
        let completed_job = job.take_last().expect("Should have a completed job");

        // Assert the value of the completed job
        // Initial value was 1, it should have been incremented by 1 (value_to_add)
        assert_eq!(completed_job.v.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
