//! DataAccessor provides persistent storage
//!
//! This creates an sqlite db that holds two tables. One to hold user information like username and
//! repo to commit to, and the other holds essentially key value pairs of a mapping between dates
//! and the number of commits required to build the desired image on that date.

use sqlite;
use chrono::{Local, NaiveDate};
use std::path::{Path, PathBuf};
use std::{fs, vec::Vec};

use gcontributor::types::CommitDict;
use gcontributor::error::SQLDataError;
use gcontributor::types::SqlResult;

/// DataAccessor struct holds a Path to where the database will be stored.
/// By default this is stored in resources/gcontrib.db
pub struct DataAccessor {
    db_location: PathBuf,
    timeout: usize,
}
/// Data Accessor implementation containing the operations for interacting with the persistent
/// storage db
impl DataAccessor {
    pub const DEFAULT_DB_NAME: &'static str = "gcontrib.db";
    pub const DEFAULT_DB_PATH: &'static str = "resources";
    pub const DEFAULT_MS_TIMEOUT: usize = 1_000;

    /// MAX retries on a busy connection with the database
    const MAX_BUSY_RETRIES: usize = 5;
    /// The multiplier at which the busy retry waits in between connection attempts to the db
    const BUSY_WAIT_MULT_MS: u64 = 100;

    const USERS_USERNAME_INDEX:usize = 0;
    const USERS_REPO_INDEX:usize = 1;
    const COMMIT_COUNT_INDEX:usize = 1;
    const COMMIT_ISRUN_INDEX:usize = 2;
    //const COMMIT_DATE_INDEX:usize = 0;

    /// Creates the user table
    pub const QCREATE_USER_T: &'static str = "
        CREATE TABLE IF NOT EXISTS users ( 
            name TEXT PRIMARY KEY NOT NULL
            ,repo TEXT NOT NULL
        );";

    /// Creates the date to commit count table
    pub const QCREATE_COMMIT_T: &'static str = "
        CREATE TABLE IF NOT EXISTS commit_plan (
            date TEXT PRIMARY KEY
            ,commit_count INTEGER NOT NULL
            ,is_run INTEGER NOT NULL
        );";

    /// Adds a user to the users table
    pub const QADD_USER: &'static str = "
        INSERT INTO users(name, repo)
        VALUES(
            :username,
            :repo_uri
        )
        RETURNING name;";

    /// Adds a commit count tied to the day
    pub const QADD_COMMIT: &'static str = "
        INSERT INTO commit_plan(date, commit_count, is_run)
        VALUES(
            :date,
            :commit_count,
            :is_run
        );";

    /// Gets the data associated with a specific date from the commit_plan table
    pub const QGET_DATE_DATA: &'static str = "
        SELECT
            date
            ,commit_count
            ,is_run
        FROM commit_plan WHERE
            date = :date
        ;";

    /// Updates the is_run field for a given date
    pub const QUPDATE_COMMIT_DATE: &'static str = "
        UPDATE commit_plan
        SET is_run = :is_run
        WHERE
            date = :date 
        RETURNING date;";

    /// Gets the repo url associated with a specific username
    pub const QGET_REPO_URL: &'static str = "
        SELECT 
            name,
            repo
        FROM users WHERE
            name = :username
        ;";

    /// Gets the list of users from the database
    pub const QGET_USERS: &'static str = "
        SELECT name FROM users
        ;";

    /// Constructs a new DataAccessor and sets up tables at the default location
    pub fn new() -> SqlResult<Self> {
        let da = DataAccessor {
            db_location: Path::new(Self::DEFAULT_DB_PATH).to_path_buf().join(Self::DEFAULT_DB_NAME),
            timeout: Self::DEFAULT_MS_TIMEOUT,
        };
        da.setup_tables()?;
        Ok(da)
    }

    /// Provides the ability to create a DataAccessor with a defined Path
    pub fn with_db<P: AsRef<Path>>(path: P) -> SqlResult<Self> {
        // TODO understand if this is copying the data
        let da = DataAccessor {
            db_location: path.as_ref().to_path_buf(),
            timeout: Self::DEFAULT_MS_TIMEOUT,
        };
        da.setup_tables()?;
        Ok(da)
    }

    /// Actually creates the database and tables within
    pub fn setup_tables(&self) -> SqlResult<()> {
        self.create_ifnot_parent_dir()?;
        self.create_commit_t()?;
        self.create_user_t()?;
        Ok(())
    }

    /// Returns the current database location as a path on the filesystem
    pub fn db_path(&self) -> &Path {
        &self.db_location
    }

    /// Adds a commit plan to the commit table
    pub fn add_commit_plan(&self, commits: &CommitDict) -> SqlResult<()> {
        let conn = self.setup_connection()?;
        conn.execute("BEGIN IMMEDIATE;")?;
        { // this scope is added to ensure the stmt drop is finalized
            let mut stmt = conn.prepare(Self::QADD_COMMIT)?;
            for (&date, &count) in commits {
                let date_str = date.to_string();
                stmt.bind((":date", date_str.as_str()))?;
                stmt.bind((":commit_count", count as i64))?;
                stmt.bind((":is_run", 0))?;
                stmt.next()?;
                stmt.reset()?;
            }
        }
        conn.execute("COMMIT;")?;
        Ok(())
    }

    /// Returns a commit count for the specific date given
    pub fn get_date_count(&self, date: NaiveDate) -> SqlResult<u32> {
        let conn = self.setup_connection()?;
        let mut stmt = conn.prepare(Self::QGET_DATE_DATA)?;
        stmt.bind((":date", date.to_string().as_str()))?;
        match stmt.next()? {
            sqlite::State::Row => {
                let res = stmt.read::<i64, _>(Self::COMMIT_COUNT_INDEX)? as u32;
                return Ok(res);
            },
            sqlite::State::Done => {
                Err(SQLDataError::not_found("count", date.to_string(), "commit_plan"))
            }
        }
    }

    /// Returns true if there is an current commit plan stored and false otherwise
    pub fn get_status(&self) -> SqlResult<bool> {
        // TODO include user data in this to make sure there is a repository to commit to
        let now = Local::now();
        let date = now.date_naive();
        let conn = self.setup_connection()?;

        let mut stmt = conn.prepare(Self::QGET_DATE_DATA)?;
        stmt.bind((":date", date.to_string().as_str()))?;

        match stmt.next()? {
            sqlite::State::Row => {
                return Ok(true)
            },
            sqlite::State::Done => {
                return Ok(false)
            }
        }
    }

    /// Returns the repository currently stored as the upstream commit destitination
    pub fn get_repo(&self, username: &str) -> SqlResult<String> {
        let conn = self.setup_connection()?;

        let mut stmt = conn.prepare(Self::QGET_REPO_URL)?;
        stmt.bind((":username", username))?;

        match stmt.next()? {
            sqlite::State::Row => {
                return Ok(stmt.read::<String, _>(Self::USERS_REPO_INDEX)?);
            },
            sqlite::State::Done => {
                Err(SQLDataError::not_found("url", username, "users"))
            }
        }
    }

    /// Returns an Array of usernames
    pub fn get_users(&self) -> SqlResult<Vec<String>> {
        let conn = self.setup_connection()?;
        let mut stmt = conn.prepare(Self::QGET_USERS)?;
        let mut usernames: Vec<String> = Vec::new();

        loop {
            match stmt.next()? {
                sqlite::State::Done => {
                    break;
                }
                sqlite::State::Row => {
                    usernames.push(stmt.read::<String, _>(Self::USERS_USERNAME_INDEX)?);
                }
            }
        }
        Ok(usernames)
    }

    /// Returns true if a commit plan has ran for a specific date and false otherwise
    pub fn has_run(&self, date: NaiveDate) -> SqlResult<bool> {
        let conn = self.setup_connection()?;
        let mut stmt = conn.prepare(Self::QGET_DATE_DATA)?;
        stmt.bind((":date", date.to_string().as_str()))?;

        if let sqlite::State::Row = stmt.next()? {
            if stmt.read::<i64, _>(Self::COMMIT_ISRUN_INDEX)? == 1 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Writes to the database that a commit count was run for a specific date
    pub fn set_run(&self, date: NaiveDate) -> SqlResult<()> {
        let conn = self.setup_connection()?;
        let mut stmt = conn.prepare(Self::QUPDATE_COMMIT_DATE)?;
        stmt.bind((":is_run", 1))?;
        stmt.bind((":date", date.to_string().as_str()))?;
        match stmt.next()? {
            sqlite::State::Done => {
                let date_str = date.to_string();
                return Err(SQLDataError::not_set_with("is_run",
                    "1",
                    "commit_plan",
                    format!("for the date: {date_str}"
                )))
            },
            sqlite::State::Row => {
                Ok(())
            }
        }
    }

    /// Add User and related info
    pub fn add_user_info(&self, username: &str, repo: &str) -> SqlResult<()> {
        let conn = self.setup_connection()?;
        let mut stmt = conn.prepare(Self::QADD_USER)?;
        stmt.bind((":username", username))?;
        stmt.bind((":repo_uri", repo))?;

        match stmt.next()? {
            sqlite::State::Row => {
                Ok(())
            },
            sqlite::State::Done => {
                return Err(SQLDataError::not_set("username and repo", format!("{username} and {repo}"), "users"));
            }
        }
    }

    /// returns a connection the database with a preset timeout and handler
    fn setup_connection(&self) -> SqlResult<sqlite::Connection> {
        let mut conn = sqlite::open(&self.db_location)?;
        conn.set_busy_timeout(self.timeout)?;
        conn.set_busy_handler(Self::db_busy_handler)?;
        Ok(conn)
    }

    /// Defines a handler in case the database is busy and sets to retry the connection
    fn db_busy_handler(retry_num: usize) -> bool {
        if retry_num > Self::MAX_BUSY_RETRIES {
            eprintln!("DB busy: exceeded {} retries", Self::MAX_BUSY_RETRIES);
            return false;
        }
        let timeout_dur = (retry_num as u64) * Self::BUSY_WAIT_MULT_MS;
        std::thread::sleep(std::time::Duration::from_millis(timeout_dur));
        true
    }

    /// Generates the parent directories of the database
    fn create_ifnot_parent_dir(&self) -> SqlResult<()> {
        if let Some(parent) = self.db_location.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        Ok(())
    }

    /// Creates the commit table
    fn create_commit_t(&self) -> SqlResult<()> {
        let conn = self.setup_connection()?;
        let mut stmt = conn.prepare(Self::QCREATE_COMMIT_T)?;
        stmt.next()?;
        Ok(())
    }

    /// Creates the user table
    fn create_user_t(&self) -> SqlResult<()> {
        let conn = self.setup_connection()?;
        let mut stmt = conn.prepare(Self::QCREATE_USER_T)?;
        stmt.next()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use sqlite::State;

    /// Tests to see if a give table name exists in an sqlite database as provided
    /// by a connection object
    fn table_exists(conn: &sqlite::Connection, name: &str) -> bool {
        let mut stmt = conn
            .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name=:name")
            .unwrap();
        stmt.bind((":name", name)).unwrap();
        matches!(stmt.next().unwrap(), State::Row)
    }

    /// Tests to see if a commit plan has been written to a database by seeing if there is at least
    /// one row that was written to the commit table
    fn commit_plan_written(conn: &sqlite::Connection) -> u32 {
        const COUNT_INDEX: usize = 0;
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM commit_plan")
            .unwrap();
        let count = match stmt.next().unwrap() {
            State::Done => {
                0
            },
            State::Row => {
                stmt.read::<i64, _>(COUNT_INDEX).unwrap()
            }
        };
        count as u32
    }

    /// Tests to see if a date is marked as a run given a specific date and connection
    fn date_has_run(conn: &sqlite::Connection, date: NaiveDate) -> bool {
        let mut stmt = conn
            .prepare("SELECT * FROM commit_plan where date=:date")
            .unwrap();
        stmt.bind((":date", date.to_string().as_str())).unwrap();
        match stmt.next().unwrap() {
            State::Row => {
                let has_run = stmt.read::<i64, _>(2).unwrap();
                has_run != 0
            }
            State::Done => false,
        }
    }

    /// Creates a random tempdirection and db file to be used for the DataAccessor
    fn create_random_db_loc() -> Option<PathBuf> {
        let dir = tempfile::tempdir().unwrap();
        let t_nano = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let db_path = dir.path().join(format!("gcontrib{t_nano}.db"));

        Some(db_path)
    }


    #[test]
    fn test_create_commit_table() {
        let db_loc = create_random_db_loc().unwrap();
        let da = DataAccessor::with_db(db_loc).unwrap();

        let conn = sqlite::open(da.db_path()).unwrap();
        assert!(table_exists(&conn, "commit_plan"));
    }

    #[test]
    fn test_create_user_table() {
        let db_loc = create_random_db_loc().unwrap();
        let da = DataAccessor::with_db(db_loc).unwrap();
        let conn = sqlite::open(da.db_path()).unwrap();
        assert!(table_exists(&conn, "users"));
    }

    #[test]
    fn test_set_run() {
        // TODO make this a set date
        let t = Local::now();
        let day = t.date_naive();

        let db_loc = create_random_db_loc().unwrap();
        let da = DataAccessor::with_db(db_loc).unwrap();


        let cd = CommitDict::from([(&day, 3)]);
        da.add_commit_plan(&cd).unwrap();

        let conn = sqlite::open(da.db_path()).unwrap();
        assert!(!date_has_run(&conn, day));
        da.set_run(day).unwrap();
        assert!(date_has_run(&conn, day));
        
    }

    #[test]
    fn test_write_plan() {
        let day1 = NaiveDate::from_ymd_opt(1980, 4, 2).unwrap();
        let day2 = NaiveDate::from_ymd_opt(1980, 4, 3).unwrap();
        let day3 = NaiveDate::from_ymd_opt(1980, 4, 4).unwrap();

        let cd = CommitDict::from([
            (&day1, 8),
            (&day2, 3),
            (&day3, 10),
        ]);

        let db_loc = create_random_db_loc().unwrap();
        let da = DataAccessor::with_db(db_loc).unwrap();
        da.add_commit_plan(&cd).unwrap();

        let conn = sqlite::open(da.db_path()).unwrap();
        assert_eq!(commit_plan_written(&conn), 3);
    }

    #[test]
    fn test_get_repository() {
        let db_loc = create_random_db_loc().unwrap();
        let da = DataAccessor::with_db(db_loc).unwrap();
        let uri: &'static str = "uri://this.is.a.fake.address";
        da.add_user_info("testuser1", uri).unwrap();
        let users = da.get_users().unwrap();
        assert!(users.len() > 0);
        let repo_uri = da.get_repo(users[0].as_str()).unwrap();
        assert_eq!(repo_uri.as_str(), uri);
    }

    #[test]
    #[should_panic]
    fn test_fail_no_user() {
        let db_loc = create_random_db_loc().unwrap();
        let da = DataAccessor::with_db(db_loc).unwrap();
        println!("{}", da.get_users().unwrap()[0]);
    }
}
// TODO add functionality that on adding user info the repository is tested for connection (should go in committer)
