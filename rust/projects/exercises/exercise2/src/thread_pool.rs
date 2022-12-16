use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle, Thread};

pub type Task = Box<dyn FnOnce() + Send + 'static>;
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// This is the standard worker type, it takes a
pub struct Worker {
    // the handle to this thread
    thread: JoinHandle<Thread>,
    // transmitter for tasks]
    workChan: Sender<(Task, Sender<()>)>,
}

/// thread pool is a wrapper around a group of pre-initialized uninitialized threads that can be multiplexed among several
/// tasks, it maintains a list of threads in use, and a list of threads that are no longer in use, each object in the pool
/// takes both a task  <T,O> fn: O -> T
pub struct ThreadPool {
    recvChan: Vec<Receiver<()>>,
    // all of these threads are parked, waiting for a task to be sent over the workChan
    availThreads: Vec<Worker>,
    workingThreads: Vec<Worker>,
}

/// methods, beginTask, endTask, and initialize,
impl ThreadPool {
    /// create num_cpu threads for active use
    pub fn new(size: u64) -> Result<Self> {
        // instantiate size workers, and append them to availThreads
        let mut workers = Vec::new();
        // vector of sending ends of channel
        for _ in 0..size {
            // push Workers onto vec
            let (tx, rx) = channel::<(Task, Sender<()>)>();
            workers.push(Worker {
                // pass reference of receive chan to the 
                thread: thread::spawn(move || {
                    // iterate continuously, until the rx is closed
                    loop {
                        // receive task on rx
                        let (task, tx)  = rx.recv().unwrap();
                        // execute task
                        task();
                        // once done send on tx chan
                        tx.send(()).unwrap();
                        // park this thread, and wait for master to unpark it
                        thread::park();
                    }
                }),
                workChan: tx,
            });
        }

        Ok(ThreadPool {
            availThreads: workers,
            workingThreads: Vec::new(),
            recvChan: Vec::new(),
        })
    }

    ///execute a function
    pub fn execute(&mut self, job: Task) -> Result<()> {
        // pop working vector off of the stack
        let is_worker = self.availThreads.pop();
        // no workers are available
        if let None = is_worker {
            self.clear(false)?;
        }
        let worker = is_worker.unwrap();
        // send task, tx over channel   
        let (tx, rx) = channel();
        worker.workChan.send((job, tx))?;
        // push worker into workingThreads
        self.workingThreads.push(worker);
        // push recv chan onto availThreads
        self.recvChan.push(rx);
        Ok(())
    }   
    
    /// clear iterates over the workingThreads, and their receive channels, 
    /// if try_recv succeeds, then the thread has finished, and it is ready for more work
    fn clear(&mut self, fully_clear: bool) -> Result<()> {
        let mut continue_loop = true;
        loop {
            for i in 0..self.workingThreads.len() {
                match self.recvChan[i].try_recv() {
                    Ok(()) => {
                        // we were able to receive on this channel, pop it from working theads, and pop recvChan
                        let thread = self.workingThreads.remove(i);
                        self.recvChan.remove(i);
                        // push this thread onto availThreads
                        thread.thread.thread().unpark();
                        self.availThreads.push(thread);
                        // we may break from the loop after this iteration
                        continue_loop = false;
                    },
                    Err(err) => {
                        // if there was an error in try recv, simply continue
                        if let TryRecvError::Empty  = err {
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

    /// join all working threads for this thread pool,
    /// It iterates through all available threads, preferring looping
    /// calls to is_finished, as opposed to joining and blocking individually
    /// on each thread
    pub fn join(&mut self) -> Result<()> {
        // clear threads, 
        self.clear(true)?;
        Ok(())
    }
}


// tests
#[cfg(test)]
mod test {
    use super::ThreadPool;
    use std::sync::{Arc, Mutex};
    use std::{thread, time};

    #[test]
    fn test_single_thread() {
        let counter = Arc::new(Mutex::new(0));
        let mut pool = ThreadPool::new(2).unwrap();
        // execute a thread with a simple mutation 
        {
            let counter = Arc::clone(&counter);
            pool.execute(Box::from(move || {
                // unlock counter
                let mut j = counter.lock().unwrap();
                *j += 1;
            })).unwrap();
        }
        // wait for pool to finish
        pool.join().unwrap();
        let i = counter.lock().unwrap();
        assert_eq!(*i, 1);
    }
    #[test]
    fn test_execution_with_thread_saturation() {
        let counter = Arc::new(Mutex::new(0));
        let mut pool = ThreadPool::new(100).unwrap();
        // execute a thread with a simple mutation 
        for _ in 0..1000 {
            let counter = Arc::clone(&counter);
            pool.execute(Box::from(move || {
                // unlock counter
                let mut j = counter.lock().unwrap();
                *j += 1;
            })).unwrap();
        }
        // wait for pool to finish
        pool.join().unwrap();
        let i = counter.lock().unwrap();
        assert_eq!(*i, 1);
    }
}