use crate::Collection;
use crate::Descriptor;
use crate::Error;
use crate::Report;
use crate::ReportType;
use std::mem::size_of;
use std::mem::swap;
use std::mem::zeroed;
use std::ptr::copy_nonoverlapping;

#[derive(Clone, Default)]
pub(crate) struct ParserGlobalState {
    pub usage_page: Option<u32>,
    pub report_size: Option<u32>,
    pub report_id: Option<u32>,
    pub report_count: Option<u32>,
    pub logical_minimum: Option<i32>,
    pub logical_maximum: Option<i32>,
    pub physical_minimum: Option<i32>,
    pub physical_maximum: Option<i32>,
}

#[derive(Default)]
pub(crate) struct ParserCollection {
    pub r#type: Option<u32>,
    pub reports: Vec<Report>,
    pub collections: Vec<ParserCollection>,
    pub state: ParserLocalState,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ParserLocalState {
    pub usage: Option<u32>,
    pub usage_minimum: Option<u32>,
    pub usage_maximum: Option<u32>,
}

trait FromBytes: Copy {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let len = bytes.len();

        if len > size_of::<Self>() || len == 0 || (len & (len - 1)) != 0 {
            return None;
        }

        // SAFETY: its used only with primitive types like u32 and i32
        unsafe {
            let mut value = zeroed();
            copy_nonoverlapping(bytes.as_ptr(), &mut value as *mut Self as *mut u8, len);
            Some(value)
        }
    }
}

impl FromBytes for u32 {}
impl FromBytes for i32 {}

const LEN_MASK: u8 = 0x3;

const TAG_MASK: u8 = 0xFC;

const TAG_INPUT: u8 = 0x80;
const TAG_OUTPUT: u8 = 0x90;
const TAG_COLLECTION: u8 = 0xA0;
const TAG_FEATURE: u8 = 0xB0;
const TAG_COLLECTION_END: u8 = 0xC0;

const TAG_USAGE_PAGE: u8 = 0x04;
const TAG_LOGICAL_MINIMUM: u8 = 0x14;
const TAG_LOGICAL_MAXIMUM: u8 = 0x24;
const TAG_PHYSICAL_MINIMUM: u8 = 0x34;
const TAG_PHYSICAL_MAXIMUM: u8 = 0x44;
const TAG_REPORT_SIZE: u8 = 0x74;
const TAG_REPORT_ID: u8 = 0x84;
const TAG_REPORT_COUNT: u8 = 0x94;
const TAG_PUSH: u8 = 0xA4;
const TAG_POP: u8 = 0xB4;

const TAG_USAGE: u8 = 0x08;
const TAG_USAGE_MINIMUM: u8 = 0x18;
const TAG_USAGE_MAXIMUM: u8 = 0x28;

const TAG_EXTENDED: u8 = 0xFC;

enum Tag {
    Long(u8),
    Short(u8),
}

#[derive(Default)]
pub struct Parser {
    stack_global: Vec<ParserGlobalState>,
    global: ParserGlobalState,
    stack_collection: Vec<ParserCollection>,
    collection: ParserCollection,
}

impl Parser {
    pub fn parse(mut self, data: &[u8]) -> Result<Descriptor, super::Error> {
        let mut it = data.iter().copied();

        self.inner_parse(&mut it)?;

        if !self.stack_collection.is_empty() {
            return Err(Error::MissingEndCollection);
        }

        if !self.stack_global.is_empty() {
            return Err(Error::MissingPop);
        }

        let collection = Collection::from(self.collection);

        Ok(Descriptor::new(collection))
    }

    fn inner_parse(&mut self, it: &mut dyn Iterator<Item = u8>) -> Result<(), super::Error> {
        while let Some(byte) = it.next() {
            let (len, tag) = if byte & TAG_MASK == TAG_EXTENDED {
                let len = it.next().ok_or(Error::UnexpectedEndOfReportDescriptor)?;
                let tag = it.next().ok_or(Error::UnexpectedEndOfReportDescriptor)?;

                (len, Tag::Long(tag))
            } else {
                let mut len = byte & LEN_MASK;
                let tag = byte & TAG_MASK;
                if len == 3 {
                    len = 4;
                }

                (len, Tag::Short(tag))
            };

            let data = Vec::from_iter(it.take(len as usize));
            if data.len() != len as usize {
                return Err(Error::UnexpectedEndOfReportDescriptor);
            }

            match tag {
                Tag::Short(TAG_USAGE_PAGE) => {
                    self.global.usage_page =
                        Some(u32::from_bytes(&data).ok_or(Error::BadUsagePage)?)
                }
                Tag::Short(TAG_USAGE) => {
                    self.collection.state.usage =
                        Some(u32::from_bytes(&data).ok_or(Error::BadUsage)?)
                }
                Tag::Short(TAG_USAGE_MINIMUM) => {
                    self.collection.state.usage_minimum =
                        Some(u32::from_bytes(&data).ok_or(Error::BadUsageMinimum)?)
                }
                Tag::Short(TAG_USAGE_MAXIMUM) => {
                    self.collection.state.usage_maximum =
                        Some(u32::from_bytes(&data).ok_or(Error::BadUsageMaximum)?)
                }
                Tag::Short(TAG_COLLECTION) => {
                    let mut collection = ParserCollection::default();
                    collection.r#type = Some(u32::from_bytes(&data).ok_or(Error::BadCollection)?);
                    collection.state = self.collection.state.clone();

                    swap(&mut collection, &mut self.collection);

                    self.stack_collection.push(collection)
                }
                Tag::Short(TAG_COLLECTION_END) => {
                    let mut collection = self
                        .stack_collection
                        .pop()
                        .ok_or(Error::UnexpectedEndCollection)?;

                    swap(&mut collection, &mut self.collection);

                    self.collection.collections.push(collection)
                }
                Tag::Short(TAG_REPORT_SIZE) => {
                    self.global.report_size =
                        Some(u32::from_bytes(&data).ok_or(Error::BadReportSize)?)
                }
                Tag::Short(TAG_REPORT_ID) => {
                    self.global.report_id = Some(u32::from_bytes(&data).ok_or(Error::BadReportId)?)
                }
                Tag::Short(TAG_REPORT_COUNT) => {
                    self.global.report_count =
                        Some(u32::from_bytes(&data).ok_or(Error::BadReportCount)?)
                }
                Tag::Short(TAG_LOGICAL_MINIMUM) => {
                    self.global.logical_minimum =
                        Some(i32::from_bytes(&data).ok_or(Error::BadLogicalMinimum)?)
                }
                Tag::Short(TAG_LOGICAL_MAXIMUM) => {
                    self.global.logical_maximum =
                        Some(i32::from_bytes(&data).ok_or(Error::BadLogicalMaximum)?)
                }
                Tag::Short(TAG_PHYSICAL_MINIMUM) => {
                    self.global.physical_minimum =
                        Some(i32::from_bytes(&data).ok_or(Error::BadPhysicalMinimum)?)
                }
                Tag::Short(TAG_PHYSICAL_MAXIMUM) => {
                    self.global.physical_maximum =
                        Some(i32::from_bytes(&data).ok_or(Error::BadPhysicalMaximum)?)
                }
                Tag::Short(TAG_INPUT) => {
                    self.collection.reports.push(Report::try_from_parser_states(
                        ReportType::Input,
                        &self.global,
                        &self.collection.state,
                    )?)
                }
                Tag::Short(TAG_OUTPUT) => {
                    self.collection.reports.push(Report::try_from_parser_states(
                        ReportType::Output,
                        &self.global,
                        &self.collection.state,
                    )?)
                }
                Tag::Short(TAG_FEATURE) => {
                    self.collection.reports.push(Report::try_from_parser_states(
                        ReportType::Feature,
                        &self.global,
                        &self.collection.state,
                    )?)
                }
                Tag::Short(TAG_PUSH) => self.stack_global.push(self.global.clone()),
                Tag::Short(TAG_POP) => {
                    self.global = self.stack_global.pop().ok_or(Error::PopWithoutPush)?
                }

                Tag::Short(x) | Tag::Long(x) => {
                    log::warn!("skipping unknown or unsupported tag {x:#x} with data {data:?}")
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{Parser, Report, ReportType};

    const REPORT_DESCRIPTOR1: &[u8] = &[
        0x05, 0x01, 0x09, 0x06, 0xa1, 0x01, 0x85, 0x01, 0x05, 0x07, 0x19, 0xe0, 0x29, 0xe7, 0x15,
        0x00, 0x25, 0x01, 0x75, 0x01, 0x95, 0x08, 0x81, 0x02, 0x05, 0x07, 0x19, 0x00, 0x29, 0x97,
        0x15, 0x00, 0x25, 0x01, 0x75, 0x01, 0x96, 0x98, 0x00, 0x81, 0x02, 0xc0, 0x05, 0x01, 0x09,
        0x80, 0xa1, 0x01, 0x85, 0x02, 0x19, 0x00, 0x29, 0xb7, 0x15, 0x00, 0x26, 0xb7, 0x00, 0x95,
        0x01, 0x75, 0x08, 0x81, 0x00, 0xc0, 0x05, 0x0c, 0x09, 0x01, 0xa1, 0x01, 0x85, 0x03, 0x1a,
        0x00, 0x00, 0x2a, 0x3c, 0x02, 0x15, 0x00, 0x26, 0x3c, 0x02, 0x75, 0x10, 0x95, 0x01, 0x81,
        0x00, 0xc0, 0x06, 0x52, 0xff, 0x0a, 0x10, 0x02, 0xa1, 0x01, 0x85, 0x07, 0x19, 0x01, 0x29,
        0x3f, 0x15, 0x00, 0x26, 0xff, 0x00, 0x75, 0x08, 0x95, 0x3f, 0x81, 0x00, 0x19, 0x01, 0x29,
        0x3f, 0x15, 0x00, 0x26, 0xff, 0x00, 0x75, 0x08, 0x95, 0x3f, 0xb1, 0x02, 0xc0,
    ];

    #[test]
    fn test1() {
        let descriptor = Parser::default().parse(REPORT_DESCRIPTOR1).unwrap();

        assert_eq!(descriptor.main_collection.reports.len(), 0);
        assert_eq!(descriptor.main_collection.nested.len(), 4);

        let nested = descriptor.main_collection.nested;

        assert_eq!(nested[0].nested.len(), 0);
        assert_eq!(nested[0].reports.len(), 2);

        assert_eq!(
            nested[0].reports[0],
            Report {
                r#type: ReportType::Input,
                usage_page: 0x07,
                usage: 0x06,
                usage_minimum: Some(0xE0),
                usage_maximum: Some(0xE7),
                report_size: 1,
                report_id: 1,
                report_count: 8,
                logical_minimum: 0,
                logical_maximum: 1,
                physical_minimum: None,
                physical_maximum: None,
            }
        );
        assert_eq!(
            nested[0].reports[1],
            Report {
                r#type: ReportType::Input,
                usage_page: 0x07,
                usage: 0x06,
                usage_minimum: Some(0x00),
                usage_maximum: Some(0x97),
                report_size: 1,
                report_id: 1,
                report_count: 152,
                logical_minimum: 0,
                logical_maximum: 1,
                physical_minimum: None,
                physical_maximum: None,
            }
        );

        assert_eq!(nested[1].nested.len(), 0);
        assert_eq!(nested[1].reports.len(), 1);

        assert_eq!(
            nested[1].reports[0],
            Report {
                r#type: ReportType::Input,
                usage_page: 0x01,
                usage: 0x80,
                usage_minimum: Some(0x00),
                usage_maximum: Some(0xB7),
                report_size: 8,
                report_id: 2,
                report_count: 1,
                logical_minimum: 0,
                logical_maximum: 183,
                physical_minimum: None,
                physical_maximum: None,
            }
        );

        assert_eq!(nested[2].nested.len(), 0);
        assert_eq!(nested[2].reports.len(), 1);

        assert_eq!(
            nested[2].reports[0],
            Report {
                r#type: ReportType::Input,
                usage_page: 0x0C,
                usage: 0x01,
                usage_minimum: Some(0x00),
                usage_maximum: Some(0x023C),
                report_size: 16,
                report_id: 3,
                report_count: 1,
                logical_minimum: 0x00,
                logical_maximum: 0x023C,
                physical_minimum: None,
                physical_maximum: None,
            }
        );

        assert_eq!(nested[3].nested.len(), 0);
        assert_eq!(nested[3].reports.len(), 2);

        assert_eq!(
            nested[3].reports[0],
            Report {
                r#type: ReportType::Input,
                usage_page: 0xFF52,
                usage: 0x0210,
                usage_minimum: Some(0x01),
                usage_maximum: Some(0x3F),
                report_size: 8,
                report_id: 7,
                report_count: 63,
                logical_minimum: 0x00,
                logical_maximum: 0xFF,
                physical_minimum: None,
                physical_maximum: None,
            }
        );
        assert_eq!(
            nested[3].reports[1],
            Report {
                r#type: ReportType::Feature,
                usage_page: 0xFF52,
                usage: 0x0210,
                usage_minimum: Some(0x01),
                usage_maximum: Some(0x3F),
                report_size: 8,
                report_id: 7,
                report_count: 63,
                logical_minimum: 0x00,
                logical_maximum: 0xFF,
                physical_minimum: None,
                physical_maximum: None,
            }
        );
    }
}
