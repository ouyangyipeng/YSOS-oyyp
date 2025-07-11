use super::ProcessId;
use alloc::collections::*;         // 内核内存分配下的集合类型
use spin::Mutex;                    // 自旋锁保护共享数据

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SemaphoreId(u32);        // 内核信号量标识符（包装u32键值）

impl SemaphoreId {
    pub fn new(key: u32) -> Self {  // 构造函数
        Self(key)
    }
}

/// Mutex is required for Semaphore
#[derive(Debug, Clone)]
pub struct Semaphore {
    count: usize,                   // 当前可用资源数（文档中的信号量值）
    wait_queue: VecDeque<ProcessId>, // 等待队列（FIFO调度）
}

/// Semaphore result
#[derive(Debug)]
pub enum SemaphoreResult {          // 操作结果枚举（文档中的唤醒/阻塞机制）
    Ok,                             // 操作成功
    NotExist,                       // 信号量不存在
    Block(ProcessId),               // 需要阻塞指定进程
    WakeUp(ProcessId),              // 需要唤醒指定进程
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(value: usize) -> Self { // 构造函数
        Self {
            count: value,           // 初始资源数
            wait_queue: VecDeque::new(), // 空等待队列
        }
    }

    /// Wait the semaphore (acquire/down/proberen)
    ///
    /// if the count is 0, then push the process into the wait queue
    /// else decrease the count and return Ok
    pub fn wait(&mut self, pid: ProcessId) -> SemaphoreResult {
        // FIXME: if the count is 0, then push pid into the wait queue
        //          return Block(pid)
        if self.count == 0 {         // 资源不足
            self.wait_queue.push_back(pid); // 加入等待队列
            SemaphoreResult::Block(pid) // 通知调度器阻塞该进程
        } else {
        // FIXME: else decrease the count and return Ok
            self.count -= 1;        // 消耗资源
            SemaphoreResult::Ok
        }
    }

    /// Signal the semaphore (release/up/verhogen)
    ///
    /// if the wait queue is not empty, then pop a process from the wait queue
    /// else increase the count
    pub fn signal(&mut self) -> SemaphoreResult {
        // FIXME: if the wait queue is not empty
        //          pop a process from the wait queue
        //          return WakeUp(pid)
        if let Some(pid) = self.wait_queue.pop_front() { // 有等待进程
            SemaphoreResult::WakeUp(pid) // 通知唤醒队首进程
        } else {
        // FIXME: else increase the count and return Ok
            self.count += 1;        // 无等待者则增加资源
            SemaphoreResult::Ok
        }
    }
}

#[derive(Debug, Default)]
pub struct SemaphoreSet {
    sems: BTreeMap<SemaphoreId, Mutex<Semaphore>>,
}

impl SemaphoreSet {
    // pub fn insert(&mut self, key: u32, value: usize) -> bool {
    //     trace!("Sem Insert: <{:#x}>{}", key, value);

    //     // FIXME: insert a new semaphore into the sems
    //     //          use `insert(/* ... */).is_none()`
    //     let sid = SemaphoreId::new(key);
    //     let val = Mutex::new(Semaphore::new(value));
    //     self.sems.insert(sid, val).is_none() // 插入新信号量
    // }
    pub fn insert(&mut self, key: u32, value: usize) -> bool {
        // info!("Sem Insert: <{:#x}>{}", key, value);

        // FIXME: insert a new semaphore into the sems
        //          use `insert(/* ... */).is_none()`
        let ret = self.sems.insert(SemaphoreId::new(key),Mutex::new(Semaphore::new(value))).is_none();
        // info!("Semaphore Insert Result: {}", ret); // 输出插入结果
        ret

    }

    pub fn remove(&mut self, key: u32) -> bool {
        trace!("Sem Remove: <{:#x}>", key);

        // FIXME: remove the semaphore from the sems
        //          use `remove(/* ... */).is_some()`
        let sid = SemaphoreId::new(key);
        self.sems.remove(&sid).is_some() // 删除现有信号量
    }

    /// Wait the semaphore (acquire/down/proberen)
    pub fn wait(&self, key: u32, pid: ProcessId) -> SemaphoreResult {
        let sid = SemaphoreId::new(key);

        // FIXME: try get the semaphore from the sems
        //         then do it's operation
        // FIXME: return NotExist if the semaphore is not exist
        match self.sems.get(&sid) {  // 查找信号量
            Some(sem) => {
                let mut guard = sem.lock(); // 自旋锁保护操作
                guard.wait(pid)       // 执行等待逻辑
            },
            None => SemaphoreResult::NotExist,
        }
    }

    /// Signal the semaphore (release/up/verhogen)
    pub fn signal(&self, key: u32) -> SemaphoreResult {
        let sid = SemaphoreId::new(key);

        // FIXME: try get the semaphore from the sems
        //         then do it's operation
        // FIXME: return NotExist if the semaphore is not exist
        match self.sems.get(&sid) {
            Some(sem) => {
                let mut guard = sem.lock();
                guard.signal()       // 执行释放逻辑
            },
            None => SemaphoreResult::NotExist,
        }
    }
}

impl core::fmt::Display for Semaphore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Semaphore({}) {:?}", self.count, self.wait_queue)
    }
}

/*
关键机制解析（结合文档）：

​​信号量存储结构​​
    SemaphoreSet 使用 BTreeMap 管理所有信号量，键为 SemaphoreId（对应用户层的 key）
    每个信号量用 Mutex 包裹，确保多核操作安全（文档中的并发锁机制章节）
​​阻塞与唤醒流程​​
    wait 操作返回 Block(pid) 时，内核调用 ProcessManager::block(pid)（文档的进程管理章节）
    signal 操作返回 WakeUp(pid) 时，内核调用 wake_up(pid) 将进程加入就绪队列
​​与系统调用的对接​​
    用户调用 sem_wait/signal 触发 sys_sem 系统调用（文档中 Syscall::Sem 的处理）
    内核通过 SemaphoreSet::wait/signal 更新状态并返回调度决策
*/
