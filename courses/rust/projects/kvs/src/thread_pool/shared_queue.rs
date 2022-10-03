use crate::thread_pool::*;
use std::sync::{Arc};
use alloc::task;
use panic_control::CheckedJoinHandle;
use parking_lot::Mutex;
use crossbeam::channel::{Receiver, Sender};
use crossbeam_channel::unbounded;
use std::thread;

/// a status message sent to threads in the event of a shutdown or some 
/// event that must be passed to the co-ordinator
enum StatusMsg {
    Job(Task),
    Shutdown,
}

/// the shared queue thread pool will maintain a shared vec of active tasks, whenever any of the threads
/// is finished with the task they were previously assigned, they will grab the handle of the
/// queue, pop the top task off of the queue and execute it
pub struct SharedQueueThreadPool {
    // jobs is the shared queue of active tasks, it is protected by an Arc<Mutex>
    // the scheduler process will push tasks onto the queue, and workers pull tasks from the queue
    jobs: Arc<Mutex<Vec<StatusMsg>>>,
    // vector of all JoinHandles, referenced when dropped and when spawing tasks
}   

/// worker is the thread structure that will work will be distributed between
struct Worker {
    // reference to the task queue
    jobs: Arc<Mutex<Vec<StatusMsg>>>,
    // prevWorker, the JoinHandle of the previous thread, this way, all threads 
    // keep track of each other
}

/// impl Worker

/// implementation of ThreadPool for a SharedQueueThreadPool 
impl ThreadPool for SharedQueueThreadPool {
    /// create a new NaiveThreadPool with threads available threads
    fn new(threads: i32) -> Result<Box<Self>>{
        // create taskqueue
        let jobs = Arc::new(Mutex::new(Vec::<StatusMsg>::new()));
        // create coord_listener for SharedQueueThreadPool
        let (tx, rx) = unbounded::<Sender<StatusMsg>>();
        // iterate over num threads, and initialize the workers
        for i in  0..threads {
            // clone task queue and the transmitter chan
            let tx = tx.clone();
            let jobs = jobs.clone();
            Worker::new(tx, jobs);
        }
        // Box the SharedQueueThreadPool, and return
        Ok(Box::from(SharedQueueThreadPool{
            jobs: jobs,
            listener_chan: rx,
        }))
    }
    /// spawn a new task as one of the threads in the pool
    fn spawn<F>(&mut self, job: F) where F: FnOnce() + Send + 'static {
        // lock task queue
        let task_queue = self.jobs.lock();
        // now we can push the newest StatusMsg into Queue
        task_queue.push(StatusMsg::Job(Box::from(job)))
        // Mutex will unlock once the MutexGuard goes out of scope
    }
}