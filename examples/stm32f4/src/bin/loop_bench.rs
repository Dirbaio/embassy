#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::{AtomicU32, Ordering};
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_stm32::time::mhz;
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::task]
async fn incrementTask() {
    loop {
        COUNTER.fetch_add(1, Ordering::Relaxed);
        yield_now().await;
        // Timer::after(Duration::from_micros(0)).await;
    }
}

#[embassy_executor::task]
async fn stateThread() {
    let mut next = Instant::from_ticks(0) + Duration::from_secs(1);
    let mut count = 0;
    Timer::at(next).await;
    loop {
        let new_count = COUNTER.load(Ordering::Relaxed);
        let cycles = 100_000_000 / (new_count - count);
        info!("counter: {} ({} cycles per iteration)", new_count, cycles);
        count = new_count;
        next += Duration::from_secs(1);
        Timer::at(next).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.sys_ck = Some(mhz(100));
    config.rcc.pclk1 = Some(mhz(50));
    let _p = embassy_stm32::init(config);
    info!("Hello World!");

    spawner.spawn(incrementTask()).unwrap();
    spawner.spawn(stateThread()).unwrap();
}
