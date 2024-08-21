use crate::color::Color;
use crate::utils::AsBytes as _;
use libbpf_rs::skel::OpenSkel as _;
use libbpf_rs::skel::SkelBuilder as _;
use libbpf_rs::Error;
use libbpf_rs::MapCore as _;
use libbpf_rs::MapFlags;
use libbpf_rs::OpenObject;
use libbpf_rs::ProgramInput;
use once_cell::sync::Lazy;
use std::fs;
use std::io;
use std::mem::size_of_val;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;
use std::ptr::copy_nonoverlapping;
use std::str;
use write_bpf::WriteSkelBuilder;
pub mod bloody;

#[path = "bpf/write.bpf.rs"]
mod write_bpf;

struct DeviceFunctions {
    probe: fn(&DeviceInfo) -> bool,
    gain_control: fn(&mut Writer) -> Result<(), Error>,
    release_control: fn(&mut Writer) -> Result<(), Error>,
    set_colors: fn(&mut Writer, &[Color; 104]) -> Result<(), Error>,
}

pub struct DeviceHandle {
    functions: &'static DeviceFunctions,
    writer: Writer,
}

impl DeviceHandle {
    pub fn probe(&self, device_info: &DeviceInfo) -> bool {
        (self.functions.probe)(device_info)
    }

    pub fn gain_control(&mut self) -> Result<(), Error> {
        (self.functions.gain_control)(&mut self.writer)
    }

    pub fn release_control(&mut self) -> Result<(), Error> {
        (self.functions.release_control)(&mut self.writer)
    }

    pub fn set_colors(&mut self, colors: &[Color; 104]) -> Result<(), Error> {
        (self.functions.set_colors)(&mut self.writer, colors)
    }
}

trait Device {
    fn probe(info: &DeviceInfo) -> bool;
    fn gain_control(writer: &mut Writer) -> Result<(), Error>;
    fn release_control(writer: &mut Writer) -> Result<(), Error>;
    fn set_colors(writer: &mut Writer, colors: &[Color; 104]) -> Result<(), Error>;
}

struct Writer {
    kernel_writer: &'static mut KernelWriter<'static>,
    hid: u16,
}

impl Writer {
    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        use write_bpf::types::Block;
        use write_bpf::types::Hdr;

        let mut block = Block::default();
        if data.len() > size_of_val(&block) {
            return Err(Error::from(io::Error::new(
                io::ErrorKind::InvalidInput,
                "data size is more than can be",
            )));
        }

        // SAFETY: sizes is checked above
        unsafe {
            copy_nonoverlapping(data.as_ptr(), block.as_bytes_mut().as_mut_ptr(), data.len())
        };

        self.kernel_writer.program.maps.array.update(
            &[0u8; 4],
            block.as_bytes(),
            MapFlags::empty(),
        )?;

        let mut hdr = Hdr {
            hid_id: self.hid as u32,
            data_size: data.len() as u32,
        };
        let mut input = ProgramInput::default();
        input.context_in = Some(hdr.as_bytes_mut());

        self.kernel_writer.program.progs.write.test_run(input)?;

        Ok(())
    }
}

struct KernelWriter<'a> {
    program: write_bpf::WriteSkel<'a>,
}

pub struct DeviceInfo {
    hid: u16,
    vid: u16,
    pid: u16,
    report_descriptor: hrd::Descriptor,
}

pub struct Devices<'a> {
    supported_devices: Vec<DeviceFunctions>,
    kernel_writer: Lazy<KernelWriter<'a>>,
}

impl KernelWriter<'_> {
    fn new() -> Self {
        static mut OBJECT: MaybeUninit<OpenObject> = MaybeUninit::uninit();

        let program = WriteSkelBuilder::default()
            .open(
                // SAFETY: should be called once
                unsafe { &mut *addr_of_mut!(OBJECT) },
            )
            .unwrap();

        let program = program.load().unwrap();

        Self { program }
    }
}

static mut DEVICES: Lazy<Devices<'static>> = Lazy::new(|| Devices {
    supported_devices: Vec::new(),
    kernel_writer: Lazy::new(KernelWriter::new),
});

fn from_hex(data: &[u8]) -> Option<u16> {
    u16::from_str_radix(str::from_utf8(data).ok()?, 16).ok()
}

impl Devices<'_> {
    fn instance() -> &'static mut Devices<'static> {
        // SAFETY: application is single threaded and mutable only at startup
        unsafe { &mut *DEVICES }
    }

    fn register<D: Device>() {
        Self::instance().supported_devices.push(DeviceFunctions {
            probe: D::probe,
            gain_control: D::gain_control,
            release_control: D::release_control,
            set_colors: D::set_colors,
        });
    }

    fn for_each_devices<E>(mut f: impl FnMut(&DeviceInfo) -> Result<(), E>) -> Result<(), E> {
        let dir = fs::read_dir("/sys/bus/hid/devices").unwrap();
        for device_dir in dir {
            let Ok(device_dir) = device_dir else {
                continue;
            };
            let name = device_dir.file_name();
            let name = name.as_encoded_bytes();

            if name.len() != 19 {
                continue;
            }

            let _bus = &name[0..4];
            let vid = &name[5..9];
            let pid = &name[10..14];
            let hid = &name[15..19];

            // SAFETY: vid, pid and hid is valid hex strings
            let vid = unsafe { from_hex(vid).unwrap_unchecked() };
            let pid = unsafe { from_hex(pid).unwrap_unchecked() };
            let hid = unsafe { from_hex(hid).unwrap_unchecked() };

            let report_descriptor = match fs::read(device_dir.path().join("report_descriptor")) {
                Ok(report_descriptor) => report_descriptor,
                Err(err) => {
                    let dev_name = device_dir.file_name().into_string().unwrap();
                    log::error!("{dev_name}: {err}");
                    continue;
                }
            };

            let report_descriptor = match hrd::parse(report_descriptor.as_slice()) {
                Ok(report_descriptor) => report_descriptor,
                Err(err) => {
                    let dev_name = device_dir.file_name().into_string().unwrap();
                    log::error!("{dev_name}: {err}");
                    continue;
                }
            };

            let info = DeviceInfo {
                hid,
                vid,
                pid,
                report_descriptor,
            };

            f(&info)?;
        }

        Ok(())
    }

    pub fn for_each_supported_devices(
        mut f: impl FnMut(&mut DeviceHandle) -> Result<(), Error>,
    ) -> Result<(), Error> {
        Self::for_each_devices(|info| {
            for functions in &Self::instance().supported_devices {
                if (functions.probe)(info) {
                    let mut dev = DeviceHandle {
                        functions,
                        writer: Writer {
                            hid: info.hid,
                            kernel_writer: &mut Self::instance().kernel_writer,
                        },
                    };

                    f(&mut dev)?;
                }
            }

            Ok::<(), Error>(())
        })
    }
}
