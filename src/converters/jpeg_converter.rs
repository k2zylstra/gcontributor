use crate::converter::Converter;

pub struct JpegConverter {
  test: u8,
}

impl JpegConverter {
  pub fn new() -> Self {
    JpegConverter { test: 8 }
  }
}

impl Converter for JpegConverter {
  fn convert(&self) -> &'static str {"jpeg"}
}