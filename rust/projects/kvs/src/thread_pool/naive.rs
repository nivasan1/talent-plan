use crate::thread_pool::*;
use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle, Thread};

/// This is the standard worker type, it takes a
pub struct Worker {
    // the handle to this thread
    thread: JoinHandle<Thread>,
    // transmitter for tasks]
    workChan: Sender<(Task, Sender<()>)>,
}

pub struct NaiveThreadPool {
    recvChan: Vec<Receiver<()>>,
    // all of these threads are parked, waiting for a task to be sent over the workChan
    availThreads: Vec<Worker>,
    workingThreads: Vec<Worker>,
}

/// Naive implementation of a thread pool
impl ThreadPool for NaiveThreadPool {
    /// create a new NaiveThreadPool with threads available threads
    fn new(threads: i32) -> Result<Box<Self>> {
        // instantiate size workers, and append them to availThreads
        let mut workers = Vec::new();
        // vector of sending ends of channel
        for _ in 0..threads {
            // push Workers onto vec
            let (tx, rx) = channel::<(Task, Sender<()>)>();
            workers.push(Worker {
                thread: thread::spawn(move || {
                    // iterate continuously, until the rx is closed
                    loop {
                        // receive task on rx
                        let (task, tx) = rx.recv().unwrap();
                        // execute task
                        task();
                        // once done send on tx chan
                        tx.send(()).unwrap();
                        // park this thread, and wait for master to unpark it
                        thread::park();
                        // drop tx
                        drop(tx);
                    }
                }),
                workChan: tx,
            });
        }

        Ok(Box::from(NaiveThreadPool {
            availThreads: workers,
            workingThreads: Vec::new(),
            recvChan: Vec::new(),
        }))
    }
    /// spawn a new task as one of the threads in the pool
    fn spawn<F>(&mut self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // pop working vector off of the stack
        let mut is_worker = self.availThreads.pop();
        // no workers are available
        if let None = is_worker {
            self.clear(false).unwrap();
            is_worker = self.availThreads.pop();
        }
        let worker = is_worker.unwrap();
        // send task, tx over channel
        let (tx, rx) = channel();
        worker.workChan.send((Box::from(job), tx)).unwrap();
        // push worker into workingThreads
        self.workingThreads.push(worker);
        // push recv chan onto availThreads
        self.recvChan.push(rx);
    }
}

impl NaiveThreadPool {
    /// clear iterates over the workingThreads, and their receive channels,
    /// if try_recv succeeds, then the thread has finished, and it is ready for more work
    fn clear(&mut self, fully_clear: bool) -> Result<()> {
        let mut continue_loop = true;
        let mut removed = 0;
        loop {
            for i in 0..self.workingThreads.len() {
                let idx = i - removed;
                match self.recvChan[idx].try_recv() {
                    Ok(()) => {
                        // we were able to receive on this channel, pop it from working theads, and pop recvChan
                        let thread = self.workingThreads.remove(idx);
                        self.recvChan.remove(idx);
                        // push this thread onto availThreads
                        thread.thread.thread().unpark();
                        self.availThreads.push(thread);
                        // we may break from the loop after this iteration
                        continue_loop = false;
                        removed += 1;
                    }
                    Err(err) => {
                        // if there was an error in try recv, simply continue
                        if let TryRecvError::Empty = err {
                            continue;
                        }
                        // if there was an error in the underlying channel fail
                        return Err(Box::from(err));
                    }
                }
            }
            // it is ok to break from this loop, let other threads finish, unless we are forced to explicitly wait
            // i.e in Join
            if !continue_loop && !fully_clear {
                return Ok(());
            }
            // if we've cleared the recvChan exit loop
            if self.recvChan.len() == 0 {
                return Ok(());
            }
        }
    }
}
