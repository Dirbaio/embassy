#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::future::poll_fn;
use core::mem;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

use cortex_m::peripheral::{DCB, DWT};
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_stm32::pac;
use embassy_stm32::time::mhz;
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::task]
async fn incrementTask() -> ! {
    poll_fn(|cx| {
        cx.waker().wake_by_ref();
        COUNTER.fetch_add(1, Ordering::Relaxed);
        Poll::Pending
    })
    .await
    /*
    loop {
        COUNTER.fetch_add(1, Ordering::Relaxed);
        yield_now().await;
        // Timer::after(Duration::from_micros(0)).await;
    }
     */
}

#[embassy_executor::task]
async fn stateThread() {
    let mut dcb: DCB = unsafe { mem::transmute(()) };
    let mut dwt: DWT = unsafe { mem::transmute(()) };
    dcb.enable_trace();
    DWT::unlock();
    dwt.enable_cycle_counter();
    dwt.set_cycle_count(0);

    let mut next = Instant::from_ticks(0) + Duration::from_secs(1);
    let mut count = 0;
    Timer::at(next).await;
    loop {
        let new_count = COUNTER.load(Ordering::Relaxed);
        let cycles = DWT::cycle_count() / (new_count - count);
        info!("counter: {} ({} cycles per iteration)", new_count, cycles);
        count = new_count;
        next += Duration::from_secs(1);
        dwt.set_cycle_count(0);
        Timer::at(next).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.sys_ck = Some(mhz(50));
    config.rcc.pclk1 = Some(mhz(50));
    let _p = embassy_stm32::init(config);

    unsafe {
        pac::FLASH.acr().modify(|w| {
            w.set_dcen(true);
            w.set_icen(true);
            w.set_prften(true);
        });
    }

    info!("Hello World!");

    spawner.spawn(incrementTask()).unwrap();
    spawner.spawn(stateThread()).unwrap();
}
