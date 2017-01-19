//! Structs and functions for working with statuses and timelines.
//!
//! In this module, you can find various structs and methods to load and interact with tweets and
//! their metadata. This also includes loading a user's timeline, posting a new tweet, or liking or
//! retweeting another tweet. However, this does *not* include searching for tweets; that
//! functionality is in the [`search`][] module.
//!
//! [`search`]: ../search/index.html
//!
//! ## Types
//!
//! - `Tweet`/`TweetEntities`/`ExtendedTweetEntities`: At the bottom of it all, this is the struct
//!   that represents a single tweet. The `*Entities` structs contain information about media,
//!   links, and hashtags within their parent tweet.
//! - `DraftTweet`: This is what you use to post a new tweet. At present, not all available options
//!   are supported, but basics like marking the tweet as a reply and attaching a location
//!   coordinate are available.
//! - `Timeline`: Returned by several functions in this module, this is how you cursor through a
//!   collection of tweets. See the struct-level documentation for details.
//!
//! ## Functions
//!
//! ### User actions
//!
//! These functions perform actions on their given tweets. They require write access to the
//! authenticated user's account.
//!
//! - `delete` (for creating a tweet, see `DraftTweet`)
//! - `like`/`unlike`
//! - `retweet`/`unretweet`
//!
//! ### Metadata lookup
//!
//! These functions either perform some direct lookup of specific tweets, or provide some metadata
//! about the given tweet in a direct (non-`Timeline`) fashion.
//!
//! - `show`
//! - `lookup`/`lookup_map` (for the differences between these functions, see their respective
//!   documentations.)
//! - `retweeters_of`
//! - `retweets_of`
//!
//! ### `Timeline` cursors
//!
//! These functions return `Timeline`s and can be cursored around in the same way. See the
//! documentation for `Timeline` to learn how to navigate these return values. This correspond to a
//! user's own view of Twitter, or with feeds you might see attached to a user's profile page.
//!
//! - `home_timeline`/`mentions_timeline`/`retweets_of_me`
//! - `user_timeline`/`liked_by`

extern crate chrono;

use rustc_serialize::json;
// use regex::Regex;

// use auth;
// use links;
// use user;
use egg_mode::user;
// use error;
use egg_mode::error;
// use error::Error::InvalidResponse;
use egg_mode::error::Error::InvalidResponse;
// use entities;
use egg_mode::entities;
// use place;
use egg_mode::place;
// use common::*;
use egg_mode::common::*;
// mod fun;

// pub use self::fun::*;

use egg_mode::tweet::TweetEntities;
use egg_mode::tweet::TweetSource;
use egg_mode::tweet::ExtendedTweetEntities;

#[derive(Debug)]
pub struct Tweet {
    //If the user has contributors enabled, this will show which accounts contributed to this
    //tweet.
    //pub contributors: Option<Contributors>,
    ///If present, the location coordinate attached to the tweet, as a (latitude, longitude) pair.
    pub coordinates: Option<(f64, f64)>,
    ///UTC timestamp from when the tweet was posted.
    pub created_at: chrono::DateTime<chrono::UTC>,
    ///If the authenticated user has retweeted this tweet, contains the ID of the retweet.
    pub current_user_retweet: Option<i64>,
    ///If this tweet is an extended tweet with "hidden" metadata and entities, contains the code
    ///point indices where the "displayable" tweet text is.
    pub display_text_range: Option<(i32, i32)>,
    ///Link, hashtag, and user mention information extracted from the tweet text.
    // pub entities: TweetEntities,
    ///Extended media information attached to the tweet, if media is available.
    ///
    ///If a tweet has a photo, set of photos, gif, or video attached to it, this field will be
    ///present and contain the real media information. The information available in the `media`
    ///field of `entities` will only contain the first photo of a set, or a thumbnail of a gif or
    ///video.
    pub extended_entities: Option<ExtendedTweetEntities>,
    ///"Approximately" how many times this tweet has been liked by users.
    pub favorite_count: i32,
    ///Indicates whether the authenticated user has liked this tweet.
    pub favorited: Option<bool>,
    //Indicates the maximum `FilterLevel` parameter that can be applied to a stream and still show
    //this tweet.
    //pub filter_level: FilterLevel,
    ///Numeric ID for this tweet.
    pub id: i64,
    ///If the tweet is a reply, contains the ID of the user that was replied to.
    pub in_reply_to_user_id: Option<i64>,
    ///If the tweet is a reply, contains the screen name of the user that was replied to.
    pub in_reply_to_screen_name: Option<String>,
    ///If the tweet is a reply, contains the ID of the tweet that was replied to.
    pub in_reply_to_status_id: Option<i64>,
    ///Can contain a language ID indicating the machine-detected language of the text, or "und" if
    ///no language could be detected.
    pub lang: String,
    ///When present, the `Place` that this tweet is associated with (but not necessarily where it
    ///originated from).
    pub place: Option<place::Place>,
    ///If the tweet has a link, indicates whether the link may contain content that could be
    ///identified as sensitive.
    pub possibly_sensitive: Option<bool>,
    ///If this tweet is quoting another by link, contains the ID of the quoted tweet.
    pub quoted_status_id: Option<i64>,
    ///If this tweet is quoting another by link, contains the quoted tweet.
    pub quoted_status: Option<Box<Tweet>>,
    //"A set of key-value pairs indicating the intended contextual delivery of the containing
    //Tweet. Currently used by Twitter’s Promoted Products."
    //pub scopes: Option<Scopes>,
    ///The number of times this tweet has been retweeted (with native retweets).
    pub retweet_count: i32,
    ///Indicates whether the authenticated user has retweeted this tweet.
    pub retweeted: Option<bool>,
    ///If this tweet is a retweet, then this field contains the original status information.
    ///
    ///The separation between retweet and original is so that retweets can be recalled by deleting
    ///the retweet, and so that liking a retweet results in an additional notification to the user
    ///who retweeted the status, as well as the original poster.
    pub retweeted_status: Option<Box<Tweet>>,
    ///The application used to post the tweet.
    pub source: TweetSource,
    ///The text of the tweet. For "extended" tweets, opening reply mentions and/or attached media
    ///or quoted tweet links do not count against character count, so this could be longer than 140
    ///characters in those situations.
    pub text: String,
    ///Indicates whether this tweet is a truncated "compatibility" form of an extended tweet whose
    ///full text is longer than 140 characters.
    pub truncated: bool,
    ///The user who posted this tweet.
    pub user: Box<user::TwitterUser>,
    ///If present and `true`, indicates that this tweet has been withheld due to a DMCA complaint.
    pub withheld_copyright: bool,
    ///If present, contains two-letter country codes indicating where this tweet is being withheld.
    ///
    ///The following special codes exist:
    ///
    ///- `XX`: Withheld in all countries
    ///- `XY`: Withheld due to DMCA complaint.
    pub withheld_in_countries: Option<Vec<String>>,
    ///If present, indicates whether the content being withheld is the `status` or the `user`.
    pub withheld_scope: Option<String>,
}

impl FromJson for Tweet {
    fn from_json(input: &json::Json) -> Result<Self, error::Error> {
        if !input.is_object() {
            return Err(InvalidResponse("Tweet received json that wasn't an object", Some(input.to_string())));
        }

        //TODO: when i start building streams, i want to extract "extended_tweet" and use its
        //fields here

        let coords = field(input, "coordinates").ok();

        Ok(Tweet {
            //contributors: Option<Contributors>,
            coordinates: coords.map(|(lon, lat)| (lat, lon)),
            created_at: try!(field(input, "created_at")),
            current_user_retweet: try!(current_user_retweet(input, "current_user_retweet")),
            display_text_range: field(input, "display_text_range").ok(),
            entities: try!(field(input, "entities")),
            extended_entities: field(input, "extended_entities").ok(),
            favorite_count: field(input, "favorite_count").unwrap_or(0),
            favorited: field(input, "favorited").ok(),
            //filter_level: FilterLevel,
            id: try!(field(input, "id")),
            in_reply_to_user_id: field(input, "in_reply_to_user_id").ok(),
            in_reply_to_screen_name: field(input, "in_reply_to_screen_name").ok(),
            in_reply_to_status_id: field(input, "in_reply_to_status_id").ok(),
            lang: try!(field(input, "lang")),
            place: field(input, "place").ok(),
            possibly_sensitive: field(input, "possibly_sensitive").ok(),
            quoted_status_id: field(input, "quoted_status_id").ok(),
            quoted_status: field(input, "quoted_status").map(Box::new).ok(),
            //scopes: Option<Scopes>,
            retweet_count: try!(field(input, "retweet_count")),
            retweeted: field(input, "retweeted").ok(),
            retweeted_status: field(input, "retweeted_status").map(Box::new).ok(),
            source: try!(field(input, "source")),
            text: try!(field(input, "full_text").or(field(input, "text"))),
            truncated: try!(field(input, "truncated")),
            user: try!(field(input, "user").map(Box::new)),
            withheld_copyright: field(input, "withheld_copyright").unwrap_or(false),
            withheld_in_countries: field(input, "withheld_in_countries").ok(),
            withheld_scope: field(input, "withheld_scope").ok(),
        })
    }
}

fn current_user_retweet(input: &json::Json, field: &'static str) -> Result<Option<i64>, error::Error> {
    if let Some(obj) = input.find(field).and_then(|f| f.as_object()) {
        match obj.get("id").and_then(|o| o.as_i64()) {
            Some(id) => Ok(Some(id)),
            None => Err(InvalidResponse("invalid structure inside current_user_retweet", None)),
        }
    }
    else {
        Ok(None)
    }
}
