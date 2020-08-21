use crate::db;
use crate::Message;
use askama::Template;

#[derive(Template, Default)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    nickname: String,
    my: Vec<db::Device>,
    unassinged: Vec<db::AliveDevice>,
    messages: Vec<Message>,
}

impl IndexTemplate {
    pub fn new(
        nickname: String,
        my: Vec<db::Device>,
        unassinged: Vec<db::AliveDevice>,
        messages: Vec<Message>,
    ) -> Self {
        Self {
            nickname,
            my,
            unassinged,
            messages,
        }
    }
}
