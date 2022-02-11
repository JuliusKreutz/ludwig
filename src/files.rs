use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use actix_files::{Directory, Files};
use actix_multipart::Multipart;
use actix_session::{Session, UserSession};
use actix_web::{
    dev::ServiceResponse,
    post,
    web::{self, Data, Json},
    HttpRequest, HttpResponse, Responder,
};
use futures::{StreamExt, TryStreamExt};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::auth::{self, User};

#[derive(Deserialize, Serialize)]
struct Info {
    name: String,
    path: String,
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(remove).service(create).service(upload).service(
        Files::new("/", "files")
            .show_files_listing()
            .files_listing_renderer(renderer),
    );
}

fn renderer(dir: &Directory, req: &HttpRequest) -> Result<ServiceResponse, io::Error> {
    let path = Path::new("/").join(
        dir.path
            .strip_prefix(env::current_dir()?)
            .unwrap()
            .strip_prefix("files")
            .unwrap(),
    );

    let mut path_string = path.display().to_string();
    if !path_string.ends_with('/') {
        path_string.push('/');
    }

    let mut files = Vec::new();
    let mut folders = Vec::new();

    for entry in fs::read_dir(&dir.path)?.flatten() {
        let metadata = fs::metadata(entry.path())?;
        let name = entry.file_name().into_string().unwrap();

        if metadata.is_file() {
            files.push(Info {
                path: path_string.replace('\'', "\\'"),
                name,
            });
        } else if metadata.is_dir() {
            folders.push(Info {
                path: path_string.clone(),
                name,
            });
        }
    }

    let parent = if path_string == "/" {
        "".to_string()
    } else {
        path.parent()
            .map_or("/".to_string(), |p| p.display().to_string())
    };

    let mut name = "files";

    if let Ok(Some(auth)) = req.get_session().get::<String>("auth") {
        if let Ok(user) = serde_json::from_str::<User>(&auth) {
            if auth::is_ludwig(&user) {
                name = "admin";
            }
        }
    }

    Ok(ServiceResponse::new(
        req.clone(),
        HttpResponse::Ok().body(
            req.app_data::<Data<Handlebars<'_>>>()
                .unwrap()
                .render(
                    name,
                    &json!({"parent": parent, "folders": folders, "files": files}),
                )
                .unwrap(),
        ),
    ))
}

#[post("remove")]
async fn remove(session: Session, info: Json<Info>) -> impl Responder {
    if let Ok(Some(auth)) = session.get::<String>("auth") {
        if let Ok(user) = serde_json::from_str::<User>(&auth) {
            if auth::is_ludwig(&user) {
                let path = format!("files{}/{}", info.path, info.name);
                let path = Path::new(&path);

                if path.is_file() {
                    let _ = fs::remove_file(path);
                } else if path.is_dir() {
                    let _ = fs::remove_dir_all(path).unwrap();
                }
            }
        }
    }

    HttpResponse::Ok()
}

#[post("create")]
async fn create(session: Session, info: Json<Info>) -> impl Responder {
    if let Ok(Some(auth)) = session.get::<String>("auth") {
        if let Ok(user) = serde_json::from_str::<User>(&auth) {
            if auth::is_ludwig(&user) {
                let _ = fs::create_dir(format!("files{}/{}", info.path, info.name));
            }
        }
    }

    HttpResponse::Ok()
}

#[post("/upload")]
async fn upload(session: Session, mut payload: Multipart) -> impl Responder {
    if let Ok(Some(auth)) = session.get::<String>("auth") {
        if let Ok(user) = serde_json::from_str::<User>(&auth) {
            if auth::is_ludwig(&user) {
                let bytes = payload
                    .try_next()
                    .await
                    .unwrap()
                    .unwrap()
                    .next()
                    .await
                    .unwrap()
                    .unwrap();

                let path = core::str::from_utf8(&bytes).unwrap();

                while let Ok(Some(mut field)) = payload.try_next().await {
                    let filepath = format!(
                        "files{}/{}",
                        path,
                        field.content_disposition().get_filename().unwrap()
                    );

                    println!("{filepath}");

                    let mut f = web::block(|| File::create(filepath))
                        .await
                        .unwrap()
                        .unwrap();

                    while let Some(Ok(chunk)) = field.next().await {
                        let data = chunk;
                        f = web::block(move || f.write_all(&data).map(|_| f))
                            .await
                            .unwrap()
                            .unwrap();
                    }
                }
            }
        }
    }

    HttpResponse::Ok()
}
