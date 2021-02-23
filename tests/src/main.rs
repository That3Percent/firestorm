use firestorm::{bench, profile_fn};
use std::time::Duration;

fn sleep(secs: u64) {
    profile_fn!(sleep);
    std::thread::sleep(Duration::from_secs(secs))
}

fn own_3_twice_call() {
    profile_fn!(own_3_twice_call);

    call();
    sleep(2);
    std::thread::sleep(Duration::from_secs(1));
    call();
}

fn call() {
    profile_fn!(call);
    sleep(1);
}
// TODO: Time-Axis is wrong

fn main() {
    bench("./", own_3_twice_call).unwrap();
}
