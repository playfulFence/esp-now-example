#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::_export::StaticCell;
use embassy_futures::select::{select, Either};
#[cfg(feature = "esp32")]
use esp32_hal as hal;
#[cfg(feature = "esp32c2")]
use esp32c2_hal as hal;
#[cfg(feature = "esp32c3")]
use esp32c3_hal as hal;
#[cfg(feature = "esp32s2")]
use esp32s2_hal as hal;
#[cfg(feature = "esp32s3")]
use esp32s3_hal as hal;

use embassy_executor::Executor;
use embassy_time::{Duration, Ticker};
use esp_backtrace as _;
use esp_println::logger::init_logger;
use esp_println::println;
use esp_wifi::esp_now::{EspNow, PeerInfo, BROADCAST_ADDRESS};
use esp_wifi::initialize;
use futures_util::StreamExt;
use hal::clock::{ClockControl, CpuClock};
use hal::Rng;
use hal::i2c::I2C;
use hal::IO;
use hal::{embassy, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc};
use core::{str, mem};
use libm::fabsf;


use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
use shared_bus::{BusManagerSimple, NullMutex, I2cProxy, BusManager};

#[cfg(any(feature = "esp32c3", feature = "esp32c2"))]
use hal::system::SystemExt;

#[cfg(any(feature = "esp32c3", feature = "esp32c2"))]
use riscv_rt::entry;
#[cfg(any(feature = "esp32", feature = "esp32s3", feature = "esp32s2"))]
use xtensa_lx_rt::entry;

#[embassy_executor::task]
async fn run(mut esp_now: EspNow, mut _icm : Icm42670<I2cProxy<'static, NullMutex<I2C<'static, hal::peripherals::I2C0, >>>>) {
    let mut ticker = Ticker::every(Duration::from_secs(3));
    loop { 
        let temp = _icm.temperature().unwrap();

        println!("Sending temp {:x?}", temp);

        // Convert f32 to u32
        let bits: u32 = temp.to_bits();

        // Convert u32 to [u8; 4]
        let b1: u8 = (bits >> 24) as u8;
        let b2: u8 = (bits >> 16) as u8;
        let b3: u8 = (bits >> 8) as u8;
        let b4: u8 = bits as u8;
        let bytes: [u8; 4] = [b1, b2, b3, b4];

        println!("Sending temp (bytes) {:x?}", bytes);
        
        let res = select(ticker.next(), async {
            let r = esp_now.receive_async().await;

            // Use this code to receive the same data

            // let rec_bytes = r.get_data();
            // let bits = ((rec_bytes[0] as u32) << 24)
            // | ((rec_bytes[1] as u32) << 16)
            // | ((rec_bytes[2] as u32) << 8)
            // | 0;

            // println!("Recieved {:x?}", f32::from_bits(bits));

            if r.info.dst_address == BROADCAST_ADDRESS {
                if !esp_now.peer_exists(&r.info.src_address).unwrap() {
                    esp_now
                        .add_peer(PeerInfo {
                            peer_address: r.info.src_address,
                            lmk: None,
                            channel: None,
                            encrypt: false,
                        })
                        .unwrap();
                }
                #[cfg(any(feature = "esp32c3", feature = "esp32c2"))]
                esp_now.send(&r.info.src_address, &bytes).unwrap();
            }
        })
        .await;

        match res {
            Either::First(_) => {
        
            }
            Either::Second(_) => (),
        }
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();


#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);
    esp_wifi::init_heap();

    let peripherals = Peripherals::take();

    #[cfg(not(feature = "esp32"))]
    let mut system = peripherals.SYSTEM.split();
    #[cfg(feature = "esp32")]
    let system = peripherals.DPORT.split();

    #[cfg(feature = "esp32c3")]
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();
    #[cfg(feature = "esp32c2")]
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock120MHz).freeze();
    #[cfg(any(feature = "esp32", feature = "esp32s3", feature = "esp32s2"))]
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);

    // Disable watchdog timers
    #[cfg(not(any(feature = "esp32", feature = "esp32s2")))]
    rtc.swd.disable();

    rtc.rwdt.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // #[cfg(any(feature = "esp32c3", feature = "esp32c2"))]
    // {
        use hal::systimer::SystemTimer;
        let syst = SystemTimer::new(peripherals.SYSTIMER);
        initialize(syst.alarm0, Rng::new(peripherals.RNG), &clocks).unwrap();
 
        unsafe{
            let i2c = I2C::new(
                peripherals.I2C0,
                io.pins.gpio10,
                io.pins.gpio8,
                100u32.kHz(),
                &mut system.peripheral_clock_control,
                &clocks,
            );

            let bus = BusManagerSimple::new(i2c);
            let mut icm = Icm42670::new(core::mem::transmute(bus.acquire_i2c()), Address::Primary).unwrap();
        
    //}
    // #[cfg(any(feature = "esp32", feature = "esp32s3", feature = "esp32s2"))]
    // {
    //     use hal::timer::TimerGroup;
    //     let timg1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    //     initialize(timg1.timer0, Rng::new(peripherals.RNG), &clocks).unwrap();
    // }

    let esp_now = esp_wifi::esp_now::esp_now().initialize().unwrap();
    println!("esp-now version {}", esp_now.get_version().unwrap());
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0.timer0);
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run(esp_now, icm)).ok();
    });
}
}

