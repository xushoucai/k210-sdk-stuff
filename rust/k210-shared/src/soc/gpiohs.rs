use k210_hal::pac;

use crate::soc::gpio;
use crate::soc::utils::set_bit;

pub fn set_direction(pin: u8, direction: gpio::direction) {
    unsafe {
        let ptr = pac::GPIOHS::ptr();
        (*ptr)
            .output_en
            .modify(|r, w| w.bits(set_bit(r.bits(), pin, direction == gpio::direction::OUTPUT)));
        (*ptr)
            .input_en
            .modify(|r, w| w.bits(set_bit(r.bits(), pin, direction == gpio::direction::INPUT)));
    }
}

pub fn set_pin(pin: u8, value: bool) {
    unsafe {
        let ptr = pac::GPIOHS::ptr();
        (*ptr)
            .output_val
            .modify(|r, w| w.bits(set_bit(r.bits(), pin, value)));
    }
}
