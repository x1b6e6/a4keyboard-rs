use crate::parser::ParserCollection;
use crate::parser::ParserGlobalState;
use crate::parser::ParserLocalState;
use crate::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum ReportType {
    Input,
    Output,
    Feature,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    pub r#type: ReportType,
    pub usage_page: u32,
    pub usage: u32,
    pub usage_minimum: Option<u32>,
    pub usage_maximum: Option<u32>,
    pub report_size: u32,
    pub report_id: u32,
    pub report_count: u32,
    pub logical_minimum: i32,
    pub logical_maximum: i32,
    pub physical_minimum: Option<i32>,
    pub physical_maximum: Option<i32>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Collection {
    pub reports: Vec<Report>,
    pub nested: Vec<Collection>,
}

impl Collection {
    pub fn iter(&self) -> Box<dyn Iterator<Item = &Report> + '_> {
        Box::new(
            self.reports
                .iter()
                .chain(self.nested.iter().map(Collection::iter).flatten()),
        )
    }
}

impl Report {
    pub(crate) fn try_from_parser_states(
        r#type: ReportType,
        global: &ParserGlobalState,
        local: &ParserLocalState,
    ) -> Result<Self, Error> {
        Ok(Report {
            r#type,
            usage_page: global.usage_page.ok_or(Error::UsagePageNotSet)?,
            usage: local.usage.ok_or(Error::UsageNotSet)?,
            usage_minimum: local.usage_minimum,
            usage_maximum: local.usage_maximum,
            report_size: global.report_size.ok_or(Error::ReportSizeNotSet)?,
            report_id: global.report_id.unwrap_or(1),
            report_count: global.report_count.ok_or(Error::ReportCountNotSet)?,
            logical_minimum: global.logical_minimum.ok_or(Error::LogicalMinimumNotSet)?,
            logical_maximum: global.logical_maximum.ok_or(Error::LogicalMaximumNotSet)?,
            physical_minimum: global.physical_minimum,
            physical_maximum: global.physical_maximum,
        })
    }
}

impl From<ParserCollection> for Collection {
    fn from(collection: ParserCollection) -> Self {
        Collection {
            reports: collection.reports,
            nested: collection.collections.into_iter().map(From::from).collect(),
        }
    }
}
