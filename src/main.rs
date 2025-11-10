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
    let zengarden_static = warp::path("zengarden")
        .and(warp::fs::dir("html/zen-garden/frontend"))
        .map(|file| Box::new(file) as Box<dyn warp::Reply>);

    let zengarden_index = warp::path("zengarden")
        .and(warp::path::tail())
        .and_then(|_| async {
            match tokio::fs::read_to_string("html/zen-garden/frontend/index.html").await {
                Ok(contents) => Ok(warp::reply::with_status(
                    warp::reply::html(contents),
                    warp::http::StatusCode::OK,
                )),
                Err(_) => Err(warp::reject::not_found()),
            }
        });

    let polaroid = warp::path("polaroid").and(warp::fs::dir("html/polaroid/"));

    let website_html = warp::fs::dir("html/website/");

    let resume_editor_static = warp::path("resume-editor")
        .and(warp::fs::dir("html/resume-editor/frontend/build"))
        .map(|file| Box::new(file) as Box<dyn warp::Reply>);

    let resume_editor_index = warp::path("resume-editor")
        .and(warp::path::tail())
        .and_then(|_| async {
            match tokio::fs::read_to_string("html/resume-editor/frontend/build/index.html").await {
                Ok(contents) => Ok(warp::reply::with_status(
                    warp::reply::html(contents),
                    warp::http::StatusCode::OK,
                )),
                Err(_) => Err(warp::reject::not_found()),
            }
        });

    let notepad_files = warp::path("notepad")
        .and(warp::fs::dir("html/notepad/_site"));

    let notepad_index = warp::path("notepad")
        .and(warp::path::end())
        .and(warp::fs::file("html/notepad/_site/index.html"));

    let notepad_routes = notepad_index.or(notepad_files);

    let routes = tiny
        .or(tiny_get)
        .or(tinyurl_html)
        .or(zengarden_static)
        .or(zengarden_index)
        .or(website_html)
        .or(resume_editor_static)
        .or(resume_editor_index)
        .or(polaroid)
        .or(notepad_routes)
        .recover(move |_err| serve_html(str_404.clone()));
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
