/** ST7789V LCD driver */
use k210_hal::pac;
use pac::spi0::ctrlr0;
use pac::spi0::spi_ctrlr0;

use crate::soc::gpio;
use crate::soc::gpiohs;
use crate::soc::sleep::usleep;
use crate::soc::spi;

pub const SPI_SLAVE_SELECT: u32 = 3;
pub const DCX_GPIONUM: u8 = 2;
pub const RST_GPIONUM: u8 = 3;

pub const LCD_X_MAX: u16 = 320;
pub const LCD_Y_MAX: u16 = 240;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum command {
    NO_OPERATION = 0x00,
    SOFTWARE_RESET = 0x01,
    READ_ID = 0x04,
    READ_STATUS = 0x09,
    READ_POWER_MODE = 0x0A,
    READ_MADCTL = 0x0B,
    READ_PIXEL_FORMAT = 0x0C,
    READ_IMAGE_FORMAT = 0x0D,
    READ_SIGNAL_MODE = 0x0E,
    READ_SELT_DIAG_RESULT = 0x0F,
    SLEEP_ON = 0x10,
    SLEEP_OFF = 0x11,
    PARTIAL_DISPLAY_ON = 0x12,
    NORMAL_DISPLAY_ON = 0x13,
    INVERSION_DISPLAY_OFF = 0x20,
    INVERSION_DISPLAY_ON = 0x21,
    GAMMA_SET = 0x26,
    DISPLAY_OFF = 0x28,
    DISPLAY_ON = 0x29,
    HORIZONTAL_ADDRESS_SET = 0x2A,
    VERTICAL_ADDRESS_SET = 0x2B,
    MEMORY_WRITE = 0x2C,
    COLOR_SET = 0x2D,
    MEMORY_READ = 0x2E,
    PARTIAL_AREA = 0x30,
    VERTICAL_SCROLL_DEFINE = 0x33,
    TEAR_EFFECT_LINE_OFF = 0x34,
    TEAR_EFFECT_LINE_ON = 0x35,
    MEMORY_ACCESS_CTL = 0x36,
    VERTICAL_SCROLL_S_ADD = 0x37,
    IDLE_MODE_OFF = 0x38,
    IDLE_MODE_ON = 0x39,
    PIXEL_FORMAT_SET = 0x3A,
    WRITE_MEMORY_CONTINUE = 0x3C,
    READ_MEMORY_CONTINUE = 0x3E,
    SET_TEAR_SCANLINE = 0x44,
    GET_SCANLINE = 0x45,
    WRITE_BRIGHTNESS = 0x51,
    READ_BRIGHTNESS = 0x52,
    WRITE_CTRL_DISPLAY = 0x53,
    READ_CTRL_DISPLAY = 0x54,
    WRITE_BRIGHTNESS_CTL = 0x55,
    READ_BRIGHTNESS_CTL = 0x56,
    WRITE_MIN_BRIGHTNESS = 0x5E,
    READ_MIN_BRIGHTNESS = 0x5F,
    READ_ID1 = 0xDA,
    READ_ID2 = 0xDB,
    READ_ID3 = 0xDC,
    RGB_IF_SIGNAL_CTL = 0xB0,
    NORMAL_FRAME_CTL = 0xB1,
    IDLE_FRAME_CTL = 0xB2,
    PARTIAL_FRAME_CTL = 0xB3,
    INVERSION_CTL = 0xB4,
    BLANK_PORCH_CTL = 0xB5,
    DISPLAY_FUNCTION_CTL = 0xB6,
    ENTRY_MODE_SET = 0xB7,
    BACKLIGHT_CTL1 = 0xB8,
    BACKLIGHT_CTL2 = 0xB9,
    BACKLIGHT_CTL3 = 0xBA,
    BACKLIGHT_CTL4 = 0xBB,
    BACKLIGHT_CTL5 = 0xBC,
    BACKLIGHT_CTL7 = 0xBE,
    BACKLIGHT_CTL8 = 0xBF,
    POWER_CTL1 = 0xC0,
    POWER_CTL2 = 0xC1,
    VCOM_CTL1 = 0xC5,
    VCOM_CTL2 = 0xC7,
    NV_MEMORY_WRITE = 0xD0,
    NV_MEMORY_PROTECT_KEY = 0xD1,
    NV_MEMORY_STATUS_READ = 0xD2,
    READ_ID4 = 0xD3,
    POSITIVE_GAMMA_CORRECT = 0xE0,
    NEGATIVE_GAMMA_CORRECT = 0xE1,
    DIGITAL_GAMMA_CTL1 = 0xE2,
    DIGITAL_GAMMA_CTL2 = 0xE3,
    INTERFACE_CTL = 0xF6,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum direction {
    XY_RLUD = 0x00,
    YX_RLUD = 0x20,
    XY_LRUD = 0x40,
    YX_LRUD = 0x60,
    XY_RLDU = 0x80,
    YX_RLDU = 0xA0,
    XY_LRDU = 0xC0,
    YX_LRDU = 0xE0,
}
pub const DIR_XY_MASK: u8 = 0x20;
pub const DIR_MASK: u8 = 0xE0;

/* Low-level functions */

fn init_dcx() {
    gpiohs::set_direction(DCX_GPIONUM, gpio::direction::OUTPUT);
    gpiohs::set_pin(DCX_GPIONUM, true);
}

fn set_dcx_control() {
    gpiohs::set_pin(DCX_GPIONUM, false);
}

fn set_dcx_data() {
    gpiohs::set_pin(DCX_GPIONUM, true);
}

fn init_rst() {
    gpiohs::set_direction(RST_GPIONUM, gpio::direction::OUTPUT);
    gpiohs::set_pin(RST_GPIONUM, true);
}

fn set_rst(val: bool) {
    gpiohs::set_pin(RST_GPIONUM, val);
}

pub fn hard_init() {
    init_dcx();
    init_rst();
    set_rst(false);
    spi::clk_init();
    spi::init(
        ctrlr0::WORK_MODEW::MODE0,
        ctrlr0::FRAME_FORMATW::OCTAL,
        8,
        0,
        8, /*instruction length*/
        0, /*address length*/
        0, /*wait cycles*/
        spi_ctrlr0::AITMW::AS_FRAME_FORMAT,
        ctrlr0::TMODW::TRANS,
    );
    spi::set_clk_rate(10000000);
    set_rst(true);
}

pub fn write_command(cmd: command) {
    set_dcx_control();
    spi::init(
        ctrlr0::WORK_MODEW::MODE0,
        ctrlr0::FRAME_FORMATW::OCTAL,
        8,
        0,
        8, /*instruction length*/
        0, /*address length*/
        0, /*wait cycles*/
        spi_ctrlr0::AITMW::AS_FRAME_FORMAT,
        ctrlr0::TMODW::TRANS,
    );
    spi::send_data(SPI_SLAVE_SELECT, &[cmd as u8]);
}

pub fn write_byte(data_buf: &[u8]) {
    set_dcx_data();
    spi::init(
        ctrlr0::WORK_MODEW::MODE0,
        ctrlr0::FRAME_FORMATW::OCTAL,
        8,
        0,
        0, /*instruction length*/
        8, /*address length*/
        0, /*wait cycles*/
        spi_ctrlr0::AITMW::AS_FRAME_FORMAT,
        ctrlr0::TMODW::TRANS,
    );
    spi::send_data(SPI_SLAVE_SELECT, data_buf);
}

pub fn write_half(data_buf: &[u16]) {
    set_dcx_data();
    spi::init(
        ctrlr0::WORK_MODEW::MODE0,
        ctrlr0::FRAME_FORMATW::OCTAL,
        16,
        0,
        0,  /*instruction length*/
        16, /*address length*/
        0,  /*wait cycles*/
        spi_ctrlr0::AITMW::AS_FRAME_FORMAT,
        ctrlr0::TMODW::TRANS,
    );
    spi::send_data(SPI_SLAVE_SELECT, data_buf);
}

pub fn write_word(data_buf: &[u32]) {
    set_dcx_data();
    spi::init(
        ctrlr0::WORK_MODEW::MODE0,
        ctrlr0::FRAME_FORMATW::OCTAL,
        32,
        0,
        0,  /*instruction length*/
        32, /*address length*/
        0,  /*wait cycles*/
        spi_ctrlr0::AITMW::AS_FRAME_FORMAT,
        ctrlr0::TMODW::TRANS,
    );
    spi::send_data(SPI_SLAVE_SELECT, data_buf);
}

pub fn fill_data(data: u32, length: usize) {
    set_dcx_data();
    spi::init(
        ctrlr0::WORK_MODEW::MODE0,
        ctrlr0::FRAME_FORMATW::OCTAL,
        32,
        0,
        0,  /*instruction length*/
        32, /*address length*/
        0,  /*wait cycles*/
        spi_ctrlr0::AITMW::AS_FRAME_FORMAT,
        ctrlr0::TMODW::TRANS,
    );
    spi::fill_data(SPI_SLAVE_SELECT, data, length);
}

/* High-level functions */

pub fn init() {
    hard_init();
    /*soft reset*/
    write_command(command::SOFTWARE_RESET);
    usleep(100000);
    /*exit sleep*/
    write_command(command::SLEEP_OFF);
    usleep(100000);
    /*pixel format*/
    write_command(command::PIXEL_FORMAT_SET);
    write_byte(&[0x55]);
    set_direction(direction::XY_LRUD);

    /*display on*/
    write_command(command::DISPLAY_ON);
}

pub fn set_direction(dir: direction) {
    /* No support for YX orientations right now --
    lcd_ctl.dir = dir;
    if (dir & DIR_XY_MASK)
    {
        lcd_ctl.width = LCD_Y_MAX - 1;
        lcd_ctl.height = LCD_X_MAX - 1;
    }
    else
    {
        lcd_ctl.width = LCD_X_MAX - 1;
        lcd_ctl.height = LCD_Y_MAX - 1;
    }
    */

    write_command(command::MEMORY_ACCESS_CTL);
    write_byte(&[dir as u8]);
}

pub fn set_area(x1: u16, y1: u16, x2: u16, y2: u16) {
    write_command(command::HORIZONTAL_ADDRESS_SET);
    write_byte(&[
        (x1 >> 8) as u8,
        (x1 & 0xff) as u8,
        (x2 >> 8) as u8,
        (x2 & 0xff) as u8,
    ]);

    write_command(command::VERTICAL_ADDRESS_SET);
    write_byte(&[
        (y1 >> 8) as u8,
        (y1 & 0xff) as u8,
        (y2 >> 8) as u8,
        (y2 & 0xff) as u8,
    ]);

    write_command(command::MEMORY_WRITE);
}

pub fn clear(color: u16) {
    let data = ((color as u32) << 16) | (color as u32);

    //set_area(0, 0, lcd_ctl.width, lcd_ctl.height);
    set_area(0, 0, LCD_X_MAX - 1, LCD_Y_MAX - 1);
    fill_data(data, (LCD_X_MAX as usize) * (LCD_Y_MAX as usize) / 2);
}

pub fn draw_picture(x1: u16, y1: u16, width: u16, height: u16, data: &[u32]) {
    set_area(x1, y1, x1 + width - 1, y1 + height - 1);
    assert!(data.len() == (width as usize) * (height as usize) / 2);
    write_word(data);
}
