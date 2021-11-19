use lunatic::{process, Mailbox};
use serde::{de::DeserializeOwned, Serialize};

#[lunatic::main]
fn main(mb: Mailbox<i32>) {
    println!("Hi!");

    let sum_process = process::spawn(sum_generic).unwrap();
    let this = process::this(&mb);

    sum_process.send((this, (1, 2)));

    let result = mb.receive().unwrap();

    println!("Result: {}", result);
}

#[moonbeam_macros::process]
fn sum(l: i32, r: i32) -> i32 {
    l + r
}

#[moonbeam_macros::process]
fn sum_generic<T>(l: T, r: T) -> T
where
    T: std::ops::Add<Output = T> + Serialize + DeserializeOwned,
{
    l + r
}
