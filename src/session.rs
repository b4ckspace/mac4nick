use crate::Message;
use crate::Request;
use serde::{Deserialize, Serialize};

pub struct AppSession<'r> {
    request: &'r mut Request,
    session: Inner,
}

impl<'r> AppSession<'r> {
    pub fn add_message(&mut self, message: Message) {
        self.session.add_message(message);
    }

    pub fn pop_messages(&mut self) -> Vec<Message> {
        self.session.pop_messages()
    }
}

impl<'r> From<&'r mut Request> for AppSession<'r> {
    fn from(request: &'r mut Request) -> Self {
        let session = Inner::from(&*request);
        Self { request, session }
    }
}

impl<'r> Drop for AppSession<'r> {
    fn drop(&mut self) {
        self.session.commit(self.request);
    }
}

#[derive(Default, Deserialize, Serialize)]
struct Inner {
    messages: Vec<Message>,
}

impl Inner {
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn pop_messages(&mut self) -> Vec<Message> {
        let mut messages: Vec<Message> = Vec::new();
        std::mem::swap(&mut messages, &mut self.messages);
        messages
    }

    pub fn commit(&mut self, request: &mut Request) {
        request.session_mut().insert("app", self).unwrap()
    }
}

impl From<&Request> for Inner {
    fn from(request: &Request) -> Self {
        request.session().get("app").unwrap_or_default()
    }
}
