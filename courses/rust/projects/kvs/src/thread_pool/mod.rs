use crate::engines::kvs_engine::Result;

pub type Task = Box<dyn FnOnce() + Send + 'static>;

pub trait ThreadPool {
    /// create i threads in this thread pool, panic if the number of active threads
    /// is above num_cpu threads
    fn new(threads: i32) -> Result<Box<Self>>;
    /// execute a task on the most recently available thread in the thread pool
    fn spawn<F>(&mut self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

pub mod naive;

pub mod rayon;

pub mod shared_queue;
