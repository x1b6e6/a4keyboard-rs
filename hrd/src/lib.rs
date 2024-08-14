pub mod descriptor;
pub mod parser;
pub mod report;

pub use descriptor::Descriptor;
pub use parser::Parser;
pub use report::Collection;
pub use report::Report;
pub use report::ReportType;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unexpected tag `Collection End")]
    UnexpectedEndCollection,
    #[error("unexpected end of report descriptor")]
    UnexpectedEndOfReportDescriptor,

    #[error("bad `Usage Page` tag")]
    BadUsagePage,
    #[error("bad `Usage` tag")]
    BadUsage,
    #[error("bad `Usage Minimum` tag")]
    BadUsageMinimum,
    #[error("bad `Usage Maximum` tag")]
    BadUsageMaximum,
    #[error("bad `Report Size` tag")]
    BadReportSize,
    #[error("bad `Report ID` tag")]
    BadReportId,
    #[error("bad `Report Count` tag")]
    BadReportCount,
    #[error("bad `Logical Minimum` tag")]
    BadLogicalMinimum,
    #[error("bad `Logical Maximum` tag")]
    BadLogicalMaximum,
    #[error("bad `Physical Minimum` tag")]
    BadPhysicalMinimum,
    #[error("bad `Physical Maximum` tag")]
    BadPhysicalMaximum,
    #[error("bad `Collection` tag")]
    BadCollection,

    #[error("tag `Usage Page` is not set")]
    UsagePageNotSet,
    #[error("tag `Usage` is not set")]
    UsageNotSet,
    #[error("tag `Report Size` is not set")]
    ReportSizeNotSet,
    #[error("tag `Report Count` is not set")]
    ReportCountNotSet,
    #[error("tag `Logical Minimum` is not set")]
    LogicalMinimumNotSet,
    #[error("tag `Logical Maximum` is not set")]
    LogicalMaximumNotSet,

    #[error("missing `End Collection` tag")]
    MissingEndCollection,
    #[error("missing pop")]
    MissingPop,
    #[error("pop without push")]
    PopWithoutPush,
}

pub fn parse(data: &[u8]) -> Result<Descriptor, Error> {
    Parser::default().parse(data)
}
