/// Estimate the total memory usage of an object.
pub trait MemoryUsage {
    fn approx_memory_usage(&self) -> usize;
}

pub struct MemoryUsageEvaluator {}
