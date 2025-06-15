// #![no_std]
// #![no_main]

// use lib::{sync::Semaphore, *};

// extern crate lib;

// const PHILOSOPHER_COUNT: usize = 5;

// // 定义5根筷子的信号量数组
// static CHOPSTICKS: [Semaphore; PHILOSOPHER_COUNT] = semaphore_array![0, 1, 2, 3, 4];

// // 简单的线性同余随机数生成器
// struct SimpleRng {
//     state: u64,
// }

// impl SimpleRng {
//     fn new(seed: u64) -> Self {
//         Self { state: seed }
//     }
    
//     fn gene(&mut self) -> u64 {
//         // 线性同余生成器 (LCG) 参数
//         const A: u64 = 6364136223846793005;
//         const C: u64 = 1;
//         self.state = self.state.wrapping_mul(A).wrapping_add(C);
//         self.state
//     }
    
//     fn gen_range(&mut self, min: u64, max: u64) -> u64 {
//         min + self.gene() % (max - min)
//     }
// }

// fn main() -> isize {
//     // 创建哲学家线程
//     let mut pids = [0u16; PHILOSOPHER_COUNT];
//     for i in 0..PHILOSOPHER_COUNT {
//         let pid = sys_fork();
//         if pid == 0 {
//             // 子进程（哲学家）
//             let my_pid = sys_get_pid();
//             // 使用进程ID和编号生成随机种子
//             let seed = (my_pid as u64).wrapping_mul(i as u64).wrapping_add(i as u64 * 12345);
//             philosopher(i, seed);
//             sys_exit(0);
//         } else {
//             pids[i] = pid;
//         }
//     }

//     // 主线程等待所有哲学家
//     for pid in pids.iter() {
//         sys_wait_pid(*pid);
//     }

//     0
// }

// fn philosopher(id: usize, seed: u64) {
//     // 初始化信号量（每个哲学家都有自己的副本）
//     for i in 0..PHILOSOPHER_COUNT {
//         CHOPSTICKS[i].init(1); // 初始值为1（筷子可用）
//     }
    
//     let mut rng = SimpleRng::new(seed);
//     let left = id;
//     let right = (id + 1) % PHILOSOPHER_COUNT;

//     for _ in 0..5 { // 每个哲学家就餐5次
//         think(id, &mut rng);
//         eat(id, left, right, &mut rng);
//     }
// }

// fn think(id: usize, rng: &mut SimpleRng) {
//     let duration = rng.gen_range(100, 500) as usize;
//     println!("Philosopher {} is thinking for {} ticks...", id, duration);
//     delay(duration);
// }

// fn eat(id: usize, left: usize, right: usize, rng: &mut SimpleRng) {
//     // 避免死锁策略：奇数编号先拿左边，偶数编号先拿右边
//     let (first, second) = if id % 2 == 0 {
//         (left, right)
//     } else {
//         (right, left)
//     };

//     // 拿起第一根筷子
//     CHOPSTICKS[first].wait();
//     println!("Philosopher {} took chopstick {}", id, first);
    
//     // 引入随机延迟增加竞争
//     delay(rng.gen_range(10, 50) as usize);
    
//     // 拿起第二根筷子
//     CHOPSTICKS[second].wait();
//     println!("Philosopher {} took chopstick {}", id, second);

//     // 就餐
//     let eat_duration = rng.gen_range(200, 400) as usize;
//     println!("Philosopher {} is eating for {} ticks!", id, eat_duration);
//     delay(eat_duration);

//     // 放下筷子（顺序无关紧要）
//     CHOPSTICKS[left].signal();
//     CHOPSTICKS[right].signal();
//     println!("Philosopher {} put down chopsticks {} and {}", id, left, right);
// }

// fn delay(ticks: usize) {
//     for _ in 0..ticks {
//         for _ in 0..1000 {
//             core::hint::spin_loop();
//         }
//     }
// }

// entry!(main);

#![no_std]
#![no_main]
use lib::{sync::Semaphore, *};

extern crate lib;
const PHI_NUM: usize = 5;
static CHOPSTICK: [Semaphore; 5] = semaphore_array![0, 1, 2, 3, 4];
static S1: Semaphore = Semaphore::new(5);
static S2: Semaphore = Semaphore::new(6);
static mut PHI_COUNT: [i32; PHI_NUM] = [0; PHI_NUM];
fn main() -> isize {
    let help = "help: \n函数1：常规解法，会造成死锁\n函数2：要求奇数号哲学家先拿左边的筷子，然后再拿右边的筷子,而偶数号哲学家刚好相反。不存在死锁和饥饿\n函数3，要求哲学家必须按照筷子编号从小到大拿筷子,会造成不公平\n函数4：使用服务生协调，不存在死锁和饥饿";
    let stdin1 = stdin();
    println!("请选择使用的函数");
    println!("{}",help);
    let s = stdin1.read_line();
    let s = s.trim();
    match s{
        "1" => {
            println!("函数1：常规解法，会造成死锁");
        }
        "2" => {
            println!("函数2：要求奇数号哲学家先拿左边的筷子，然后再拿右边的筷子,而偶数号哲学家刚好相反。不存在死锁和饥饿");
        }
        "3" => {
            println!("函数3，要求哲学家必须按照筷子编号从小到大拿筷子,会造成不公平");
        },
        "4" => {
            println!("函数4：使用服务生协调，不存在死锁和饥饿");
        }
        _ => {
            println!("invalid input");
            println!("{}",help);
            return 0;
        }
    };
    //初始化信号量
    for i in 0..PHI_NUM{
        CHOPSTICK[i].init(1);
    }
    S1.init(1);
    S2.init(1);
    let mut pids: [u16; PHI_NUM] = [0u16; PHI_NUM];
    for i in 0..PHI_NUM{
        let pid = sys_fork();
        if pid == 0{
            if s == "1"{
                philosopher1(i);
            }else if s == "2"{
                philosopher2(i);
            }else if s == "3"{
                philosopher3(i);
            }else if s == "4"{
                philosopher4(i);
            }else{
                panic!("s should be 1, 2 or 3");
            }
            sys_exit(0);
        }
        pids[i] = pid;
    }
    let cpid = sys_get_pid();
    for i in 0..PHI_NUM {
        //println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }
    //销毁信号量
    for i in 0..PHI_NUM{
        CHOPSTICK[i].remove();
    }
    S1.remove();
    S2.remove();
    0
}
const SLEEP_TIME: i64 = 2;
// 函数1，常规解法，会造成死锁
fn philosopher1(i: usize){
    let c1 = i;
    let c2 = (i + 1) % PHI_NUM;
    for _a in 0..20{
        //thinking
        CHOPSTICK[c1].wait();
        println!("Philosopher {} get chopstick {}", i, c1);
        delay();
        CHOPSTICK[c2].wait();
        println!("Philosopher {} get chopstick {}", i, c2);
        delay();
        //eating
        println!("\x1b[32mPhilosopher {} is eating\x1b[0m", i);
        CHOPSTICK[c1].signal();
        println!("Philosopher {} release chopstick {}", i, c1);
        delay();
        CHOPSTICK[c2].signal();
        println!("Philosopher {} release chopstick {}", i, c2);
    }
}
// 函数2，要求奇数号哲学家先拿左边的筷子，然后再拿右边的筷子,而偶数号哲学家刚好相反。不存在死锁和饥饿
fn philosopher2(i: usize){
    let mut c1 = i;
    let mut c2 = (i + 1) % PHI_NUM;
    for _a in 0..20{
        //thinking
        if i % 2 == 0 {
            c1 = c1 ^ c2;
            c2 = c1 ^ c2;
            c1 = c1 ^ c2;
        }
        CHOPSTICK[c1].wait();
        println!("Philosopher {} get chopstick {}", i, c1);
        delay();
        CHOPSTICK[c2].wait();
        println!("Philosopher {} get chopstick {}", i, c2);
        delay();
        //eating
        unsafe{
            PHI_COUNT[i] += 1;
            println!("\x1b[32mPhilosopher {} is eating, he has eaten {} times.\x1b[0m", i, PHI_COUNT[i]);
        }
        CHOPSTICK[c1].signal();
        println!("Philosopher {} release chopstick {}", i, c1);
        delay();
        CHOPSTICK[c2].signal();
        println!("Philosopher {} release chopstick {}", i, c2);
    }
}

// 函数3，要求哲学家必须按照筷子编号从小到大拿筷子,会造成不公平
fn philosopher3(i: usize){
    let mut c1 = i;
    let mut c2 = (i + 1) % PHI_NUM;
    for _a in 0..100{
        //thinking
        if c1 > c2 {
            c1 = c1 ^ c2;
            c2 = c1 ^ c2;
            c1 = c1 ^ c2;
        }
        CHOPSTICK[c1].wait();
        //println!("Philosopher {} get chopstick {}", i, c1);
        delay();
        CHOPSTICK[c2].wait();
        //println!("Philosopher {} get chopstick {}", i, c2);
        delay();
        //eating
        unsafe{
            PHI_COUNT[i] += 1;
            println!("\x1b[32mPhilosopher {} is eating, he has eaten {} times.\x1b[0m", i, PHI_COUNT[i]);
        }
        
        CHOPSTICK[c1].signal();
        //println!("Philosopher {} release chopstick {}", i, c1);
        delay();
        CHOPSTICK[c2].signal();
        //println!("Philosopher {} release chopstick {}", i, c2);
    }
}

//函数4：使用服务生协调，不存在死锁和饥饿
static mut CHO_USED: [bool; 5] = [false; 5];

fn ask_server_for_cho(i: usize) -> bool{
    let c1 = i;
    let c2 = (i + 1) % PHI_NUM;
    unsafe{
        if !CHO_USED[c1] && !CHO_USED[c2] {
            CHO_USED[c1] = true;
            CHO_USED[c2] = true;
            return true;
        }else{
            return false;
        }
    }
}
fn release_cho(i: usize){
    let c1 = i;
    let c2 = (i + 1) % PHI_NUM;
    unsafe{
        CHO_USED[c1] = false;
        CHO_USED[c2] = false;
    }
}
fn philosopher4(i: usize){
    let mut c1 = i;
    let mut c2 = (i + 1) % PHI_NUM;
    for _a in 0..30{
        delay();
        //thinking
        loop{
            S1.wait();
            let ret = ask_server_for_cho(i);
            S1.signal();
            if ret{
                break;
            }
        }
        // eating
        println!("Philosopher {} get chopstick {}", i, c1);
        println!("Philosopher {} get chopstick {}", i, c2);
        unsafe{
            PHI_COUNT[i] += 1;
            println!("\x1b[32mPhilosopher {} is eating, he has eaten {} times.\x1b[0m", i, PHI_COUNT[i]);
        }
        println!("Philosopher {} release chopstick {}", i, c1);
        println!("Philosopher {} release chopstick {}", i, c2);
        S2.wait();
        release_cho(i);
        S2.signal();
        
    }
}

#[inline(never)]
// #[no_mangle]
fn delay() {
    for _ in 0..0x1 {
        core::hint::spin_loop();
    }
}


entry!(main);