#![no_std]
#![no_main]
#![feature(min_const_generics)]

use arduino_uno::prelude::*;
use encode_unicode::IterExt as _;

extern "C" {
    pub fn dtostrf(val: f32, width: i8, prec: u8, buffer: *mut u8) -> *const u8; // char * dtostrf (double val, signed char width, unsigned char prec, char *sout)
}

struct Buffer<const S: usize>([u8; S]);
impl<const S: usize> Buffer<S> {
    pub fn new() -> Self {
        Self([0; S])
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }
}
impl<const S: usize> ufmt::uDisplay for Buffer<S> {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        self.0.iter().to_utf8chars().for_each(|ch| {
            f.write_char(ch.unwrap().to_char()).map_err(|_| ()).unwrap();
        });
        Ok(())
    }
}

// https://stackoverflow.com/a/63201905/1439153
fn int32_to_float_rz(a: i32) -> f32 {
    let mut i = a as u32;
    let mut shift = 0u32;
    // take absolute value of integer
    if a < 0 {
        i = 0 - i;
    }
    // normalize integer so MSB is set
    if !(i > 0x0000ffff) {
        i <<= 16;
        shift += 16;
    }
    if !(i > 0x00ffffff) {
        i <<= 8;
        shift += 8;
    }
    if !(i > 0x0fffffff) {
        i <<= 4;
        shift += 4;
    }
    if !(i > 0x3fffffff) {
        i <<= 2;
        shift += 2;
    }
    if !(i > 0x7fffffff) {
        i <<= 1;
        shift += 1;
    }
    // form mantissa with explicit integer bit
    i = i >> 8;
    // add in exponent, taking into account integer bit of mantissa
    if a != 0 {
        i += (127 + 31 - 1 - shift) << 23;
    }
    // add in sign bit
    if a < 0 {
        i |= 0x80000000;
    }
    // reinterpret bit pattern as 'float'
    f32::from_bits(i)
}

#[arduino_uno::entry]
fn main() -> ! {
    let dp = arduino_uno::Peripherals::take().unwrap();
    let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);
    let mut serial = arduino_uno::Serial::new(
        dp.USART0,
        pins.d0,
        pins.d1.into_output(&mut pins.ddr),
        57600,
    );

    for i32 in 0..500 {
        let float_ok = int32_to_float_rz(i32);
        let float_off = i32 as f32;

        let mut buf_ok = Buffer::<32>::new();
        unsafe { dtostrf(float_ok, 7, 4, buf_ok.as_mut_ptr()) };

        let mut buf_off = Buffer::<32>::new();
        unsafe { dtostrf(float_off, 7, 4, buf_off.as_mut_ptr()) };

        ufmt::uwriteln!(&mut serial, "{} {} {}\r", i32, buf_ok, buf_off).void_unwrap();
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
