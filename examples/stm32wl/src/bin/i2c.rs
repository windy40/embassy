#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::I2c;
use embassy_stm32::interrupt;
use embassy_stm32::time::Hertz;
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 0x5F;
const WHOAMI: u8 = 0x0F;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let irq = interrupt::take!(I2C2_EV);
    let mut i2c = I2c::new(
        p.I2C2,
        p.PA12,
        p.PA11,
        irq,
        NoDma,
        NoDma,
        Hertz(100_000),
        Default::default(),
    );

    let mut data = [0u8; 1];
    unwrap!(i2c.blocking_write_read(ADDRESS, &[WHOAMI], &mut data));
    info!("Whoami: {}", data[0]);
}
