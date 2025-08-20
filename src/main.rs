use std::sync::Arc;

use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::url::{TinyUrlService, UrlPostResult};
mod url;

#[derive(Deserialize, Serialize, Debug)]
struct Url {
    url: String,
    preference: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct UrlResponse {
    tiny_url: String,
}

impl UrlResponse {
    fn from(url: Url) -> UrlResponse {
        UrlResponse { tiny_url: url.url }
    }
}

#[tokio::main]
async fn main() {
    let service = Arc::new(TinyUrlService::from("db/url_store"));
    let tiny = {
        let service = Arc::clone(&service);
        warp::post()
            .and(warp::path("tiny"))
            .and(warp::body::json())
            .map(move |url: Url| {
                if let Some(preference) = &url.preference {
                    warp::reply::json(&UrlResponse::from(Url {
                        url: match service.post(String::from(url.url), None) {
                            UrlPostResult::Success(value) => value,
                            UrlPostResult::Taken => String::from("Oops."),
                            UrlPostResult::DbError => String::from("Err!"),
                        },
                        preference: Some(preference.clone()),
                    }))
                } else {
                    warp::reply::json(&UrlResponse::from(url))
                }
            })
    };
    let tiny2 = warp::path("tiny").map(|| "hey!");

    let html = warp::path("url_shortener").and(warp::fs::file("example.html"));

    warp::serve(tiny.or(tiny2).or(html))
        .run(([127, 0, 0, 1], 3030))
        .await;
}
