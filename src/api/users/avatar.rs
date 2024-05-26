use std::{io::Cursor, sync::Arc};

use axum::{debug_handler, extract::{FromRequest, Path, Request}, Extension, Json};
use image::{imageops::FilterType, io::Reader};
use multer::{parse_boundary, Multipart};
use tracing::{warn, error};

use crate::{api::FailableResponse, context::Context, db::user::User};

#[debug_handler]
pub async fn upload_avatar(
    Extension(ctx): Extension<Arc<Context>>,
    Extension(authorized_user): Extension<Option<User>>,
    Path(id): Path<String>,
    request: Request
) -> Json<FailableResponse<bool>> {

    if authorized_user.is_none() {
        return Json(FailableResponse {
            message: Some("Unauthorized".into()),
            data: None,
            ok: false
        })
    }

    let mut avatar_file: Option<Vec<u8>> = None;
    
    let content_type = request.headers().get("content-type").unwrap().to_str().unwrap_or_default();
    let boundary = parse_boundary(content_type).unwrap_or("".into());
    
    let data_stream = request.into_body().into_data_stream();

    let mut multipart = Multipart::new(data_stream, boundary);
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        if name == "attachment" {
            avatar_file = Some(field.bytes().await.unwrap().to_vec());

        }
    }

    if let None = avatar_file {
        return Json(FailableResponse {
            data: None,
            message: Some("No attachment field found.".into()),
            ok: false
        })
    }

    let avatar_image = Reader::new(Cursor::new(avatar_file.unwrap())).with_guessed_format().unwrap().decode();

    if let Err(error) = avatar_image {
        warn!("{:#?}", error);
        return Json(FailableResponse { ok: false, message: Some("Failed to read image.".into()), data: None })
    }

    let avatar_image = avatar_image.unwrap();
    
    let image = avatar_image.resize_to_fill(512, 512, FilterType::Lanczos3);

    if let Err(error) = image.save(format!("data/avatars/{}.png", authorized_user.unwrap().id)) {
        error!("Failed to save avatar file: {}", error);
        return Json(FailableResponse { ok: false, message: Some("Failed to save file.".into()), data: None })
    }
    // let mut resizer = Resizer::new();
    // resizer.resize(&avatar_image, &mut dst_image, ResizeOptions::new().crop(left_offset, top_offset, 512, 512));

    Json(FailableResponse {
        data: Some(true),
        message: None,
        ok: true,
    })
}
