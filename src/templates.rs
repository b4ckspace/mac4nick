use crate::db;
use askama::Template;

#[derive(Template, Default)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    devices: Vec<Device<'a>>,
}

pub struct Device<'a> {
    pub macaddr: &'a str,
    pub nickname: &'a str,
    pub descr: &'a str,
    pub privacy: PrivacyLevel,
}

#[repr(u8)]
pub enum PrivacyLevel {
    Public = 1,
    Private = 2,
    Internal = 3,
}

impl<'a> IndexTemplate<'a> {
    pub fn new() -> Self {
        Self::default()
    }
}
