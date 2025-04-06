use crate::db;
use crate::AppMessage;
use askama::Template;

#[derive(Template, Default)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    nickname: String,
    my: Vec<db::Device>,
    unassinged: Vec<db::AliveDevice>,
    messages: Vec<AppMessage>,
}

impl IndexTemplate {
    pub fn new(
        nickname: String,
        my: Vec<db::Device>,
        unassinged: Vec<db::AliveDevice>,
        messages: Vec<AppMessage>,
    ) -> Self {
        Self {
            nickname,
            my,
            unassinged,
            messages,
        }
    }
}
