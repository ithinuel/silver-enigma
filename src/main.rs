#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::{executor::Executor, util::Forever};
use panic_probe as _;

use embedded_hal::digital::v2::ToggleableOutputPin;

use rp2040_hal::{
    clocks::init_clocks_and_plls,
    gpio::{self, bank0, Pin, PushPullOutput},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

static EXECUTOR: Forever<Executor> = Forever::new();

async fn _yield() {
    let mut once = false;
    futures::future::poll_fn(move |cx| {
        cx.waker().wake_by_ref();
        if !once {
            once = true;
            core::task::Poll::Pending
        } else {
            core::task::Poll::Ready(())
        }
    })
    .await
}

#[embassy::task]
async fn one(mut pin: Pin<bank0::Gpio20, PushPullOutput>) {
    loop {
        _yield().await;
        let _ = pin.toggle();
    }
}
#[embassy::task]
async fn two(mut pin: Pin<bank0::Gpio22, PushPullOutput>) {
    loop {
        _yield().await;
        _yield().await;
        let _ = pin.toggle();
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let _clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let pin20 = pins.gpio20.into_mode();
    let pin22 = pins.gpio22.into_mode();

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        spawner.spawn(one(pin20)).unwrap();
        spawner.spawn(two(pin22)).unwrap();
    });
}
