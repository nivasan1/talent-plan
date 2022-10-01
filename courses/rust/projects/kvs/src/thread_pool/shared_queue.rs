use crate::thread_pool::*;

pub struct SharedQueueThreadPool {

}

/// implementation of ThreadPool for a SharedQueueThreadPool 
impl ThreadPool for SharedQueueThreadPool {
    /// create a new NaiveThreadPool with threads available threads
    fn new(threads: i32) -> Result<Box<Self>>{
        todo!()
    }
    /// spawn a new task as one of the threads in the pool
    fn spawn<F>(&mut self, job: F) where F: FnOnce() + Send + 'static {
        todo!()
    }
}