use crate::thread_pool::*;

pub struct NaiveThreadPool {

}

/// Naive implementation of a thread pool
impl ThreadPool for NaiveThreadPool {
    /// create a new NaiveThreadPool with threads available threads
    fn new(threads: i32) -> Result<Box<Self>>{
        todo!()
    }
    /// spawn a new task as one of the threads in the pool
    fn spawn<F>(&mut self, job: F) where F: FnOnce() + Send + 'static {
        todo!()
    }
}