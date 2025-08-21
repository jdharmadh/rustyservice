use core::str;
use std::sync::Arc;
use warp::{Filter, Reply};

mod url;
use crate::url::{TinyUrlService, UrlPostResult, TinyUrlHttpRequest, TinyUrlHttpResponse};

async fn serve_html(html: String) -> Result<impl Reply, std::convert::Infallible> {
        Ok(warp::reply::with_status(
            warp::reply::html(html),
            warp::http::StatusCode::NOT_FOUND,
        ))
}

#[tokio::main]
async fn main() {
    let service = Arc::new(TinyUrlService::from("db/url_store"));
    let str_404 = tokio::fs::read_to_string("html/404.html").await.unwrap();
    let tiny = {
        let service = Arc::clone(&service);
        warp::post()
            .and(warp::path("tiny"))
            .and(warp::body::json())
            .map(move |url: TinyUrlHttpRequest| {
                warp::reply::json(&TinyUrlHttpResponse::from(TinyUrlHttpRequest {
                    url: match service.post(String::from(url.url), url.preference.clone()) {
                        UrlPostResult::Success(value) => value,
                        UrlPostResult::Taken => String::from("Oops."),
                        UrlPostResult::DbError => String::from("Err!"),
                    },
                    preference: None,
                }))
            })
    };
    let tiny_get = warp::path!("tiny" / String).and_then(move |str: String| {
        let service = Arc::clone(&service);
        async move {
            match service.get(str) {
                Ok(url) => {
                    match url.parse::<warp::http::Uri>() {
                        Ok(uri) => Ok(warp::redirect::temporary(uri)),
                        Err(_) => Err(warp::reject::not_found()),
                    }
                },
                Err(_) => Err(warp::reject::not_found()),
            }
        }
    });

    let html = warp::path::end().and(warp::fs::file("html/url_shortener_main.html"));

    let routes = tiny.or(tiny_get).or(html).recover(move |_err| serve_html(str_404.clone()));

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
