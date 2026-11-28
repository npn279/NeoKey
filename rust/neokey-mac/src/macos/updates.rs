use std::sync::mpsc::{self, Receiver};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{
    ns_string, NSBundle, NSData, NSDictionary, NSError, NSHTTPURLResponse, NSJSONReadingOptions,
    NSJSONSerialization, NSNumber, NSString, NSURLRequest, NSURLRequestCachePolicy, NSURLResponse,
    NSURLSession, NSURLSessionConfiguration, NSURL,
};

const MAX_VERSION_BYTES: usize = 64 * 1024;
pub const RELEASES: &str = "https://github.com/npn279/NeoKey/releases";

pub enum Update {
    Newer(String),
    Current,
    Unavailable,
}

pub fn bundle_version(key: &NSString) -> Option<String> {
    Some(
        NSBundle::mainBundle()
            .objectForInfoDictionaryKey(key)?
            .downcast::<NSString>()
            .ok()?
            .to_string(),
    )
}

pub fn check() -> Receiver<Update> {
    let (send, receive) = mpsc::channel();
    let Some(current) =
        bundle_version(ns_string!("CFBundleVersion")).and_then(|s| s.parse::<u64>().ok())
    else {
        let _ = send.send(Update::Unavailable);
        return receive;
    };

    let url = NSURL::URLWithString(ns_string!(
        "https://raw.githubusercontent.com/npn279/NeoKey/main/version.json"
    ))
    .expect("hard-coded update URL");
    let request = NSURLRequest::requestWithURL_cachePolicy_timeoutInterval(
        &url,
        NSURLRequestCachePolicy::ReloadIgnoringLocalCacheData,
        10.0,
    );
    let config = NSURLSessionConfiguration::ephemeralSessionConfiguration();
    config.setTimeoutIntervalForResource(15.0);
    let session = NSURLSession::sessionWithConfiguration(&config);
    let completion = RcBlock::new(
        move |data: *mut NSData, response: *mut NSURLResponse, error: *mut NSError| {
            // NSURLSession owns these values for the duration of this callback.
            let version = unsafe {
                match (error.is_null(), data.as_ref(), response.as_ref()) {
                    (true, Some(data), Some(response))
                        if response
                            .downcast_ref::<NSHTTPURLResponse>()
                            .is_some_and(|response| response.statusCode() == 200) =>
                    {
                        parse_version(data)
                    }
                    _ => None,
                }
            };
            let _ = send.send(match version {
                Some((code, name)) if code > current => Update::Newer(name),
                Some(_) => Update::Current,
                None => Update::Unavailable,
            });
        },
    );
    // The callback captures only Send values and parses on the session queue.
    unsafe { session.dataTaskWithRequest_completionHandler(&request, &completion) }.resume();
    session.finishTasksAndInvalidate();
    receive
}

fn parse_version(data: &NSData) -> Option<(u64, String)> {
    if data.length() > MAX_VERSION_BYTES {
        return None;
    }
    let root =
        NSJSONSerialization::JSONObjectWithData_options_error(data, NSJSONReadingOptions::empty())
            .ok()?;
    let version = json_field(&root, ns_string!("latestVersion"))?;
    let code = json_field(&version, ns_string!("versionCode"))?;
    let code = if let Some(number) = code.downcast_ref::<NSNumber>() {
        number.stringValue().to_string().parse().ok()?
    } else {
        code.downcast_ref::<NSString>()?.to_string().parse().ok()?
    };
    let name = json_field(&version, ns_string!("versionName"))?;
    let name = name.downcast_ref::<NSString>()?.to_string();
    (!name.is_empty() && name.len() <= 64).then_some((code, name))
}

fn json_field(object: &AnyObject, key: &NSString) -> Option<Retained<AnyObject>> {
    // JSON object keys are NSString and values can be any JSON object.
    let dictionary = object.downcast_ref::<NSDictionary>()?;
    dictionary.objectForKey(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_only_the_macos_version_from_the_legacy_feed() {
        let feed = NSData::with_bytes(
            br#"{
            "latestVersion":{"versionCode":50,"versionName":"2.0.6"},
            "latestWinVersion":{"versionCode":131074,"versionName":"2.0.2"}
        }"#,
        );
        assert_eq!(parse_version(&feed), Some((50, "2.0.6".to_owned())));
    }

    #[test]
    fn missing_or_invalid_release_metadata_cannot_trigger_an_update() {
        for body in [
            "{}",
            r#"{"latestVersion":{"versionCode":-2,"versionName":"2.0.6"}}"#,
            r#"{"latestVersion":{"versionCode":999,"versionName":{}}}"#,
        ] {
            assert_eq!(parse_version(&NSData::with_bytes(body.as_bytes())), None);
        }
    }
}
