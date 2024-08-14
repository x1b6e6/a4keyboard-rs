use a4keyboard::color::Color;
use a4keyboard::devices::DeviceHandle;
use a4keyboard::devices::Devices;

mod cmd {
    pub mod color;

    #[cfg(feature = "disco")]
    pub mod disco;
}

#[derive(clap::Parser)]
enum Command {
    /// Set color to all keys
    Color {
        #[arg(value_name = "HEXCOLOR")]
        color: Color,
    },

    /// Enter "disco" mode
    #[cfg(feature = "disco")]
    Disco {},
}

#[derive(clap::Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,

    #[arg(long)]
    no_gain_control: bool,
}

fn main() {
    env_logger::init();

    let Args {
        command,
        no_gain_control,
    } = clap::Parser::parse();

    if !no_gain_control {
        Devices::for_each_supported_devices(DeviceHandle::gain_control).unwrap();
    }

    match command {
        Command::Color { color } => {
            cmd::color::run(color).unwrap();
        }

        #[cfg(feature = "disco")]
        Command::Disco {} => {
            cmd::disco::run().unwrap();
        }
    }
}
