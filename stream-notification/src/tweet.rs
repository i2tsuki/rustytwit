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
use egg_mode;

use rustc_serialize::json;
// use regex::Regex;

// use auth;
// use links;
// use user;
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

// use egg_mode::tweet::TweetEntities;
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
    pub entities: TweetEntities,
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
    pub user: Box<TwitterUser>,
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

///Container for URL, hashtag, mention, and media information associated with a tweet.
///
///If a tweet has no hashtags, financial symbols ("cashtags"), links, or mentions, those respective
///Vecs will be empty. If there is no media attached to the tweet, that field will be `None`.
///
///Note that for media attached to a tweet, this struct will only contain the first image of a
///photo set, or a thumbnail of a video or GIF. Full media information is available in the tweet's
///`extended_entities` field.
#[derive(Debug)]
pub struct TweetEntities {
    ///Collection of hashtags parsed from the tweet.
    pub hashtags: Vec<entities::HashtagEntity>,
    ///Collection of financial symbols, or "cashtags", parsed from the tweet.
    pub symbols: Vec<entities::HashtagEntity>,
    ///Collection of URLs parsed from the tweet.
    pub urls: Vec<entities::UrlEntity>,
    ///Collection of user mentions parsed from the tweet.
    pub user_mentions: Vec<entities::MentionEntity>,
    ///If the tweet contains any attached media, this contains a collection of media information
    ///from the tweet.
    pub media: Option<Vec<entities::MediaEntity>>,
}

impl FromJson for TweetEntities {
    fn from_json(input: &json::Json) -> Result<Self, error::Error> {
        if !input.is_object() {
            return Err(InvalidResponse("TweetEntities received json that wasn't an object", Some(input.to_string())));
        }

        Ok(TweetEntities {
            hashtags: try!(field(input, "hashtags")),
            symbols: try!(field(input, "symbols")),
            urls: try!(field(input, "urls")),
            user_mentions: try!(field(input, "user_mentions")),
            media: field(input, "media").ok(),
        })
    }
}

// WIP for egg_mode::user
#[derive(Debug)]
pub struct TwitterUser {
    ///Indicates this user has an account with "contributor mode" enabled, allowing
    ///for Tweets issued by the user to be co-authored by another account. Rarely `true`.
    pub contributors_enabled: bool,
    ///The UTC timestamp for when this user account was created on Twitter.
    pub created_at: chrono::DateTime<chrono::UTC>,
    ///When true, indicates that this user has not altered the theme or background of
    ///their user profile.
    pub default_profile: bool,
    ///When true, indicates that the user has not uploaded their own avatar and a default
    ///egg avatar is used instead.
    pub default_profile_image: bool,
    ///The user-defined string describing their account.
    pub description: Option<String>,
    ///Link information that has been parsed out of the `url` or `description` fields given by the
    ///user.
    // pub entities: UserEntities,
    ///The number of tweets this user has favorited or liked in the account's lifetime.
    ///The term "favourites" and its British spelling are used for historical reasons.
    pub favourites_count: i32,
    ///When true, indicates that the authenticating user has issued a follow request to
    ///this protected account.
    pub follow_request_sent: Option<bool>,
    ///Indicates whether the authenticating user is following this account. Deprecated
    ///(and thus hidden) due to increasing error conditions where this returns None.
    following: Option<bool>,
    ///The number of followers this account has.
    ///
    ///In certain server-stress conditions, this may temporarily mistakenly return 0.
    pub followers_count: i32,
    ///The number of users this account follows, aka its "followings".
    ///
    ///In certain server-stress conditions, this may temporarily mistakenly return 0.
    pub friends_count: i32,
    ///Indicates whether this user as enabled their tweets to be geotagged.
    ///
    ///If this is set for the current user, then they can attach geographic data when
    ///posting a new Tweet.
    pub geo_enabled: bool,
    ///Unique identifier for this user.
    pub id: i64,
    ///Indicates whether the user participates in Twitter's translator community.
    pub is_translator: bool,
    ///Language code for the user's self-declared interface language.
    ///
    ///Codes are formatted as a language tag from [BCP 47][]. Only indicates the user's
    ///interface language, not necessarily the content of their Tweets.
    ///
    ///[BCP 47]: https://tools.ietf.org/html/bcp47
    pub lang: String,
    ///The number of public lists the user is a member of.
    pub listed_count: i32,
    ///The user-entered location field from their profile. Not necessarily parseable
    ///or even a location.
    pub location: Option<String>,
    ///The user-entered display name.
    pub name: String,
    ///Indicates whether the authenticated user has chosen to received this user's tweets
    ///via SMS. Deprecated (and thus hidden) due to bugs where this incorrectly returns
    ///false.
    notifications: Option<bool>,
    ///The hex color chosen by the user for their profile background.
    pub profile_background_color: String,
    ///A URL pointing to the background image chosen by the user for their profile. Uses
    ///HTTP as the protocol.
    pub profile_background_image_url: Option<String>,
    ///A URL pointing to the background image chosen by the user for their profile. Uses
    ///HTTPS as the protocol.
    pub profile_background_image_url_https: Option<String>,
    ///Indicates whether the user's `profile_background_image_url` should be tiled when
    ///displayed.
    pub profile_background_tile: Option<bool>,
    ///A URL pointing to the banner image chosen by the user. Uses HTTPS as the protocol.
    ///
    ///This is a base URL that a size specifier can be appended onto to get variously
    ///sized images, with size specifiers according to [Profile Images and Banners][profile-img].
    ///
    ///[profile-img]: https://dev.twitter.com/overview/general/user-profile-images-and-banners
    pub profile_banner_url: Option<String>,
    ///A URL pointing to the user's avatar image. Uses HTTP as the protocol. Size
    ///specifiers can be used according to [Profile Images and Banners][profile-img].
    ///
    ///[profile-img]: https://dev.twitter.com/overview/general/user-profile-images-and-banners
    pub profile_image_url: String,
    ///A URL pointing to the user's avatar image. Uses HTTPS as the protocol. Size
    ///specifiers can be used according to [Profile Images and Banners][profile-img].
    ///
    ///[profile-img]: https://dev.twitter.com/overview/general/user-profile-images-and-banners
    pub profile_image_url_https: String,
    ///The hex color chosen by the user to display links in the Twitter UI.
    pub profile_link_color: String,
    ///The hex color chosen by the user to display sidebar borders in the Twitter UI.
    pub profile_sidebar_border_color: String,
    ///The hex color chosen by the user to display sidebar backgrounds in the Twitter UI.
    pub profile_sidebar_fill_color: String,
    ///The hex color chosen by the user to display text in the Twitter UI.
    pub profile_text_color: String,
    ///Indicates whether the user wants their uploaded background image to be used.
    pub profile_use_background_image: bool,
    ///Indicates whether the user is a [protected][] account.
    ///
    ///[protected]: https://support.twitter.com/articles/14016
    pub protected: bool,
    ///The screen name or handle identifying this user.
    ///
    ///Screen names are unique per-user but can be changed. Use `id` for an immutable identifier
    ///for an account.
    ///
    ///Typically a maximum of 15 characters long, but older accounts may exist with longer screen
    ///names.
    pub screen_name: String,
    ///Indicates that the user would like to see media inline. "Somewhat disused."
    pub show_all_inline_media: Option<bool>,
    ///If possible, the most recent tweet or retweet from this user.
    ///
    ///"In some circumstances, this data cannot be provided and this field will be omitted, null,
    ///or empty." Do not depend on this field being filled. Also note that this is actually their
    ///most-recent tweet, not the status pinned to their profile.
    ///
    ///"Perspectival" items within this tweet that depend on the authenticating user
    ///[may not be completely reliable][stale-embed] in this embed.
    ///
    ///[stale-embed]: https://dev.twitter.com/docs/faq/basics/why-are-embedded-objects-stale-or-inaccurate
    pub status: Option<Box<egg_mode::tweet::Tweet>>,
    ///The number of tweets (including retweets) posted by this user.
    pub statuses_count: i32,
    ///The full name of the time zone the user has set their UI preference to.
    pub time_zone: Option<String>,
    ///The website link given by this user in their profile.
    pub url: Option<String>,
    ///The UTC offset of `time_zone` in minutes.
    pub utc_offset: Option<i32>,
    ///Indicates whether this user is a verified account.
    pub verified: bool,
    ///When present, lists the countries this user has been withheld from.
    pub withheld_in_countries: Option<Vec<String>>,
    ///When present, indicates whether the content being withheld is a "status" or "user".
    pub withheld_scope: Option<String>,
}

///Container for URL entity information that may be paired with a user's profile.
#[derive(Debug)]
pub struct UserEntities {
    ///URL information that has been parsed out of the user's `description`. If no URLs were
    ///detected, then the contained Vec will be empty.
    pub description: UserEntityDetail,
    ///Link information for the user's `url`.
    ///
    ///If `url` is present on the user's profile, so will this field. Twitter validates the URL
    ///entered to a user's profile when they save it, so this can be reasonably assumed to have URL
    ///information if it's present.
    pub url: Option<UserEntityDetail>,
}

///Represents a collection of URL entity information paired with a specific user profile field.
#[derive(Debug)]
pub struct UserEntityDetail {
    ///Collection of URL entity information.
    ///
    ///There should be one of these per URL in the paired field. In the case of the user's
    ///`description`, if no URLs are present, this field will still be present, but empty.
    pub urls: Vec<entities::UrlEntity>,
}

impl FromJson for TwitterUser {
    fn from_json(input: &json::Json) -> Result<Self, error::Error> {
        if !input.is_object() {
            return Err(InvalidResponse("TwitterUser received json that wasn't an object", Some(input.to_string())));
        }
        Ok(TwitterUser {
            contributors_enabled: field(input, "contributors_enabled").unwrap_or(false),
            created_at: try!(field(input, "created_at")),
            default_profile: try!(field(input, "default_profile")),
            default_profile_image: try!(field(input, "default_profile_image")),
            description: field(input, "description").ok(),
            // entities: try!(field(input, "entities")),
            favourites_count: try!(field(input, "favourites_count")),
            follow_request_sent: field(input, "follow_request_sent").ok(),
            following: field(input, "following").ok(),
            followers_count: try!(field(input, "followers_count")),
            friends_count: try!(field(input, "friends_count")),
            geo_enabled: try!(field(input, "geo_enabled")),
            id: try!(field(input, "id")),
            is_translator: try!(field(input, "is_translator")),
            lang: try!(field(input, "lang")),
            listed_count: try!(field(input, "listed_count")),
            location: field(input, "location").ok(),
            name: try!(field(input, "name")),
            notifications: field(input, "notifications").ok(),
            profile_background_color: try!(field(input, "profile_background_color")),
            profile_background_image_url: field(input, "profile_background_image_url").ok(),
            profile_background_image_url_https: field(input, "profile_background_image_url_https").ok(),
            profile_background_tile: field(input, "profile_background_tile").ok(),
            profile_banner_url: field(input, "profile_banner_url").ok(),
            profile_image_url: try!(field(input, "profile_image_url")),
            profile_image_url_https: try!(field(input, "profile_image_url_https")),
            profile_link_color: try!(field(input, "profile_link_color")),
            profile_sidebar_border_color: try!(field(input, "profile_sidebar_border_color")),
            profile_sidebar_fill_color: try!(field(input, "profile_sidebar_fill_color")),
            profile_text_color: try!(field(input, "profile_text_color")),
            profile_use_background_image: try!(field(input, "profile_use_background_image")),
            protected: try!(field(input, "protected")),
            screen_name: try!(field(input, "screen_name")),
            show_all_inline_media: field(input, "show_all_inline_media").ok(),
            status: field(input, "status").map(Box::new).ok(),
            statuses_count: try!(field(input, "statuses_count")),
            time_zone: field(input, "time_zone").ok(),
            url: field(input, "url").ok(),
            utc_offset: field(input, "utc_offset").ok(),
            verified: try!(field(input, "verified")),
            withheld_in_countries: input.find("withheld_in_countries").and_then(|f| f.as_array())
                                        .and_then(|arr| arr.iter().map(|x| x.as_string().map(|x| x.to_string()))
                                                           .collect::<Option<Vec<String>>>()),
            withheld_scope: field(input, "withheld_scope").ok(),
        })
    }
}

impl FromJson for UserEntities {
    fn from_json(input: &json::Json) -> Result<Self, error::Error> {
        if !input.is_object() {
            return Err(InvalidResponse("UserEntities received json that wasn't an object", Some(input.to_string())));
        }

        Ok(UserEntities {
            description: try!(field(input, "description")),
            url: field(input, "url").ok(),
        })
    }
}

impl FromJson for UserEntityDetail {
    fn from_json(input: &json::Json) -> Result<Self, error::Error> {
        if !input.is_object() {
            return Err(InvalidResponse("UserEntityDetail received json that wasn't an object", Some(input.to_string())));
        }

        Ok(UserEntityDetail {
            urls: try!(field(input, "urls")),
        })
    }
}
