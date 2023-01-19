use crate::thread_pool::*;
use crossbeam::channel::{Receiver, Sender};
use crossbeam_channel::unbounded;
use parking_lot::Mutex;
use std::thread::{self, JoinHandle};
use std::{collections::VecDeque, sync::Arc};

/// a status message sent to threads in the event of a shutdown or some
/// event that must be passed to the co-ordinator
enum StatusMsg {
    Job(Task),
    Shutdown,
    Panic,
}

/// the shared queue thread pool will maintain a shared vec of active tasks, whenever any of the threads
/// is finished with the task they were previously assigned, they will grab the handle of the
/// queue, pop the top task off of the queue and execute it
pub struct SharedQueueThreadPool {
    // jobs is the shared queue of active tasks, it is protected by an Arc<Mutex>
    // the scheduler process will push tasks onto the queue, and workers pull tasks from the queue
    jobs: Arc<Mutex<VecDeque<StatusMsg>>>,
    // number of threads that are shared in this thread pool
    threads: i32,
    // handles,
    handles: Vec::<JoinHandle<()>>,
}

/// worker is the thread structure that will work will be distributed between
struct Worker {
    // reference to the task queue
    jobs: Arc<Mutex<VecDeque<StatusMsg>>>,
    // help_chan is the channel through which panics are communicated to thread pool
    help_chan: Receiver<StatusMsg>,
    // panic_chain, is the sender of the Panic message, in the event that a thread panics
    panic_chan: Sender<StatusMsg>,
}

/// impl Drop for Worker, before dropping the Channel / arc, make sure that all
/// threads are notified that a thread has failed
impl Drop for Worker {
    fn drop(&mut self) {
        // this thread is panicking, send on the Panic channel so another thread
        // receives this message
        if thread::panicking() {
            self.panic_chan.send(StatusMsg::Panic); // callstack is already unwinding here, don't panic on Err
        } else {
            // this thread is dropping from Shutdown, send Shutdown to help thread
            self.panic_chan.send(StatusMsg::Shutdown);
        }
    }
}

/// impl Worker,
impl Worker {
    fn run(&mut self) {
        loop {
            // pop element from jobs
            let mut job_guard = self.jobs.lock();
            if let Some(task) = job_guard.pop_front() {
                match task {
                    StatusMsg::Shutdown => {
                        // thread is shutting down, return
                        // mutex guard dropped here
                        return;
                    }
                    StatusMsg::Job(avail_task) => {
                        // drop the MutexGuard so write access is available
                        drop(job_guard);
                        // execute task
                        avail_task();
                    },
                    // Panic will not be sent over jobs
                    _ => (),
                }
            }
        }
    }
    fn help(&mut self) {
        loop {
            // otherwise, run help thread
            if let Ok(msg) = self.help_chan.try_recv() {
                match msg {
                    StatusMsg::Panic => {
                        // spawn a new task
                        let jobs = self.jobs.clone();
                        let help_chan = self.help_chan.clone();
                        let panic_chan = self.panic_chan.clone();
                        // spawn a new thread to take care of failed chan
                        thread::spawn(|| {
                            let mut worker = Worker {
                                jobs: jobs,
                                help_chan: help_chan,
                                panic_chan: panic_chan,
                            };
                            // run the worker in this thread, if it happens to panic, Worker will be unwound,
                            // and a panic message over the channel will be sent
                            worker.run()
                        });
                    }
                    StatusMsg::Shutdown => return,
                    _ => (),
                }
            }
        }
    }
}
/// implementation of ThreadPool for a SharedQueueThreadPool
impl ThreadPool for SharedQueueThreadPool {
    /// create a new NaiveThreadPool with threads available threads
    fn new(threads: i32) -> Result<Box<Self>> {
        // create taskqueue
        let jobs = Arc::new(Mutex::new(VecDeque::<StatusMsg>::new()));
        // create coord_listener for SharedQueueThreadPool
        let (tx, rx) = unbounded::<StatusMsg>();
        // create vec of handles so that when dropped, SharedQueueThreadPool takes care of all threads
        let mut handles = Vec::<JoinHandle<()>>::new();
        // iterate over num threads, and initialize the workers
        for i in 0..threads {
            // run the worker task on a separate thread
            let jobs = jobs.clone();
            let help_chan = rx.clone();
            let panic_chan = tx.clone();
            // push JoinHandle of thread so that the top level obj will keep track of threads when dropped
            handles.push(thread::spawn(|| {
                // capture cloned values
                let mut worker = Worker {
                    jobs: jobs,
                    help_chan: help_chan,
                    panic_chan: panic_chan,
                };
                // run the worker in this thread, if it happens to panic, Worker will be unwound,
                // and a panic message over the channel will be sent
                worker.run()
            }));
        }
        let clone_jobs = jobs.clone();
        // spawn a helper worker
        handles.push(thread::spawn(move || {
            // capture values
            let mut worker = Worker {
                jobs: clone_jobs,
                help_chan: rx,
                panic_chan: tx,
            };
            // this will be the helper thread to assist any failed threads
            worker.help()
        }));

        // Box the SharedQueueThreadPool, and return
        // must account for the extra help thread
        Ok(Box::from(SharedQueueThreadPool {
            jobs: jobs,
            threads: threads,
            handles: handles,
        }))
    }
    /// spawn a new task as one of the threads in the pool
    fn spawn<F>(&mut self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // lock task queue
        let mut task_queue = self.jobs.lock();
        // now we can push the newest StatusMsg into Queue
        task_queue.push_back(StatusMsg::Job(Box::from(job)))
        // Mutex will unlock once the MutexGuard goes out of scope
    }
}



impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        let mut jobs = self.jobs.lock();
        for i in 0..self.threads {
            // for all threads, send shutdown, join on all threads finishing
            jobs.push_back(StatusMsg::Shutdown);
        }
        // threads take lock now
        drop(jobs);
        // clean-up threads
        let _ = self.handles.iter().map(|handle| {
            loop {
                // loop until the thread is finished
                if handle.is_finished() {
                     break;
                }
            }
        }).collect::<()>();
        // drop Mutex
    }
}

#[cfg(test)]
mod test {
    #[test]
    // create thread panic, ensure that panic_chan, and help_chan passed to next thread
    fn test_panic_thread() {
        //
    }
}
