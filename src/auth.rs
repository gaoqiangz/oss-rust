use super::oss::OSS;
use hmac::Mac;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::header::{CONTENT_TYPE, DATE};

pub trait Auth {
    fn oss_sign(
        &self,
        verb: &str,
        key_id: &str,
        key_secret: &str,
        bucket: &str,
        object: &str,
        oss_resources: &str,
        headers: &HeaderMap,
    ) -> String;
}

impl Auth for OSS {
    fn oss_sign(
        &self,
        verb: &str,
        key_id: &str,
        key_secret: &str,
        bucket: &str,
        object: &str,
        oss_resources: &str,
        headers: &HeaderMap,
    ) -> String {
        let date = headers
            .get(DATE)
            .and_then(|d| Some(d.to_str().unwrap_or_default()))
            .unwrap_or_default();
        let content_type = headers
            .get(CONTENT_TYPE)
            .and_then(|c| Some(c.to_str().unwrap_or_default()))
            .unwrap_or_default();
        let content_md5 = headers
            .get("Content-MD5")
            .and_then(|md5| Some(base64::encode(md5.to_str().unwrap_or_default())))
            .unwrap_or_default();

        let mut oss_headers: Vec<(&HeaderName, &HeaderValue)> = headers
            .iter()
            .filter(|(k, _)| k.as_str().contains("x-oss-"))
            .collect();
        oss_headers.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));
        let mut oss_headers_str = String::new();
        for (k, v) in oss_headers {
            oss_headers_str += &format!(
                "{}:{}\n",
                k.to_owned().as_str(),
                v.to_owned().to_str().unwrap_or("")
            );
        }

        let oss_resource_str = get_oss_resource_str(bucket, object, oss_resources);
        let sign_str = format!(
            "{}\n{}\n{}\n{}\n{}{}",
            verb, content_md5, content_type, date, oss_headers_str, oss_resource_str
        );

        let mut hasher = hmac::Hmac::<sha1::Sha1>::new_from_slice(key_secret.as_bytes()).unwrap();
        hasher.update(sign_str.as_bytes());
        let sign_str_base64 = base64::encode(hasher.finalize().into_bytes());

        let authorization = format!("OSS {}:{}", key_id, sign_str_base64);
        debug!("authorization: {}", authorization);
        authorization
    }
}

#[inline]
fn get_oss_resource_str(bucket: &str, object: &str, oss_resources: &str) -> String {
    let oss_resources = if oss_resources != "" {
        String::from("?") + oss_resources
    } else {
        String::new()
    };
    if bucket == "" {
        format!("/{}{}", bucket, oss_resources)
    } else {
        format!("/{}/{}{}", bucket, object, oss_resources)
    }
}
