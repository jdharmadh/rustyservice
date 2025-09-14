use core::str;
use std::sync::Arc;
use warp::{Filter, Reply};

mod url;
use crate::url::{TinyUrlHttpRequest, TinyUrlHttpResponse, TinyUrlService};

async fn serve_html(html: String) -> Result<impl Reply, std::convert::Infallible> {
    Ok(warp::reply::with_status(
        warp::reply::html(html),
        warp::http::StatusCode::NOT_FOUND,
    ))
}

#[tokio::main]
async fn main() {
    let service = Arc::new(TinyUrlService::from("app/url_store"));
    let str_404 = tokio::fs::read_to_string("html/404.html").await.unwrap();
    let tiny = {
        let service = Arc::clone(&service);
        warp::post()
            .and(warp::path("tiny"))
            .and(warp::body::json())
            .map(move |url: TinyUrlHttpRequest| {
                let res = service.post(String::from(url.url), url.preference.clone());
                let (status, message) = res.into();
                warp::reply::with_status(
                    warp::reply::json(&TinyUrlHttpResponse::from(String::from(message))),
                    status,
                )
            })
    };
    let tiny_get = warp::path!("tiny" / String).and_then(move |str: String| {
        let service = Arc::clone(&service);
        async move {
            match service.get(str) {
                Ok(url) => match url.parse::<warp::http::Uri>() {
                    Ok(uri) => Ok(warp::redirect::temporary(uri)),
                    Err(_) => Err(warp::reject::not_found()),
                },
                Err(_) => Err(warp::reject::not_found()),
            }
        }
    });

    let tinyurl_html = warp::path("tiny")
        .and(warp::path::end())
        .and(warp::fs::file("html/tiny_url/index.html"));

    let zengarden_index = warp::path("zengarden")
        .and(warp::path::end())
        .and(warp::fs::file("html/zen-garden/frontend/index.html"));
    let zengarden_static = warp::path("zengarden").and(warp::fs::dir("html/zen-garden/frontend"));

    let website_html = warp::fs::dir("html/website/");

    let routes = tiny
        .or(tiny_get)
        .or(tinyurl_html)
        .or(zengarden_index)
        .or(zengarden_static)
        .or(website_html)
        .recover(move |_err| serve_html(str_404.clone()));

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
