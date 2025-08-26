pub mod error;


pub mod types {
    use std::collections::HashMap;
    use chrono::NaiveDate;
    use crate::error::SQLDataError;

    pub type SqlResult<T> = std::result::Result<T, SQLDataError>;

    pub type CommitDict<'a> = HashMap<&'a NaiveDate, u32>;
}
