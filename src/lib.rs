//! This is an embedded-hal device driver for the Sparkfun SerLCD LCD screen.

use embedded_hal as hal;
use hal::blocking::delay::DelayMs;
use hal::blocking::spi::{Transfer, Write};
use hal::digital::v2::OutputPin;

#[derive(Debug)]
pub enum Error<SpiE, PinE> {
    Spi(SpiE),
    Pin(PinE),
}

pub struct SerLCD<SPI, CS, DS> {
    spi: SPI,
    cs: CS,
    delay_source: DS,
    display_control: u8,
    display_mode: u8,
}

impl<SPI, CS, DS, SpiE, PinE> SerLCD<SPI, CS, DS>
where
    SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    CS: OutputPin<Error = PinE>,
    DS: DelayMs<u8>,
    SpiE: core::fmt::Debug,
    PinE: core::fmt::Debug,
{
    pub fn new(spi: SPI, cs: CS, delay_source: DS) -> Self {
        Self {
            spi,
            cs,
            delay_source,
            display_control: LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF,
            display_mode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT,
        }
    }

    pub fn setup(&mut self) -> Result<(), Error<SpiE, PinE>> {
        self.begin_transmission()?;
        self.transmit(SPECIAL_COMMAND)?;
        self.transmit(LCD_DISPLAYCONTROL)?;
        self.transmit(SPECIAL_COMMAND)?;
        self.transmit(LCD_ENTRYMODESET)?;
        self.transmit(SETTING_COMMAND)?;
        self.transmit(CLEAR_COMMAND)?;
        self.end_transmission()?;

        self.delay_source.delay_ms(50);

        Ok(())
    }

    pub fn command(&mut self, command: u8) -> Result<(), Error<SpiE, PinE>> {
        self.begin_transmission()?;
        self.transmit(SETTING_COMMAND)?;
        self.transmit(command)?;
        self.end_transmission()?;

        self.delay_source.delay_ms(10);

        Ok(())
    }

    pub fn special_command(&mut self, command: u8) -> Result<(), Error<SpiE, PinE>> {
        self.begin_transmission()?;
        self.transmit(SPECIAL_COMMAND)?;
        self.transmit(command)?;
        self.end_transmission()?;

        self.delay_source.delay_ms(50);

        Ok(())
    }

    pub fn special_command_count(
        &mut self,
        command: u8,
        count: u8,
    ) -> Result<(), Error<SpiE, PinE>> {
        self.begin_transmission()?;

        for _ in 0..count {
            self.transmit(SPECIAL_COMMAND)?;
            self.transmit(command)?;
        }

        self.end_transmission()?;

        self.delay_source.delay_ms(50);

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error<SpiE, PinE>> {
        self.command(CLEAR_COMMAND)?;
        self.delay_source.delay_ms(10);
        Ok(())
    }

    pub fn home(&mut self) -> Result<(), Error<SpiE, PinE>> {
        self.special_command(LCD_RETURNHOME)
    }

    pub fn set_cursor(&mut self, col: u8, row: u8) -> Result<(), Error<SpiE, PinE>> {
        let row_offsets = [0x00, 0x40, 0x14, 0x54];

        let mut row = std::cmp::max(0, row);
        row = std::cmp::min(row, MAX_ROWS - 1);

        self.special_command(LCD_SETDDRAMADDR | (col + row_offsets[row as usize]))?;

        Ok(())
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), Error<SpiE, PinE>> {
        self.begin_transmission()?;

        for b in buf {
            self.transmit(*b)?;
        }

        self.end_transmission()?;

        self.delay_source.delay_ms(10);

        Ok(())
    }

    pub fn write_str(&mut self, s: &str) -> Result<(), Error<SpiE, PinE>> {
        if !s.is_empty() {
            self.write(s.as_bytes())?;
        }

        Ok(())
    }

    pub fn no_display(&mut self) -> Result<(), Error<SpiE, PinE>> {
        self.display_control &= !LCD_DISPLAYON;
        self.special_command(LCD_DISPLAYCONTROL | self.display_control)?;
        Ok(())
    }

    pub fn display(&mut self) -> Result<(), Error<SpiE, PinE>> {
        self.display_control |= LCD_DISPLAYON;
        self.special_command(LCD_DISPLAYCONTROL | self.display_control)?;
        Ok(())
    }

    pub fn no_cursor(&mut self) -> Result<(), Error<SpiE, PinE>> {
        self.display_control &= !LCD_CURSORON;
        self.special_command(LCD_DISPLAYCONTROL | self.display_control)?;
        Ok(())
    }

    pub fn cursor(&mut self) -> Result<(), Error<SpiE, PinE>> {
        self.display_control |= LCD_CURSORON;
        self.special_command(LCD_DISPLAYCONTROL | self.display_control)?;
        Ok(())
    }

    fn begin_transmission(&mut self) -> Result<(), Error<SpiE, PinE>> {
        self.cs.set_low().map_err(Error::Pin)?;

        self.delay_source.delay_ms(10);

        Ok(())
    }

    fn end_transmission(&mut self) -> Result<(), Error<SpiE, PinE>> {
        self.cs.set_high().map_err(Error::Pin)?;

        self.delay_source.delay_ms(10);

        Ok(())
    }

    fn transmit(&mut self, data: u8) -> Result<(), Error<SpiE, PinE>> {
        let rc = self.spi.write(&[data]);
        rc.map_err(Error::Spi)?;

        Ok(())
    }
}

const DISPLAY_ADDRESS: u8 = 0x72;
const MAX_ROWS: u8 = 4;
const MAX_COLUMNS: u8 = 20;

const SPECIAL_COMMAND: u8 = 254;
const SETTING_COMMAND: u8 = 0x7c;

const CLEAR_COMMAND: u8 = 0x2d;
const CONTRAST_COMMAND: u8 = 0x18;
const ADDRESS_COMMAND: u8 = 0x19;
const SET_RGB_COMMAND: u8 = 0x2b;
const ENABLE_SYSTEM_MESSAGE_DISPLAY: u8 = 0x2e;
const DISABLE_SYSTEM_MESSAGE_DISPLAY: u8 = 0x2f;
const ENABLE_SPLASH_DISPLAY: u8 = 0x30;
const DISABLE_SPLASH_DISPLAY: u8 = 0x31;
const SAVE_CURRENT_DISPLAY_AS_SPLASH: u8 = 0x0a;

const LCD_RETURNHOME: u8 = 0x02;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
const LCD_CURSORSHIFT: u8 = 0x10;
const LCD_SETDDRAMADDR: u8 = 0x80;

const LCD_ENTRYRIGHT: u8 = 0x00;
const LCD_ENTRYLEFT: u8 = 0x02;
const LCD_ENTRYSHIFTINCREMENT: u8 = 0x01;
const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

const LCD_DISPLAYON: u8 = 0x04;
const LCD_DISPLAYOFF: u8 = 0x00;
const LCD_CURSORON: u8 = 0x02;
const LCD_CURSOROFF: u8 = 0x00;
const LCD_BLINKON: u8 = 0x01;
const LCD_BLINKOFF: u8 = 0x00;

const LCD_DISPLAYMOVE: u8 = 0x08;
const LCD_CURSORMOVE: u8 = 0x00;
const LCD_MOVERIGHT: u8 = 0x04;
const LCD_MOVELEFT: u8 = 0x00;
