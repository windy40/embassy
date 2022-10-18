#![no_std]
#![no_main]
#![macro_use]
#![allow(dead_code)]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_lora::stm32wl::*;
use embassy_lora::LoraTimer;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Speed};
use embassy_stm32::rng::Rng;
use embassy_stm32::subghz::*;
use embassy_stm32::{interrupt, pac};
use lorawan::default_crypto::DefaultFactory as Crypto;
use lorawan_device::async_device::{region, Device, JoinMode};
use {defmt_rtt as _, panic_probe as _};

struct RadioSwitch<'a> {
    ctrl1: Output<'a, AnyPin>,
    ctrl2: Output<'a, AnyPin>,
    ctrl3: Output<'a, AnyPin>,
}

impl<'a> RadioSwitch<'a> {
    fn new(ctrl1: Output<'a, AnyPin>, ctrl2: Output<'a, AnyPin>, ctrl3: Output<'a, AnyPin>) -> Self {
        Self { ctrl1, ctrl2, ctrl3 }
    }
}

impl<'a> embassy_lora::stm32wl::RadioSwitch for RadioSwitch<'a> {
    fn set_rx(&mut self) {
        self.ctrl1.set_high();
        self.ctrl2.set_low();
        self.ctrl3.set_high();
    }

    fn set_tx(&mut self) {
        self.ctrl1.set_high();
        self.ctrl2.set_high();
        self.ctrl3.set_high();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.mux = embassy_stm32::rcc::ClockSrc::HSI16;
    config.rcc.enable_lsi = true;
    let p = embassy_stm32::init(config);

    unsafe { pac::RCC.ccipr().modify(|w| w.set_rngsel(0b01)) }

    let ctrl1 = Output::new(p.PC3.degrade(), Level::High, Speed::High);
    let ctrl2 = Output::new(p.PC4.degrade(), Level::High, Speed::High);
    let ctrl3 = Output::new(p.PC5.degrade(), Level::High, Speed::High);
    let rfs = RadioSwitch::new(ctrl1, ctrl2, ctrl3);

    let radio = SubGhz::new(p.SUBGHZSPI, NoDma, NoDma);
    let irq = interrupt::take!(SUBGHZ_RADIO);

    let mut radio_config = SubGhzRadioConfig::default();
    radio_config.calibrate_image = CalibrateImage::ISM_863_870;
    let radio = SubGhzRadio::new(radio, rfs, irq, radio_config).unwrap();

    let mut region: region::Configuration = region::EU868::default().into();

    // NOTE: This is specific for TTN, as they have a special RX1 delay
    //region.set_receive_delay1(5000);
    region.set_receive_delay1(1000);

    let mut device: Device<_, Crypto, _, _> = Device::new(region, radio, LoraTimer::new(), Rng::new(p.RNG));

    // Depending on network, this might be part of JOIN
    device.set_datarate(region::DR::_0); // SF12

    // device.set_datarate(region::DR::_1); // SF11
    // device.set_datarate(region::DR::_2); // SF10
    // device.set_datarate(region::DR::_3); // SF9
    // device.set_datarate(region::DR::_4); // SF8
    // device.set_datarate(region::DR::_5); // SF7

    defmt::info!("Joining LoRaWAN network");

    // TODO: Adjust the EUI and Keys according to your network credentials
    device
        .join(&JoinMode::OTAA {
 //           deveui: [0x00,0x80,0xe1,0x15,0x00,0x0a,0xe3,0x2e],
            deveui: [0x2e,0xe3,0x0a,0x00,0x15,0xe1,0x80,0x00],
            appeui: [0, 0, 0, 0, 0, 0, 0, 0],
            appkey: [0x2c,0xc1,0x72,0x96,0x9d,0x5c,0xc2,0x63,0x82,0xe0,0xad,0x05,0x45,0x68,0xce,0x3e],
 //           appkey: [0x3e,0xce,0x68,0x45,0x05,0xad,0xe0,0x82,0x63,0xc2,0x5c,0x9d,0x96,0x72,0xc1,0x2c],
        })
        .await
        .ok()
        .unwrap();
    defmt::info!("LoRaWAN network joined");

    let mut rx: [u8; 255] = [0; 255];
    defmt::info!("Sending 'PING'");
    let len = device.send_recv(b"PING", &mut rx[..], 1, true).await.ok().unwrap();
    if len > 0 {
        defmt::info!("Message sent, received downlink: {:?}", &rx[..len]);
        defmt::info!("  as str: {:?}", core::str::from_utf8(&rx[..len]).unwrap());
    } else {
        defmt::info!("Message sent!");
    }
}
