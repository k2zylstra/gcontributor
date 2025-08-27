//! Testing this ichiban
//! 
mod committer;
mod converter;
mod converters {
    pub mod jpeg_converter;
}
mod data_access;

use chrono::{Local, NaiveTime, TimeZone};
use std::thread;
use committer::*;
use converter::*;
use data_access::*;

use crate::converters::jpeg_converter::JpegConverter;

pub struct FlowControl<C: Converter> {
    converter: C,
    committer: Committer,
    data_accessor: DataAccessor,
    schedule_time: String,
}

impl<C: Converter> FlowControl<C> {
    pub fn new(converter: C, committer:Committer, data_accessor:DataAccessor) -> Self {
        FlowControl {
            committer,
            converter,
            data_accessor,
            schedule_time: "21:00".to_string(),
        }
    }

    pub fn run(&self) -> () {
        let mut now = Local::now();
        let mut today = now.date_naive();
        if ! self.data_accessor.get_status().unwrap() {
            self.setup_plan();
        };
        if ! self.data_accessor.has_run(today).unwrap() {
            self.run_commit();
        };

        let at = NaiveTime::parse_from_str(&self.schedule_time, "%H:%M").unwrap();
        loop {
            now = Local::now();
            today = now.date_naive();

            let target_today = Local.from_local_datetime(&today.and_time(at)).single().unwrap();
            let next = if target_today > now {
                target_today
            } else {
                let tomorrow = today.succ_opt().unwrap();
                Local.from_local_datetime(&tomorrow.and_time(at)).single().unwrap()
            };

            let wait = (next - now).to_std().unwrap();
            thread::sleep(wait);
            self.run_commit();
        }
    }

    fn run_commit(&self) -> Option<()> {

        Some(())
    }

    fn setup_plan(&self) -> Option<()> {
        Some(())
    }
}

fn main() {
    let com: Committer = Committer::new("Test2".to_string());
    let conv = JpegConverter::new();
    conv.convert();
    let da: DataAccessor = DataAccessor::new().unwrap();
    let fc: FlowControl = FlowControl::new(conv, com, da);
    fc.run();
    println!("Hello, world!");
}
