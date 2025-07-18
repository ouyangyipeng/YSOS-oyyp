#![no_std]
#![no_main]
const MOD: u64 = 1000000007;
extern crate lib;
use lib::*;

fn factorial(n: u64) -> u64 {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1) % MOD
    }
}

fn main() -> isize {
    print!("Input n: ");

    let input = lib::stdin().read_line();

    // prase input as u64
    let n = input.parse::<u64>().unwrap();
    // let n= 999999;

    if n > 1000000 {
        println!("n must be less than 1000000");
        return 1;
    }

    // calculate factorial
    let result = factorial(n);

    // print system status
    sys_stat();

    // print result
    println!("The factorial of {} under modulo {} is {}.", n, MOD, result);

    0
}

entry!(main);