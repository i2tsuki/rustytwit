#[macro_use]
extern crate hyper;
extern crate egg_mode;
extern crate rustc_serialize;
extern crate url;
extern crate notify_rust;

mod tweet;
mod notification;

use egg_mode::KeyPair;
use egg_mode::auth::{self, Token};
use egg_mode::common::*;
use egg_mode::user;
use hyper::Client;
use hyper::header::{Authorization, ContentType, Headers};
use hyper::method::Method;
use hyper::mime::Mime;

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::str;
use url::percent_encoding::{EncodeSet, utf8_percent_encode};

// Change these values to your real Twitter API credentials
const CONSUMER: &'static str = "xxxxxxxxxxxxxxxxxxxxxxxxx";
const CONSUMER_SECRET: &'static str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
const TOKEN: &'static str = "000000000-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
const TOKEN_SECRET: &'static str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
const ID: &'static str = "twitter"

// Struct TwitterEncodeSet
#[derive(Copy, Clone)]
struct TwitterEncodeSet;

impl EncodeSet for TwitterEncodeSet {
    fn contains(&self, byte: u8) -> bool {
        match byte {
            b'a'...b'z' | b'A'...b'Z' | b'0'...b'9' | b'-' | b'.' | b'_' | b'~' => false,
            _ => true,
        }
    }
}

fn main() {
    let consumer_token = KeyPair::new(CONSUMER, CONSUMER_SECRET);
    let access_token = KeyPair::new(TOKEN, TOKEN_SECRET);
    let token = Token::Access {
        consumer: consumer_token.clone(),
        access: access_token.clone(),
    };

    let api = user::friends_ids(ID, &token);
    let friends_ids = api.call().unwrap();
    println!("number of follows: {:?}", friends_ids.ids.len());

    let follow = friends_ids.response
        .ids
        .iter()
        .map(|f| format!("{}", f))
        .collect::<Vec<_>>()
        .join(",");

    let mut params = HashMap::new();
    add_param(&mut params, "follow", follow);

    let url = "https://stream.twitter.com/1.1/statuses/filter.json";

    // Construct full url
    let full_url = match Some(&params) {
        Some(params) => {
            let query = params.iter()
                .map(|(k, v)| {
                    format!("{}={}",
                            utf8_percent_encode(k, TwitterEncodeSet).collect::<String>(),
                            utf8_percent_encode(v, TwitterEncodeSet).collect::<String>())
                })
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", url, query)
        },
        None => url.to_string(),
    };

    // Construct headers
    let mut headers = Headers::new();
    let header = auth::get_header(Method::Get,
                                  url,
                                  &consumer_token,
                                  Some(&access_token),
                                  None,
                                  None,
                                  Some(&params));
    let content: Mime = "application/x-www-form-urlencoded".parse().unwrap();
    headers.set(Authorization(header.to_owned()));
    headers.set(ContentType(content));

    // Api call
    let client = Client::new();
    let resp = client.get(&full_url).headers(headers).send().unwrap();

    // Display and formatting tweet
    for resp_str in BufReader::new(resp).as_str() {
        match tweet::Tweet::from_str(&resp_str) {
            Ok(response) => {
                let web_resp: egg_mode::WebResponse<tweet::Tweet> = Ok(Response {
                    rate_limit: 0,
                    rate_limit_remaining: 0,
                    rate_limit_reset: 0,
                    response: response,
                });

                let tweet = web_resp.unwrap();

                if let Some(ref retweeted_status) = tweet.retweeted_status {
                    // When the tweet is retweet and that is retweeted by friends
                    for id in &friends_ids.ids {
                        if *id == tweet.user.id as u64 {
                            let obj = notification::TweetObject {
                                user_screen_name: tweet.user.screen_name.clone(),
                                retweeted_status_user_screen_name: Some(retweeted_status.user.screen_name.clone()),
                                text: retweeted_status.text.clone(),
                            };
                            notification::new(obj);
                            break;
                        }
                    }
                } else {
                    // When the weet is single tweet and that is tweeted by friends
                    let obj = notification::TweetObject {
                        user_screen_name: tweet.user.screen_name.clone(),
                        retweeted_status_user_screen_name: None,
                        text: tweet.text.clone(),
                    };
                    notification::new(obj);
                    break;
                }
            },
            Err(_) => {
                println!("resp_str: {}", resp_str);
                // println!("tweet is deleted");
            },
        }
    }
}

pub struct JsonStr<'a, B>
    where B: 'a
{
    reader: &'a mut B,
}

pub trait JsonStrStreamer: Sized {
    fn as_str(&mut self) -> JsonStr<Self>;
}

impl<T: BufRead> JsonStrStreamer for T {
    fn as_str(&mut self) -> JsonStr<T> {
        JsonStr { reader: self, }
    }
}

impl<'a, B> Iterator for JsonStr<'a, B> where B: BufRead + 'a
{
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut buf: Vec<u8> = Vec::new();

        // Waiting to read endswith '\b'
        loop {
            let _ = self.reader.read_until(10, &mut buf);
            if buf.len() == 2 {
                buf.clear();
                continue;
            }
            break;
        }
        let line = match str::from_utf8(&buf) {
            Ok(line) => line,
            Err(_) => return None,
        };
        Some(line.to_string())
    }
}
