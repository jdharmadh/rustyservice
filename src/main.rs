use warp::Filter;
use serde::{Deserialize, Serialize};

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
    let tiny = warp::post().and(warp::path("tiny")).and(warp::body::json()).map(|url: Url| {
        if let Some(preference) = &url.preference {
            warp::reply::json(&UrlResponse::from(Url {
                url: String::from("heebie jeebie"),
                preference: Some(preference.clone()),
            }))
        } else {
            warp::reply::json(&UrlResponse::from(url))
        }
    });
    let tiny2 = warp::path("tiny").map(|| {
        "hey!"
    });

    let html = warp::path("url_shortener").and(warp::fs::file("example.html"));

    warp::serve(tiny.or(tiny2).or(html))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

// fn two() -> sled::Result<()> {
//     let db = sled::open("db/url_store")?;
//     Ok(())
// }
