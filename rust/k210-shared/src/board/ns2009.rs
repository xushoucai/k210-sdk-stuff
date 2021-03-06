use core::result::Result;

use crate::soc::i2c;
use crate::util::filters;

/* low level functions */
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum command {
    LOW_POWER_READ_X = 0xc0,
    LOW_POWER_READ_Y = 0xd0,
    LOW_POWER_READ_Z1 = 0xe0,
    LOW_POWER_READ_Z2 = 0xf0,
}

pub fn read(cmd: command) -> Result<u16, ()>
{
    let mut buf = [0u8; 2];
    if i2c::recv_data(&[cmd as u8], &mut buf).is_ok() {
        Ok(((buf[0] as u16) << 4) | ((buf[1] as u16) >> 4))
    } else {
        Err(())
    }
}

/** Position filter */
pub struct TSFilter {
    mx: filters::Median<i32>,
    my: filters::Median<i32>,
    nx: filters::Mean<i32>,
    ny: filters::Mean<i32>,
    cal: [i32; 7],
}

impl TSFilter {
    /* input: calibration matrix */
    pub fn new(cal: [i32; 7]) -> Self {
        Self {
            mx: filters::Median::<i32>::new(),
            my: filters::Median::<i32>::new(),
            nx: filters::Mean::<i32>::new(),
            ny: filters::Mean::<i32>::new(),
            cal,
        }
    }

    pub fn update(&mut self, x: u16, y: u16) -> (i32, i32) {
        let tx = self.mx.update(x as i32);
        let ty = self.my.update(y as i32);
        let tx = self.nx.update(tx);
        let ty = self.ny.update(ty);
        ((self.cal[2] + self.cal[0] * tx + self.cal[1] * ty) / self.cal[6],
         (self.cal[5] + self.cal[3] * tx + self.cal[4] * ty) / self.cal[6])
    }

    pub fn clear(&mut self) {
        self.mx.clear();
        self.my.clear();
        self.nx.clear();
        self.ny.clear();
    }
}

/** Event kind */
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EventKind {
    Begin,
    Move,
    End,
}

/** Touch screen event */
#[derive(Debug, Clone)]
pub struct Event {
    pub kind: EventKind,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/** High-level touch screen abstraction */
pub struct TouchScreen {
    filter: TSFilter,
    press: bool,
    x: i32,
    y: i32,
}

impl TouchScreen {
    /* input: calibration matrix */
    pub fn init(cal: [i32; 7]) -> Option<Self> {
        // Do a test read
        if let Ok(_) = read(command::LOW_POWER_READ_Z1) {
            Some(Self {
                filter: TSFilter::new(cal),
                press: false,
                x: 0,
                y: 0,
            })
        } else {
            None
        }
    }

    /** Poll for touch screen event, return the current event (Begin, Move, End) or None */
    pub fn poll(&mut self) -> Option<Event> {
        let mut ev: Option<Event> = None;
        if let Ok(z1) = read(command::LOW_POWER_READ_Z1) {
            if z1 > 80 && z1 < 2000 {
                if let (Ok(x), Ok(y)) = (read(command::LOW_POWER_READ_X), read(command::LOW_POWER_READ_Y)) {
                    let (x, y) = self.filter.update(x, y);
                    if !self.press
                    {
                        self.press = true;
                        ev = Some(Event { kind: EventKind::Begin, x, y, z: z1 as i32 });
                    }
                    else if self.x != x || self.y != y {
                        ev = Some(Event { kind: EventKind::Move, x, y, z: z1 as i32 });
                    }
                    self.x = x;
                    self.y = y;
                }
            } else {
                if self.press
                {
                    self.filter.clear();
                    self.press = false;
                    ev = Some(Event { kind: EventKind::End, x: self.x, y: self.y, z: 0 });
                }
            }
        }
        ev
    }
}
