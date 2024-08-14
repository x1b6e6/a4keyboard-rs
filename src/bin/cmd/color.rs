use a4keyboard::color::Color;
use a4keyboard::devices::Devices;
use a4keyboard::Error;

pub fn run(color: Color) -> Result<(), Error> {
    let colors = [color; 104];

    Devices::for_each_supported_devices(|dev| dev.set_colors(&colors))
}
