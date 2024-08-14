use crate::color::Color;
use crate::devices::Device;
use crate::devices::DeviceInfo;
use crate::devices::Devices;
use crate::devices::Writer;
use crate::utils::startup;
use crate::Error;

struct Bloody;

startup! {
    Devices::register::<Bloody>();
}

impl Bloody {
    fn take_control(writer: &mut Writer, value: bool) -> Result<(), Error> {
        writer.write(&make_prepared_control_packet(0x01))?;

        let mut buf = make_prepared_control_packet(0x00);
        if value {
            buf[8] = 0x01;
        }
        writer.write(&buf)?;

        Ok(())
    }
}

const fn make_prepared_control_packet(target: u8) -> [u8; 64] {
    let mut buf = [0u8; 64];
    buf[0] = 0x07;
    buf[1] = 0x03;
    buf[2] = 0x06;
    buf[3] = target;
    buf
}

const BLOODY_B820R_RGB_OFFSET: usize = 6;
const BLOODY_B820R_RGB_BUFFER_SIZE: usize = 58;

impl Device for Bloody {
    fn probe(info: &DeviceInfo) -> bool {
        if info.vid != 0x09da || info.pid != 0xfa10 {
            return false;
        }

        info.report_descriptor
            .iter()
            .any(|report| report.usage_page == 0xFF52 && report.usage == 0x0210)
    }

    fn gain_control(writer: &mut Writer) -> Result<(), Error> {
        Self::take_control(writer, true)
    }

    fn release_control(writer: &mut Writer) -> Result<(), Error> {
        Self::take_control(writer, false)
    }

    fn set_colors(writer: &mut Writer, colors: &[Color; 104]) -> Result<(), Error> {
        let mut r1_buf = make_prepared_control_packet(0x07);
        let mut g1_buf = make_prepared_control_packet(0x09);
        let mut b1_buf = make_prepared_control_packet(0x0B);
        let mut r2_buf = make_prepared_control_packet(0x08);
        let mut g2_buf = make_prepared_control_packet(0x0A);
        let mut b2_buf = make_prepared_control_packet(0x0C);

        let buffers = [
            [&mut r1_buf, &mut g1_buf, &mut b1_buf],
            [&mut r2_buf, &mut g2_buf, &mut b2_buf],
        ];

        for (i, color) in colors.iter().enumerate() {
            let offset = BLOODY_B820R_RGB_OFFSET;
            let buffer_idx = offset + i % BLOODY_B820R_RGB_BUFFER_SIZE;
            let line = i / BLOODY_B820R_RGB_BUFFER_SIZE;

            buffers[line][0][buffer_idx] = color.r;
            buffers[line][1][buffer_idx] = color.g;
            buffers[line][2][buffer_idx] = color.b;
        }

        writer.write(&r1_buf)?;
        writer.write(&r2_buf)?;
        writer.write(&g1_buf)?;
        writer.write(&g2_buf)?;
        writer.write(&b1_buf)?;
        writer.write(&b2_buf)?;

        Ok(())
    }
}
