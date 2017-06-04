extern crate notify_rust;

use notify_rust::{Notification, NotificationHandle, NotificationHint};
use notify_rust::Error;
use std::str;

const ICON: &'static str = "thunderbird-bin-icon";
const APPNAME: &'static str = "stream_notification";
const TIMEOUT: i32 = 20000;

pub struct TweetObject {
    pub user_screen_name: String,
    pub retweeted_status_user_screen_name: Option<String>,
    pub text: String,
}

pub trait NotificationFmt {
    fn fmt_summary(&self) -> String;
    fn fmt_body(&self) -> String;
}

impl NotificationFmt for TweetObject {
    fn fmt_summary(&self) -> String {
        println!("self.retweeted_status_user_screen_name: {:?}", self.retweeted_status_user_screen_name);
        if let Some(ref retweeted_status) = self.retweeted_status_user_screen_name {
            return format!("retweet by @{} from @{}",
                           retweeted_status,
                           self.user_screen_name)
        }
        format!("tweet by @{}", self.user_screen_name)
    }

    fn fmt_body(&self) -> String {
        format!("{}", self.text)
    }
}

pub fn new(obj: TweetObject) -> Result<NotificationHandle, Error> {
    Notification::new()
        .summary(obj.fmt_summary().as_ref())
        .body(obj.fmt_body().as_ref())
        .icon(ICON)
        .appname(APPNAME)
        .action("clicked", "↰")
        .action("clicked", "⇄")
        .action("clicked", "♥")
        .action("view", "View")
        .hint(NotificationHint::Category("email".to_owned()))
        .timeout(TIMEOUT)
        .show_debug()
}
