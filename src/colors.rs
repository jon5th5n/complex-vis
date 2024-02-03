use crate::color::RGBA;

pub const TRANSPARANT: RGBA = RGBA {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

pub const BLACK: RGBA = RGBA {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};

pub const WHITE: RGBA = RGBA {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
};

pub const RED: RGBA = RGBA {
    r: 255,
    g: 0,
    b: 0,
    a: 255,
};

pub const GREEN: RGBA = RGBA {
    r: 0,
    g: 255,
    b: 0,
    a: 255,
};

pub const BLUE: RGBA = RGBA {
    r: 0,
    g: 0,
    b: 255,
    a: 255,
};
