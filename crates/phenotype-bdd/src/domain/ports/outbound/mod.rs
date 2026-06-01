use crate::domain::entities::FeatureResult;
use crate::BddError;

pub trait ReportWriterPort {
    fn write_feature_report(&self, result: &FeatureResult) -> Result<(), BddError>;
    fn format(&self) -> &str;
    fn flush(&self) -> Result<(), BddError>;
}
