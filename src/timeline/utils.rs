extern crate regex;

use regex::Regex;

pub fn format_tweet_body(text: &str) -> Result<String, regex::Error> {
    let hyperlink_re = Regex::new(r"(?P<hyperlink>http[s]://[0-9a-zA-Z\./]+)").unwrap();
    let body = format!(r#"{}"#,
                       hyperlink_re.replace_all(text, r#"<a href="$hyperlink">$hyperlink</a>"#));
    return Ok(body);
}
