use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Multipart, Path},
    response::Response,
    routing::{get, post},
    Extension, Router,
};
use tokio_util::io::ReaderStream;

use crate::{
    context::Context, utils::general_utils::random_string, utils::user_utils::validate_auth,
    web::scores::submission::ParsedMultipart,
};

use self::direct::{download_osz, search_beatmap_set, search_beatmaps};

mod direct;

async fn upload_screenshot(
    Extension(ctx): Extension<Arc<Context>>,
    multipart: Multipart,
) -> Response {
    let form_data: ParsedMultipart = ParsedMultipart::from_multipart(multipart).await;

    if !validate_auth(
        &ctx.redis,
        &ctx.pool,
        form_data.get_field::<String>("u").unwrap(),
        form_data.get_field::<String>("p").unwrap(),
    )
    .await
    {
        return Response::builder()
            .status(400)
            .body(Body::from("github.com/shoe001a"))
            .unwrap();
    }

    let file_name = random_string(8) + ".jpg";

    tokio::fs::write(
        format!("data/screenshots/{file_name}"),
        form_data.get_file("ss").unwrap(),
    )
    .await
    .unwrap();

    let domain = ctx.config.server_url.to_string();

    Response::builder()
        .status(200)
        .body(Body::from(format!("https://osu.{domain}/ss/{file_name}")))
        .unwrap()
}

async fn view_screenshot(Path(file_name): Path<String>) -> Response {
    let file = tokio::fs::File::open(format!("data/screenshots/{}", file_name.clone()))
        .await
        .unwrap();

    Response::builder()
        .header("content-type", r#"image/jpeg"#.to_string())
        .header(
            "content-length",
            format!(r#"{}"#, file.metadata().await.unwrap().len()),
        )
        .body(Body::from_stream(ReaderStream::new(file)))
        .unwrap()
}

pub fn serve() -> Router {
    Router::new()
        .route("/web/osu-search.php", get(search_beatmaps))
        .route("/web/osu-search-set.php", get(search_beatmap_set))
        .route("/web/osu-screenshot.php", post(upload_screenshot))
        .route("/ss/:file", get(view_screenshot))
        .route("/d/:id", get(download_osz))
}
