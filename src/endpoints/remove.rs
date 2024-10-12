use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};
use futures::TryStreamExt;

use crate::args::ARGS;
use crate::endpoints::errors::ErrorTemplate;
use crate::util::animalnumbers::to_u64;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::remove_expired;
use crate::AppState;
use askama::Template;

#[get("/remove/{id}")]
pub async fn remove(data: web::Data<AppState>, id: web::Path<String>) -> Result<HttpResponse, Error> {
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            
            return Ok(HttpResponse::Found()
                .append_header(("Location", format!("{}/list", ARGS.public_path_as_str())))
                .finish());
        }
    }

    remove_expired(&mut pastas);

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}

#[post("/remove/{id}")]
pub async fn post_remove(
    data: web::Data<AppState>,
    id: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let mut password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == "password" {
            while let Some(chunk) = field.try_next().await? {
                password = std::str::from_utf8(&chunk).unwrap().to_string();
            }
        }
    }

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {

            return Ok(HttpResponse::Found()
                .append_header((
                    "Location",
                    format!(
                        "{}/upload/{}",
                        ARGS.public_path_as_str(),
                        pastas[i].id_as_animals()
                    ),
                ))
                .finish());
        }
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}
