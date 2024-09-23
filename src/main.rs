mod error;

use error::handle_recover;

use core::convert::Infallible;
use std::str::FromStr;
use warp::{filters::BoxedFilter, http::StatusCode, reply, reply::Reply, Filter};

const STATIC_PATH: &str = "public_html";

#[tokio::main]
async fn main() {
    // ex http://127.0.0.1:8080/service/healthcheck
    // ex http://127.0.0.1:8080/service/manage/other/execute

    let router = routers();
    warp::serve(router).run(([0, 0, 0, 0], 8080)).await;
}

fn routers() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let hc_service = healthcheck_router();
    let othet_service = other_router();

    warp::path("service")
        .and(hc_service.or(othet_service))
        .or(static_serve())
}

fn healthcheck_router() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let execute_action = warp::any()
        .and(warp::path::end())
        .and_then(healthcheck_handler);

    warp::path("healthcheck").and(execute_action.recover(handle_recover))
}

pub async fn healthcheck_handler() -> Result<impl warp::Reply, warp::Rejection> {
    return Ok(reply::with_status("Success", StatusCode::OK));
}

pub fn other_router() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let hello_action = warp::any()
        .and(warp::path("hello"))
        .and(warp::path::end())
        .and_then(hello_handler);

    let wait_action = warp::any()
        .and(warp::path("wait"))
        .and(warp::path::end())
        .and_then(wait_handler);

    let json_action = warp::post()
        .and(warp::path("json"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(json_handler);

    let with_fullpath =
        warp::path::full().map(move |path: warp::path::FullPath| path.as_str().to_string());
    let path_action = warp::any()
        .and(warp::path("path"))
        .and(with_fullpath)
        .and_then(path_handler);

    warp::path("manage").and(warp::path("other")).and(
        hello_action
            .or(wait_action)
            .or(json_action)
            .or(path_action)
            .recover(handle_recover),
    )
}

pub async fn path_handler(path: String) -> Result<impl warp::Reply, warp::Rejection> {
    return Ok(warp::reply::json(&serde_json::json!({"fullPath":path})));
}

pub async fn hello_handler() -> Result<impl warp::Reply, warp::Rejection> {
    return Ok(warp::reply::json(
        &serde_json::json!({"message":"Hello,World!"}),
    ));
}

pub async fn wait_handler() -> Result<impl warp::Reply, warp::Rejection> {
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    return Ok(warp::reply::json(
        &serde_json::json!({"message":"wait mow!"}),
    ));
}

pub async fn json_handler(params: serde_json::Value) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&params))
}

pub fn static_serve() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // handlerでアクセスさせたくないディレクトリを制御して
    // warp::fs::dirで静的公開するディレクトリを指定する
    static_handler().or(warp::fs::dir(STATIC_PATH))
}

/// static access handler
pub fn static_handler() -> BoxedFilter<(impl Reply,)> {
    warp::path::full()
        .and_then(move |path: warp::path::FullPath| async move {
            let path = path.as_str();

            if path.starts_with("/service/") || path.eq("/service") {
                // 静的アクセスさせないディレクトリはここ
                return Ok(warp::Reply::into_response(
                    warp::http::StatusCode::NOT_FOUND,
                ));
            }

            if path.is_empty() || !path.ends_with("/") {
                if std::path::Path::new(STATIC_PATH)
                    .join(path.trim_start_matches('/'))
                    .is_dir()
                {
                    // 末尾に/なしでindex.htmlを表示させたければ、/を付加してリダイレクト
                    return Ok(warp::Reply::into_response(warp::redirect::redirect(
                        warp::http::Uri::from_str(&[path, "/"].concat()).unwrap(),
                    )));
                }
            }

            // リジェクトすれば静的サイトになる
            Err(warp::reject())
        })
        .boxed()
}
