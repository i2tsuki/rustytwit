extern crate curl;
extern crate crypto;
extern crate egg_mode;
extern crate rustc_serialize;
extern crate time;

use gtk;
use gtk::{Image, Label};
use gtk::{Orientation, RevealerTransitionType};
use gtk::prelude::*;
use regex;
use std::clone::Clone;

// TimelineError
#[derive(Debug)]
pub enum TimelineError {
    CreateWidget(CreateWidgetError),
    String(String),
    Widget(gtk::Widget),
}

impl From<CreateWidgetError> for TimelineError {
    fn from(err: CreateWidgetError) -> TimelineError {
        TimelineError::CreateWidget(err)
    }
}

impl From<String> for TimelineError {
    fn from(err: String) -> TimelineError {
        TimelineError::String(err)
    }
}

impl From<gtk::Widget> for TimelineError {
    fn from(err: gtk::Widget) -> TimelineError {
        TimelineError::Widget(err)
    }
}

// CreateWidgetError
#[derive(Debug)]
pub enum CreateWidgetError {
    Regex(regex::Error),
    Time(time::ParseError),
    ReadCache(::utils::UtilsError),
}

impl From<regex::Error> for CreateWidgetError {
    fn from(err: regex::Error) -> CreateWidgetError {
        CreateWidgetError::Regex(err)
    }
}

impl From<time::ParseError> for CreateWidgetError {
    fn from(err: time::ParseError) -> CreateWidgetError {
        CreateWidgetError::Time(err)
    }
}

impl From<::utils::UtilsError> for CreateWidgetError {
    fn from(err: ::utils::UtilsError) -> CreateWidgetError {
        CreateWidgetError::ReadCache(err)
    }
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct TimelineRow {
    pub tweet: Tweet,
    pub unread: bool,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Tweet {
    pub created_at: String,
    pub id: u64,
    pub text: String,
    pub attr: String,
    pub user: User, // pub retweeted_status: RetweetedStatus,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct User {
    pub screen_name: String,
    pub profile_image_url: String,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct RetweetedStatus {
    pub id: i64,
    pub text: String,
    pub user: User,
    pub created_at: String,
}

pub fn fixup_home(timeline: &mut Vec<TimelineRow>, limit: usize) {
    while timeline.len() > limit {
        timeline.remove(limit);
    }
}

pub fn update_home(
    listbox: &gtk::ListBox,
    timeline: &Vec<TimelineRow>,
    add: bool,
    unread_filter: bool,
) -> Result<(), TimelineError> {
    // When add flag is false, refresh all listboxrow
    if !add {
        for widget in listbox.get_children() {
            listbox.remove(&widget);
        }
    }
    // いっぱいつぶやくユーザをフィルタ
    let muted_user = Some(None);
    let mut index: i32 = 0;

    for status in timeline {
        if unread_filter == true && status.unread == false {
            continue;
        }
        for muted in muted_user.clone().unwrap() {
            if status.tweet.user.screen_name == muted {
                break;
            } else if muted == muted_user.clone().unwrap().last().unwrap().to_string() {
                let listboxrow = gtk::ListBoxRow::new();
                let revealer = try!(create_revealer(status.clone()));
                listboxrow.add(&revealer);
                listbox.insert(&listboxrow, index);

                index += 1;
                try!(show_listboxrow(&listboxrow));
            }
        }
    }
    return Ok(());
}

pub fn create_revealer(row: TimelineRow) -> Result<gtk::Revealer, CreateWidgetError> {
    let create_box_header = move |tweet: Tweet| -> Result<gtk::Box, CreateWidgetError> {
        let user_label = Label::new(None);
        let user = format!("<b>{}</b>", tweet.attr);
        user_label.set_text(user.as_ref());
        user_label.set_selectable(true);
        user_label.set_use_markup(true);
        user_label.set_xalign(0.0);

        let created_at_label = Label::new(None);
        created_at_label.set_text(tweet.created_at.as_ref());
        let box_header = gtk::Box::new(Orientation::Horizontal, 2);
        box_header.pack_start(&user_label, true, true, 0);
        box_header.pack_start(&created_at_label, false, false, 0);

        return Ok(box_header);
    };

    let create_box_label = move |tweet: Tweet| -> Result<gtk::Box, CreateWidgetError> {
        let box_header = try!(create_box_header(tweet.clone()));

        let body = try!(::timeline::utils::format_tweet_body(&tweet.text));
        let label_body = Label::new(None);
        label_body.set_text(body.as_ref());
        label_body.set_selectable(true);
        label_body.set_use_markup(true);
        label_body.set_line_wrap(true);
        label_body.set_xalign(0.0);

        let box_label = gtk::Box::new(Orientation::Vertical, 2);
        box_label.pack_start(&box_header, false, false, 0);
        box_label.pack_start(&label_body, true, true, 0);

        return Ok(box_label);
    };

    let create_box_revealer = move |row: TimelineRow| -> Result<gtk::Box, CreateWidgetError> {
        println!("{}", &row.tweet.user.profile_image_url);
        let profile_image_filename = try!(::utils::get_profile_image(
            &row.tweet.user.profile_image_url,
        ));
        let image_profile_image = Image::new_from_file(profile_image_filename);
        image_profile_image.set_padding(4, 4);

        let box_label = try!(create_box_label(row.tweet.clone()));

        let image_unread = Image::new_from_icon_name("gtk-media-record", 1);
        if !row.unread {
            image_unread.clear();
            image_unread.set_padding(8, 8);
        };

        let label_null = Label::new(None);
        let null = "   ";
        label_null.set_text(null.as_ref());

        let label_id = Label::new(None);
        let id = format!("{}", row.tweet.id);
        label_id.set_text(id.as_ref());
        label_id.set_visible(false);

        let label_profile_image = Label::new(None);
        let profile_image_url = format!("{}", row.tweet.user.profile_image_url);
        label_profile_image.set_text(profile_image_url.as_ref());
        label_profile_image.set_visible(false);

        let box_revealer = gtk::Box::new(Orientation::Horizontal, 2);
        box_revealer.pack_start(&image_profile_image, false, false, 0);
        box_revealer.pack_start(&box_label, true, true, 0);
        box_revealer.pack_start(&image_unread, false, false, 0);
        box_revealer.pack_start(&label_null, false, false, 0);
        box_revealer.pack_start(&label_id, false, false, 0);
        box_revealer.pack_start(&label_profile_image, false, false, 0);

        return Ok(box_revealer);
    };

    let create_revealer = move |row: TimelineRow| -> Result<gtk::Revealer, CreateWidgetError> {
        // FixMe: revealer is not available
        let revealer = gtk::Revealer::new();
        revealer.set_transition_type(RevealerTransitionType::Crossfade);
        revealer.set_transition_duration(15000);
        revealer.set_reveal_child(true);

        let box_revealer = try!(create_box_revealer(row));
        revealer.add(&box_revealer);

        return Ok(revealer);
    };

    let revealer = try!(create_revealer(row.clone()));

    // ToDo: display when event_box clicked
    // let popover = Popover::new(Some(&event_box));
    // let popover_button = gtk::Button::new_with_label("hogehoge");
    // popover.add(&popover_button);
    Ok(revealer)
}

pub fn create_expanded_revealer(row: TimelineRow) -> Result<gtk::Revealer, CreateWidgetError> {
    let create_expanded_box_header = move |tweet: Tweet| -> Result<gtk::Box, CreateWidgetError> {
        let user_label = Label::new(None);
        let user = format!("{}", tweet.attr);
        user_label.set_text(user.as_ref());
        user_label.set_selectable(true);
        user_label.set_use_markup(true);
        user_label.set_xalign(0.0);

        let created_at_label = Label::new(None);
        created_at_label.set_text(tweet.created_at.as_ref());

        let box_header = gtk::Box::new(Orientation::Horizontal, 2);
        box_header.pack_start(&user_label, true, true, 0);
        box_header.pack_start(&created_at_label, false, false, 0);

        return Ok(box_header);
    };

    let create_expanded_box_label = move |tweet: Tweet| -> Result<gtk::Box, CreateWidgetError> {
        let box_header = try!(create_expanded_box_header(tweet.clone()));

        let body = try!(::timeline::utils::format_tweet_body(&tweet.text));
        let label_body = Label::new(None);
        label_body.set_text(body.as_ref());
        label_body.set_selectable(true);
        label_body.set_use_markup(true);
        label_body.set_line_wrap(true);
        label_body.set_xalign(0.0);

        let box_label = gtk::Box::new(Orientation::Vertical, 2);
        box_label.pack_start(&box_header, false, false, 0);
        box_label.pack_start(&label_body, true, true, 0);

        return Ok(box_label);
    };

    let create_expanded_box_revealer = move |row: TimelineRow| -> Result<gtk::Box, CreateWidgetError> {
        let profile_image_filename = try!(::utils::get_profile_image(
            &row.tweet.user.profile_image_url,
        ));
        let image_profile_image = Image::new_from_file(profile_image_filename);
        image_profile_image.set_padding(4, 4);

        let box_label = try!(create_expanded_box_label(row.tweet.clone()));

        let image_unread = Image::new_from_icon_name("gtk-media-record", 1);
        if !row.unread {
            image_unread.clear();
            image_unread.set_padding(8, 8);
        };

        let label_null = Label::new(None);
        let null = "   ";
        label_null.set_text(null.as_ref());

        let label_id = Label::new(None);
        let id = format!("{}", row.tweet.id);
        label_id.set_text(id.as_ref());
        label_id.set_visible(false);

        let label_profile_image = Label::new(None);
        let profile_image_url = format!("{}", row.tweet.user.profile_image_url);
        label_profile_image.set_text(profile_image_url.as_ref());
        label_profile_image.set_visible(false);

        let box_revealer = gtk::Box::new(Orientation::Horizontal, 2);
        box_revealer.pack_start(&image_profile_image, false, false, 0);
        box_revealer.pack_start(&box_label, true, true, 0);
        box_revealer.pack_start(&image_unread, false, false, 0);
        box_revealer.pack_start(&label_null, false, false, 0);
        box_revealer.pack_start(&label_id, false, false, 0);
        box_revealer.pack_start(&label_profile_image, false, false, 0);

        return Ok(box_revealer);
    };

    let create_expanded_revealer = move |row: TimelineRow| -> Result<gtk::Revealer, CreateWidgetError> {
        // FixMe: revealer is not available
        let revealer = gtk::Revealer::new();
        revealer.set_transition_type(RevealerTransitionType::Crossfade);
        revealer.set_transition_duration(3000);
        revealer.set_reveal_child(true);

        let box_revealer = try!(create_expanded_box_revealer(row));
        revealer.add(&box_revealer);

        return Ok(revealer);
    };

    let revealer = try!(create_expanded_revealer(row.clone()));

    // ToDo: display when event_box clicked
    // let popover = Popover::new(Some(&event_box));
    // let popover_button = gtk::Button::new_with_label("hogehoge");
    // popover.add(&popover_button);

    Ok(revealer)
}

pub fn show_listboxrow(listboxrow: &gtk::ListBoxRow) -> Result<(), gtk::Widget> {
    listboxrow.show_all();
    let revealer = listboxrow
        .get_child()
        .unwrap()
        .downcast::<gtk::Revealer>()
        .unwrap();
    let listboxrow_box = revealer
        .get_child()
        .unwrap()
        .downcast::<gtk::Box>()
        .unwrap();
    listboxrow_box.get_children()[4].hide();
    listboxrow_box.get_children()[5].hide();
    Ok(())
}

pub fn home_timeline(
    token: &egg_mode::Token,
    since_id: Option<u64>,
    count: i32,
) -> Result<Vec<TimelineRow>, egg_mode::error::Error> {
    let mut timeline: Vec<TimelineRow> = Vec::new();
    let home_timeline = egg_mode::tweet::home_timeline(&token).with_page_size(count);
    for status in &home_timeline.call(since_id, None).unwrap().response {
        let mut text = status.text.clone();
        let mut attr = format!("@{}", status.clone().user.unwrap().screen_name);
        if let Some(ref screen_name) = status.in_reply_to_screen_name {
            attr = format!(
                "@{} --> in reply to @{}",
                status.clone().user.unwrap().screen_name,
                screen_name
            );
        }
        if let Some(ref retweeted_status) = status.retweeted_status {
            text = retweeted_status.text.clone();
            attr = format!(
                "@{} retweeted from @{}",
                status.clone().user.unwrap().screen_name,
                retweeted_status.clone().user.unwrap().screen_name
            );
        }
        let created_at_local = status.created_at.naive_local();
        timeline.push(TimelineRow {
            tweet: Tweet {
                created_at: format!("{}", created_at_local),
                id: status.id.clone(),
                text: text,
                attr: attr,
                user: User {
                    screen_name: status.clone().user.unwrap().screen_name.clone(),
                    profile_image_url: status.clone().user.unwrap().profile_image_url.clone(),
                },
            },
            unread: true,
        });
        debug!("{:?}", status);
    }
    Ok(timeline)
}

// pub fn print_tweet(tweet: &egg_mode::tweet::Tweet) {
//     if let Some(ref status) = tweet.quoted_status {
//         println!("--Quoting the following status:");
//         print_tweet(status);
//     }

//     if !tweet.entities.hashtags.is_empty() {
//         println!("Hashtags contained in the tweet:");
//         for tag in &tweet.entities.hashtags {
//             println!("{}", tag.text);
//         }
//     }

//     if !tweet.entities.symbols.is_empty() {
//         println!("Symbols contained in the tweet:");
//         for tag in &tweet.entities.symbols {
//             println!("{}", tag.text);
//         }
//     }

//     if !tweet.entities.urls.is_empty() {
//         println!("URLs contained in the tweet:");
//         for url in &tweet.entities.urls {
//             println!("{}", url.expanded_url);
//         }
//     }

//     if !tweet.entities.user_mentions.is_empty() {
//         println!("Users mentioned in the tweet:");
//         for user in &tweet.entities.user_mentions {
//             println!("{}", user.screen_name);
//         }
//     }

//     if let Some(ref media) = tweet.extended_entities {
//         println!("Media attached to the tweet:");
//         for info in &media.media {
//             println!("A {:?}", info.media_type);
//         }
//     }
// }
