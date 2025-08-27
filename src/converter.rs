
pub trait Converter {
    fn convert(&self) -> &'static str;
}
