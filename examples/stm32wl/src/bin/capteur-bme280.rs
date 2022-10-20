#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
//use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::I2c;
use embassy_stm32::interrupt;
use embassy_stm32::time::Hertz;

//use embedded_hal_async::i2c::I2c as I2cTrait;

use {defmt_rtt as _, panic_probe as _};

use bme280;
use embassy_time::Delay;
use embedded_hal::blocking::delay::DelayUs;


#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let irq = interrupt::take!(I2C2_EV);
    let i2c = I2c::new(
        p.I2C2,
        p.PA12,
        p.PA11,
        irq,
        NoDma,
        NoDma,
        Hertz(100_000),
        Default::default(),
    );
 //   let mut i2c = BlockingAsync::new(i2c);

    let mut bme280 = bme280::i2c::BME280::new_primary(i2c);

    // initialize the sensor
    bme280.init(&mut Delay).unwrap();

    // measure temperature, pressure, and humidity
    let measurements = bme280.measure(&mut Delay).unwrap();

    println!("Relative Humidity = {}%", measurements.humidity);
    println!("Temperature = {} deg C", measurements.temperature);
    println!("Pressure = {} pascals", measurements.pressure);

    loop{
        Delay.delay_us(1000000u32);
        let measurements = bme280.measure(&mut Delay).unwrap();

        println!("Relative Humidity = {}%", measurements.humidity);
        println!("Temperature = {} deg C", measurements.temperature);
        println!("Pressure = {} pascals", measurements.pressure);
    

    }
}
