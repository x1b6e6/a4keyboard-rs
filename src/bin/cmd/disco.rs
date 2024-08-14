use a4keyboard::color::Color;
use a4keyboard::devices::Devices;
use a4keyboard::utils::AsBytes as _;
use a4keyboard::Error;
use rand::RngCore as _;

fn make_diff(color: Color, diff: Color, speed: u8) -> Color {
    fn make_diff(color: u8, diff: u8, speed: u8) -> u8 {
        let m = 1 << speed;
        let s = (255 / m / 2) + 1;
        let diff = diff / m;

        if diff < s {
            color.checked_add(diff).unwrap_or(color)
        } else {
            color.checked_sub(diff - s).unwrap_or(color)
        }
    }

    Color {
        r: make_diff(color.r, diff.r, speed),
        g: make_diff(color.g, diff.g, speed),
        b: make_diff(color.b, diff.b, speed),
    }
}

pub fn run() -> Result<(), Error> {
    let mut values = [Color::default(); 104];

    let mut rng = rand::thread_rng();
    rng.fill_bytes(values.as_bytes_mut());

    let mut speeds = [Color::default(); 104];
    rng.fill_bytes(speeds.as_bytes_mut());

    let mut accels = [Color::default(); 104];

    loop {
        rng.fill_bytes(accels.as_bytes_mut());

        values
            .iter_mut()
            .zip(speeds.iter().copied())
            .for_each(|(value, speed)| *value = make_diff(*value, speed, 2));

        speeds
            .iter_mut()
            .zip(accels.iter().copied())
            .for_each(|(speed, accel)| *speed = make_diff(*speed, accel, 2));

        Devices::for_each_supported_devices(|dev| dev.set_colors(&values))?;
    }
}
