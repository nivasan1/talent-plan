use crate::thread_pool::*;
use ::rayon::ThreadPool as Pool;
use ::rayon::ThreadPoolBuilder;
/// implementation of a rayon thread pool
pub struct RayonThreadPool {
    thread_pool: Pool,
}

/// implementation of ThreadPool for a SharedQueueThreadPool
impl ThreadPool for RayonThreadPool {
    /// create a new NaiveThreadPool with threads available threads
    fn new(threads: i32) -> Result<Box<Self>> {
        // build a new rayon thread pool
        Ok(Box::from(RayonThreadPool {
            thread_pool: ThreadPoolBuilder::new()
                .num_threads(threads as usize)
                .build()?,
        }))
    }
    /// spawn a new task as one of the threads in the pool
    fn spawn<F>(&mut self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.thread_pool.install(job)
    }
}
