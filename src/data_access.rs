use gcontributor::types::CommitDict;
use chrono::NaiveDate;

#[derive(Default)]
pub struct DataAccessor {
    db_location: String,
}

impl DataAccessor {
    pub fn new(db_location: String) -> Self {
        DataAccessor {
            db_location
        }
    }

    pub fn create_dict(&self, commits: CommitDict) -> Option<()> {
        Some(())
    }

    pub fn read_date(&self, date: NaiveDate) -> Option<u8> {
        Some(8)
    }

    pub fn get_status(&self) -> Option<bool> {
        Some(false)
    }

    pub fn has_run(&self, date: NaiveDate) -> Option<bool> {
        Some(false)
    }

    pub fn log_run(&self, date: NaiveDate) -> Option<()> {
        Some(())
    }
}