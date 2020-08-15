use crate::db;
use askama::Template;

#[derive(Template, Default)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    my: Vec<db::Entry>,
    unassinged: Vec<db::AliveDevice>,
}

impl IndexTemplate {
    pub fn new(my: Vec<db::Entry>, unassinged: Vec<db::AliveDevice>) -> Self {
        Self { my, unassinged }
    }
}
