#![cfg(feature = "compat")]
#![cfg(not(miri))] // Miri does not support epoll

use futures::{
    compat::Future01CompatExt,
    prelude::*,
};
use std::time::Instant;
use tokio::{
    runtime::Runtime,
    timer::Delay,
};

#[test]
fn can_use_01_futures_in_a_03_future_running_on_a_01_executor() {
    let f = async { Delay::new(Instant::now()).compat().await };

    let mut runtime = Runtime::new().unwrap();
    runtime.block_on(f.boxed().compat()).unwrap();
}
