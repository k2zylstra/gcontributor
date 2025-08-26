use thiserror;
use std::borrow::Cow;
use std::io;

#[derive(thiserror::Error, Debug)]
pub enum SQLDataError {
    #[error("Unable to find {item} with the name: {name} in the table: {table}{notes}")]
    ErrNoEnt {
        item: &'static str,
        name: String,
        table: &'static str,
        notes: Cow<'static, str>,
    },
    #[error("Unable to set {item} with name: {name} in the table: {table}{notes}")]
    ErrNoSet {
        item: &'static str,
        name: String,
        table: &'static str,
        notes: Cow<'static, str>,
    },
    #[error(transparent)]
    Sql(#[from] sqlite::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl SQLDataError {
    pub fn not_found(item: &'static str, name: impl Into<String>, table: &'static str) -> Self {
        SQLDataError::ErrNoEnt { item: item, name: name.into(), table: table, notes: Cow::Borrowed("")}
    }

    pub fn not_found_with(item: &'static str, name: impl Into<String>, table: &'static str, notes: impl Into<String>) -> Self {
        SQLDataError::ErrNoEnt { item: item, name: name.into(), table: table, notes: Cow::Owned(notes.into()) }
    }

    pub fn not_set(item: &'static str, name: impl Into<String>, table: &'static str) -> Self {
        SQLDataError::ErrNoSet { item: item, name: name.into(), table: table, notes: Cow::Borrowed("")}
    }

    pub fn not_set_with(item: &'static str, name: impl Into<String>, table: &'static str, notes: impl Into<String>) -> Self {
        SQLDataError::ErrNoSet { item: item, name: name.into(), table: table, notes: Cow::Owned(notes.into()) }
    }
}

