use std::sync::{Arc, Mutex, Condvar};
use std::time::Duration;

#[derive(Debug, PartialEq)]
enum ThreadStatus {
    Uninitialized,
    Pending,
    Up,
}

#[derive(Clone)]
pub struct ThreadHandle {
    status: Arc<(Mutex<ThreadStatus>, Condvar)>,
}

/// Guarantee that one and only one thread is up
impl ThreadHandle {
    pub fn new() -> Self {
        ThreadHandle { status: Arc::new((Mutex::new(ThreadStatus::Uninitialized), Condvar::new())) }
    }

    pub fn wait_for_thread_up(&self) {
        let (ref lock, ref cvar) = *self.status.clone();
        let mut status = lock.lock().unwrap();
        debug!("wait_for_thread_up: enter");
        loop {
            let result = cvar.wait_timeout(status, Duration::from_millis(10))
                .unwrap();
            debug!("10 milliseconds have passed: result: {:?} {:?}",
                   result,
                   *result.0);
            status = result.0;
            if let ThreadStatus::Up = *status {
                info!("wait_for_thread_up: exit");
                break;
            }
        }
    }

    pub fn notify_thread_up(&self) {
        let (ref lock, ref cvar) = *self.status;
        let mut status = lock.lock().unwrap();
        *status = ThreadStatus::Up;
        info!("notify the condvar that thread is up.");
        cvar.notify_one();
    }

    /// return false if init is already done
    pub fn thread_init(&self) -> bool {
        let (ref lock, _) = *self.status;
        let mut status = lock.lock().unwrap();
        match *status {
            ThreadStatus::Uninitialized => {
                *status = ThreadStatus::Pending;
                true
            }
            _ => false,
        }
    }
}
