#![no_main]
#![no_std]

extern crate panic_semihosting;  // 4004 bytes
// extern crate panic_halt; // 672 bytes

// #[macro_use(block)]
// extern crate nb;

use cortex_m_rt::entry;
use cortex_m_semihosting::heprintln;

use lpc55_hal as hal;
use hal::prelude::*;
use hal::{
    drivers::{
        Pins,
    },
    peripherals::{
        pint::{
            Mode,
        },
    },
};



#[entry]
fn main() -> ! {

    heprintln!("External interrupts").unwrap();

    let mut hal = hal::new();

    let _clocks = hal::ClockRequirements::default()
        .system_frequency(12.mhz())
        .configure(&mut hal.anactrl, &mut hal.pmc, &mut hal.syscon)
        .unwrap();

    let mut gpio = hal.gpio.enabled(&mut hal.syscon);
    let mut iocon = hal.iocon.enabled(&mut hal.syscon);
    let pins = Pins::take().unwrap();

    // // NFC IRQ pin for Solo-bee
    let input = pins.pio0_0.into_gpio_pin(&mut iocon, &mut gpio).into_input();

    let mut mux = hal.inputmux.enabled(&mut hal.syscon);
    let mut pint = hal.pint.enabled(&mut hal.syscon);

    pint.enable_interrupt(&mut mux, &input, 0, Mode::RisingEdge);
    pint.enable_interrupt(&mut mux, &input, 0, Mode::FallingEdge);

    // // Dont need mux anymore
    mux.disabled(&mut hal.syscon);

    let pint = pint.release();

    // Clear interrupts initially
    pint.rise.write(|w| unsafe { w.bits(1) });
    pint.fall.write(|w| unsafe { w.bits(1) });

    loop {

        if (pint.rise.read().bits() & 1) != 0 {
            pint.rise.write(|w| unsafe { w.bits(1) });
            heprintln!("Rising edge detected").unwrap();
        }

        if (pint.fall.read().bits() & 1) != 0 {
            pint.fall.write(|w| unsafe { w.bits(1) });
            heprintln!("Falling edge detected").unwrap();
        }

    }
}