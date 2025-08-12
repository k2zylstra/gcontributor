
pub struct Committer {
    origin: String,
}

impl Committer {
    pub fn new(origin: String) -> Self {
        Committer {
            origin,
        }
    }

    pub fn commit(&self, count:u8) -> Option<()> {
        Some(())
    }
}