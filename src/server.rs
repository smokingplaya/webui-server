use std::{path::Path, sync::atomic::Ordering::Relaxed};
use actix_web::{http::header::{ContentDisposition, DispositionType}, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::Context;
use mime_guess::from_path;
use crate::ALREADY_BINDED;

const BASE_SEARCH_PATH: &str = "garrysmod/webui/";

async fn search(req: HttpRequest) -> anyhow::Result<HttpResponse> {
  let path = req.match_info()
    .get("file_path")
    .context(anyhow::anyhow!("file_path is not provided"))?;

  let path_full = Path::new(BASE_SEARCH_PATH).join(path);

  if !path_full.is_file() {
    return Ok(HttpResponse::NotFound().body("File not found"));
  }

  let mime_type = from_path(&path_full).first_or_octet_stream();

  let file_content = std::fs::read(&path_full)
    .context(anyhow::anyhow!("Failed to read file"))?;

  Ok(HttpResponse::Ok()
    .content_type(mime_type.as_ref())
    .insert_header(ContentDisposition {
      disposition: DispositionType::Inline,
      parameters: vec![],
    })
    .body(file_content))
}

async fn searcher(req: HttpRequest) -> impl Responder {
  search(req).await
    .map_err(|_| HttpResponse::BadRequest().finish())
    .unwrap_or_else(|err| err)
}

pub(crate) async fn run(
  port: isize
) -> anyhow::Result<()> {
  HttpServer::new(|| {
    App::new()
      .route("/{file_path:.*}", web::get().to(searcher))
  })
  .bind(&format!("0.0.0.0:{port}"))?
  .run()
  .await?;

  ALREADY_BINDED.store(false, Relaxed);

  Ok(())
}