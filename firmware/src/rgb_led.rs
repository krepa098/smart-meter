// RGB LED is on IO3 (R) IO4 (G) IO5 (B)

pub struct RGBLed<R, G, B>
where
    R: embedded_hal::digital::v2::OutputPin,
    G: embedded_hal::digital::v2::OutputPin,
    B: embedded_hal::digital::v2::OutputPin,
{
    pub r: R,
    pub g: G,
    pub b: B,
}

impl<R, G, B> RGBLed<R, G, B>
where
    R: embedded_hal::digital::v2::OutputPin,
    G: embedded_hal::digital::v2::OutputPin,
    B: embedded_hal::digital::v2::OutputPin,
{
    pub fn set_red(&mut self) {
        self.r.set_high().ok();
        self.g.set_low().ok();
        self.b.set_low().ok();
    }
}

//pub fn set_rgb_color(R,G,B)

pub struct RGBLedPwm<R, G, B>
where
    R: embedded_hal::PwmPin,
    G: embedded_hal::PwmPin,
    B: embedded_hal::PwmPin,
{
    pub r: R,
    pub g: G,
    pub b: B,

    pub rgb: (u8, u8, u8),
}

#[allow(unused)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Teal,
    White,
    Black,
}

impl<R, G, B> RGBLedPwm<R, G, B>
where
    R: embedded_hal::PwmPin<Duty = u32>,
    G: embedded_hal::PwmPin<Duty = u32>,
    B: embedded_hal::PwmPin<Duty = u32>,
{
    pub fn set_color(&mut self, color: &Color) {
        let rgb = match color {
            Color::Red => (0xFF, 0, 0),
            Color::Green => (0, 0xFF, 0),
            Color::Blue => (0, 0, 0xFF),
            Color::Yellow => (0xFF, 0xFF, 0),
            Color::Magenta => (0xFF, 0, 0xFF),
            Color::Teal => (0, 0xFF, 0xFF),
            Color::White => (0xFF, 0xFF, 0xFF),
            Color::Black => (0, 0, 0),
        };
        self.set_rgb(rgb.0 / 4, rgb.1 / 4, rgb.2 / 4);
    }

    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        let r_max = self.r.get_max_duty();
        let g_max = self.r.get_max_duty();
        let b_max = self.r.get_max_duty();

        self.r.set_duty(0xFF - (r_max * r as u32 / 0xFF));
        self.g.set_duty(0xFF - (g_max * g as u32 / 0xFF));
        self.b.set_duty(0xFF - (b_max * b as u32 / 0xFF));

        self.rgb = (r, g, b);
    }

    pub fn set_hsv(&mut self, hue: f32, sat: f32, val: f32) {
        let x = val;
        let m = x * (1.0 - sat);
        let z = (x - m) * (1.0 - (((hue / 60.0) % 2.0) - 1.0).abs());

        let rgb_p = match hue {
            h if (0.0..60.0).contains(&h) => (x, z + m, m),
            h if (60.0..120.0).contains(&h) => (z + m, x, m),
            h if (120.0..180.0).contains(&h) => (m, x, z + m),
            h if (180.0..240.0).contains(&h) => (m, z + m, x),
            h if (240.0..300.0).contains(&h) => (z + m, m, x),
            h if (300.0..360.0).contains(&h) => (x, m, z + m),
            _ => (0.0, 0.0, 0.0),
        };
        let rgb = (
            ((rgb_p.0) * 255.0) as u8,
            ((rgb_p.1) * 255.0) as u8,
            ((rgb_p.2) * 255.0) as u8,
        );

        self.set_rgb(rgb.0, rgb.1, rgb.2);
    }
}
