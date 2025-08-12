
pub mod types {
    use std::collections::HashMap;
    use chrono::NaiveDate;

    pub type CommitDict = HashMap<NaiveDate, u32>;
}
