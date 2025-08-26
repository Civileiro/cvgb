use futures::{
    executor::{LocalPool, LocalSpawner},
    task::LocalSpawnExt,
};

pub struct TaskManager {
    pool: LocalPool,
    spawner: LocalSpawner,
}

impl Default for TaskManager {
    fn default() -> Self {
        let pool = LocalPool::new();
        let spawner = pool.spawner();
        Self { pool, spawner }
    }
}
impl TaskManager {
    pub fn add_task<Fut>(&mut self, future: Fut)
    where
        Fut: Future<Output = ()> + 'static,
    {
        self.spawner
            .spawn_local(future)
            .expect("Executor should not be shutdown");
    }
    pub fn poll(&mut self) {
        self.pool.run_until_stalled();
    }
}
