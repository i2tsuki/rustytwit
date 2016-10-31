extern crate curl;
extern crate crypto;
extern crate egg_mode;
extern crate rustc_serialize;
extern crate time;

use regex;

use gtk;
use gtk::prelude::*;
use gtk::{ListBox, ListBoxRow, Revealer, Widget};
use gtk::{Orientation, RevealerTransitionType};
use gtk::{Image, Label};

use rustc_serialize::Decodable;
use rustc_serialize::json::{self, Json};

use std::clone::Clone;
use std::rc::Rc;

// error
// TimeilineError
#[derive(Debug)]
pub enum TimelineError {
    CreateWidget(CreateWidgetError),
    String(String),
    Widget(Widget),
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

impl From<Widget> for TimelineError {
    fn from(err: Widget) -> TimelineError {
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

// #[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
// pub enum Tweet {
//     Tweet(egg_mode::tweet::Tweet),
// }

// impl From<egg_mode::tweet::Tweet> for Tweet {
//     fn from(tweet: egg_mode::tweet::Tweet) -> Tweet {
//         Tweet::Tweet(tweet)
//     }
// }

// // use rustc_serialize::json;
// ///Helper trait to provide a general interface for deserializing Twitter API data structures.
// // pub trait FromJson : Sized {
// pub trait AsString : Sized {
//     ///Parse the given Json object into a data structure.
//     // fn from_json(&json::Json) -> Result<Self, error::Error>;
//     // fn as_json(egg_mode::tweet::Tweet) -> Result<String, json::Json::Error>;

//     ///Parse the given string into a Json object, then into a data structure.
//     // fn from_str(input: &str) -> Result<Self, error::Error> {
//     //     let json = try!(json::Json::from_str(input));

//     //     Self::from_json(&json)
//     // }
//     fn as_string<'a>(input: &'a egg_mode::tweet::Tweet) -> Option<&'a str>;
// }

// impl AsString for egg_mode::tweet::Tweet {
//     fn as_string<'a>(input: &'a Self) -> Option<&'a str> {
//         let json = match json::Json::as_string(input) {
//             json => json,
//             None => None,
//         };
//     }
// }
// use std::clone::Clone;


// impl<'a> Clone for &'a egg_mode::tweet::Tweet {
//   // impl Clone for egg_mode::tweet::Tweet {
// // impl Clone for egg_mode::tweet::Tweet {
//     #[inline]
//     fn clone(&self) -> &'a egg_mode::tweet::Tweet{ *self }    
// }

// #[derive(Clone, Debug, RustcEncodable, RustcDecodable)]

#[derive(Clone, Debug)]
pub struct TimelineRow {
    // pub tweet: Tweet,
    pub tweet: Tweet,
    pub unread: bool,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Tweet {
    pub created_at: String,
    pub id: i64,
    pub text: String,
    pub user: User,
    // pub retweeted_status: RetweetedStatus,
}

// impl Tweet {
//     pub fn parse_timeline(json_string: String) -> Result<Vec<Tweet>, twitter_api::Error> {
//         let value = try!(Json::from_str(&json_string));
//         let decoded = try!(Decodable::decode(&mut json::Decoder::new(value)));
//         Ok(decoded)
//     }
// }

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

pub fn update_home_timeline(listbox: &ListBox, timeline: &Vec<TimelineRow>, add: bool, unread_filter: bool) -> Result<(), TimelineError> {
    // when add flag is false, refresh all listboxrow
    if !add {
        for widget in listbox.get_children() {
            listbox.remove(&widget);
        }
    }
    
    let mute_user = vec!["syuu1228", "kakkun61", "methane"];
    let mut index: i32 = 0;

    let last_mute = try!(mute_user.last().ok_or("".to_owned())).to_string();
    
    for row in timeline {
        if unread_filter == true && row.unread == false {
            break;
        }
        for mute in mute_user.clone() {
            if row.tweet.user.screen_name == mute {
                break;
            } else if mute == last_mute {
                let listboxrow = ListBoxRow::new();
                let revealer = try!(create_revealer(row.clone()));
                listboxrow.add(&revealer);                
                listbox.insert(&listboxrow, index);

                index += 1;                
                try!(show_listboxrow(&listboxrow));
            }
        }
    }    
    return Ok(());
}

pub fn create_revealer(row: TimelineRow) -> Result<Revealer, CreateWidgetError> {
    let create_box_header = move | tweet: Tweet | -> Result<gtk::Box, CreateWidgetError> {
        let user_label = Label::new(None);
        let user = format!("<b>@{}:</b>(archived)",tweet.user.screen_name);
        user_label.set_text(user.as_ref());
        user_label.set_selectable(true);
        user_label.set_use_markup(true);
        user_label.set_xalign(0.0);
            
        let created_at_label = Label::new(None);
        let parsed = try!(time::strptime(&tweet.created_at, "%a %b %d %H:%M:%S %z %Y"));
        let created_at = try!(parsed.to_local().strftime("%a, %H:%M")).to_string();        
        // let created_at = format!("{}", tweet.created_at.format("%a, %H:%M"));
        created_at_label.set_text(created_at.as_ref());
        let box_header = gtk::Box::new(Orientation::Horizontal, 2);
        box_header.pack_start(&user_label, true, true, 0);
        box_header.pack_start(&created_at_label, false, false, 0);

        return Ok(box_header)
    };

    let create_box_label = move | tweet: Tweet | -> Result<gtk::Box, CreateWidgetError> {
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

        return Ok(box_label)
    };

    let create_box_revealer = move | row: TimelineRow | -> Result<gtk::Box, CreateWidgetError> {
        let profile_image_filename = try!(::utils::get_profile_image(&row.tweet.user.profile_image_url));
        let image_profile_image = Image::new_from_file(profile_image_filename);
        image_profile_image.set_padding(4, 4);
        
        // let box_label = try!(create_box_label(row.tweet.clone()));
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
        
        return Ok(box_revealer)
    };

    let create_revealer = move | row: TimelineRow | -> Result<Revealer, CreateWidgetError> {
        // FixMe: revealer is not available        
        let revealer = Revealer::new();
        revealer.set_transition_type(RevealerTransitionType::Crossfade);
        revealer.set_transition_duration(15000);
        revealer.set_reveal_child(true);

        let box_revealer = try!(create_box_revealer(row));
        revealer.add(&box_revealer);

        return Ok(revealer)
    };

    let revealer = try!(create_revealer(row.clone()));
    
    // ToDo: display when event_box clicked
    // let popover = Popover::new(Some(&event_box));
    // let popover_button = gtk::Button::new_with_label("hogehoge");
    // popover.add(&popover_button);
    Ok(revealer)
}

pub fn create_expanded_revealer(row: TimelineRow) -> Result<Revealer, CreateWidgetError> {
    let create_expanded_box_header = move | tweet: Tweet | -> Result<gtk::Box, CreateWidgetError> {
        let user_label = Label::new(None);
        let user = format!("<b>@{}:</b>",tweet.user.screen_name);
        user_label.set_text(user.as_ref());
        user_label.set_selectable(true);
        user_label.set_use_markup(true);
        user_label.set_xalign(0.0);
            
        let created_at_label = Label::new(None);
        let parsed = try!(time::strptime(&tweet.created_at, "%a %b %d %H:%M:%S %z %Y"));
        let created_at = try!(parsed.to_local().strftime("%a, %H:%M")).to_string();
        // let created_at = format!("{}", tweet.created_at.format("%a, %H:%M"));
        created_at_label.set_text(created_at.as_ref());

        let box_header = gtk::Box::new(Orientation::Horizontal, 2);
        box_header.pack_start(&user_label, true, true, 0);
        box_header.pack_start(&created_at_label, false, false, 0);

        return Ok(box_header)
    };

    let create_expanded_box_label = move | tweet: Tweet | -> Result<gtk::Box, CreateWidgetError> {
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

        return Ok(box_label)
    };

    let create_expanded_box_revealer = move | row: TimelineRow | ->Result<gtk::Box, CreateWidgetError> {
        let profile_image_filename = try!(::utils::get_profile_image(&row.tweet.user.profile_image_url));
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
    
        return Ok(box_revealer)
    };

    let create_expanded_revealer = move | row: TimelineRow | -> Result<Revealer, CreateWidgetError> {
        // FixMe: revealer is not available        
        let revealer = Revealer::new();
        revealer.set_transition_type(RevealerTransitionType::Crossfade);
        revealer.set_transition_duration(3000);
        revealer.set_reveal_child(true);

        let box_revealer = try!(create_expanded_box_revealer(row));
        revealer.add(&box_revealer);

        return Ok(revealer)
    };

    let revealer = try!(create_expanded_revealer(row.clone()));
    
    // ToDo: display when event_box clicked
    // let popover = Popover::new(Some(&event_box));
    // let popover_button = gtk::Button::new_with_label("hogehoge");
    // popover.add(&popover_button);

    Ok(revealer)
}

pub fn show_listboxrow(listboxrow: &ListBoxRow) -> Result<(), Widget> {
    listboxrow.show_all();
    let revealer = listboxrow.get_child().unwrap().downcast::<Revealer>().unwrap();
    let listboxrow_box = revealer.get_child().unwrap().downcast::<gtk::Box>().unwrap();
    listboxrow_box.get_children()[4].hide();
    listboxrow_box.get_children()[5].hide();
    Ok(())
}

mod api_twitter_soft {
    // pub const UPDATE_STATUS: &'static str = "https://api.twitter.com/1.1/statuses/update.json";
    pub const HOME_TIMELINE: &'static str = "https://api.twitter.com/1.1/statuses/home_timeline.json";
}

pub fn get_home_timeline(consumer_token: &egg_mode::Token, access_token: &egg_mode::Token) -> Result<Vec<TimelineRow>, egg_mode::error::Error> {
    let mut timeline: Vec<TimelineRow> = Vec::new();
    let mut home_timeline = egg_mode::tweet::home_timeline(&consumer_token, &access_token).with_page_size(5);
    
    for status in &home_timeline.start().unwrap().response {
        // let s = Rc::new(status);
        timeline.push(
            TimelineRow {
                tweet: Tweet {
                    created_at: format!("{}", status.clone().created_at),
                    id: status.clone().id.clone(),
                    text: status.clone().text.clone(),
                    user: User {
                        screen_name: status.clone().user.screen_name.clone(),
                        profile_image_url: status.clone().user.profile_image_url.clone(),
                    },
                },
                unread: true,
            }
        );        
        println!("{:?}", &status);
        println!("");
    }

    Ok(timeline)
    
    // match get_last_tweets(consumer, access, param) {
    //     Ok(tweets) => {
    //         for tweet in tweets {
    //             timeline.push(
    //                 TimelineRow {
    //                     tweet: tweet.clone(),
    //                     unread: true,
    //                 }
    //             );
    //         }
    //         return Ok(timeline)
    //     },
    //     Err(err) => return Err(err),
    // }
}

// pub fn get_last_tweets(consumer_token: egg_mode::Token, access_token: egg_mode::Token, param: &oauth_client::ParamList) -> Result<Vec<Tweet>, egg_mode::error::Error> {
//     match oauth_client::get(api_twitter_soft::HOME_TIMELINE, consumer_token, Some(access_token), Some(param)) {
//         Ok(bytes) => {
//             let last_tweets_json = try!(String::from_utf8(bytes));
//             let tweets = try!(Tweet::parse_timeline(last_tweets_json));
//             Ok(tweets)
//         },
//         Err(err) => return Err(twitter_api::Error::OAuth(err)),
//     }
// }


// extern crate chrono;

// pub fn print_tweet(tweet: &egg_mode::tweet::Tweet) {
//     println!("{} (@{}) posted at {}", tweet.user.name, tweet.user.screen_name, tweet.created_at.with_timezone(&chrono::Local));

//     if let Some(ref screen_name) = tweet.in_reply_to_screen_name {
//         println!("--> in reply to @{}", screen_name);
//     }

//     if let Some(ref status) = tweet.retweeted_status {
//         println!("Retweeted from {}:", status.user.name);
//         print_tweet(status);
//         return;
//     }
//     else {
//         println!("{}", tweet.text);
//     }

//     println!("--via {} ({})", tweet.source.name, tweet.source.url);

//     if let Some(ref place) = tweet.place {
//         println!("--from {}", place.full_name);
//     }

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
