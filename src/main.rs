use std::io;
use std::fs;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_session::{
    CookieSession, Session
};
use actix_web::{
    App, http, HttpResponse, HttpServer,
    web,
};
use base64;
use serde::{
    Deserialize, Serialize,
};

mod auth;
mod database;
use crate::database::{
    DiskCache, HBT,
    Table,
};
#[cfg(test)]
mod server_test;

const SERV_PRIVATE_KEY: [u8; 32] = [0; 32];

#[macro_export]
macro_rules! default_user_table(
    () =>
    {{
        let mut utable = HBT::new();

        utable.set(String::from("leo"),
                   User{ hpass: String::from("5f4dcc3b5aa765d61d8327deb882cf99") });
        utable.set(String::from("sphinx"),
                   User{ hpass: String::from("5f4dcc3b5aa765d61d8327deb882cf99") });
        utable.set(String::from("tony"),
                   User{ hpass: String::from("5f4dcc3b5aa765d61d8327deb882cf99") });

        utable
    }};
);

// ---- DataTypes ----

pub type UserKey = String;
type UserTable = HBT<UserKey, User>;

pub struct User
{
    // salt: String,
    hpass: String,
}

pub type ImageKey = String;
type ImageTable = DiskCache<ImageKey, Image>;

#[derive(Deserialize, Serialize)]
pub struct Image
{
    public: bool,
    owner: UserKey,
    data: Vec<u8>,
}

struct Database
{
    utable: UserTable,
    icache: ImageTable,
}

#[derive(Deserialize)]
struct AddRequest
{
    public: Option<bool>,
    id: ImageKey,
    img: String,
}

#[derive(Deserialize)]
struct RmRequest
{
    id: ImageKey,
}

#[derive(Deserialize)]
pub struct LogonRequest
{
    uname: UserKey,
    hpass: String,
}

// ---- User Procedures ----

fn add_img(db: &mut Database, sess: &Session, req: &AddRequest) -> HttpResponse
{
    if let Some(auth_user) = auth::get_auth_user(sess)
    {
        if !db.icache.contains_key(&req.id)
        {
            let img_data: Vec<u8> = match base64::decode(&req.img)
            {
                Ok(data) =>
                    data,
                Err(e) =>
                    return HttpResponse::InternalServerError().body(format!("{:?}", e)),
            };

            db.icache.set(req.id.clone(), Image {
                public: req.public.unwrap_or(false),
                owner: auth_user,
                data: img_data,
            });

            HttpResponse::Ok().body(format!("Added {} to the database.", req.id))
        }
        else
        {
            HttpResponse::Conflict().body(format!("{} is already present in the database. Please use another id, or remove the existing value.", req.id))
        }
    }
    else
    {
        HttpResponse::Unauthorized().finish()
    }
}

fn add_img_dispatch(db: web::Data<Mutex<Database>>, sess: Session, req: web::Json<AddRequest>) -> HttpResponse
{
    match db.lock()
    {
        Ok(mut db) =>
            add_img(&mut db, &sess, &req.into_inner()),
        Err(e) =>
            HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

fn remove_img(db: &mut Database, sess: &Session, req: &RmRequest) -> HttpResponse
{
    if let Some(_auth_user) = auth::get_auth_user(sess)
    {
        if db.icache.remove(&req.id).is_some()
        {
            HttpResponse::Ok().body(format!("Removed {} from the database.", req.id))
        }
        else
        {
            HttpResponse::Conflict().body(format!("{} is not present in the database and cannot be removed.", req.id))
        }
    }
    else
    {
        HttpResponse::Unauthorized().finish()
    }
}

fn remove_img_dispatch(db: web::Data<Mutex<Database>>, sess: Session, req: web::Json<RmRequest>) -> HttpResponse
{
    match db.lock()
    {
        Ok(mut db) =>
            remove_img(&mut db, &sess, &req.into_inner()),
        Err(e) =>
            HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

fn view_img(db: &mut Database, sess: &Session, img_id: &String) -> HttpResponse
{
    match (auth::get_auth_user(sess), db.icache.get(img_id))
    {
        (_, None) =>
            HttpResponse::NotFound().body(format!("We couldn't find {}", img_id)),
        (None, Some(img)) =>
            if img.public
            {
                HttpResponse::Ok()
                    .header(http::header::CONTENT_TYPE, "image/jpeg")
                    .body(img.data.clone())
            }
            else
            {
                HttpResponse::Unauthorized().finish()
            },
        (Some(auth_user), Some(img)) =>
            if img.public || (auth_user == img.owner)
            {
                HttpResponse::Ok()
                    .header(http::header::CONTENT_TYPE, "image/jpeg")
                    .body(img.data.clone())
            }
            else
            {
                HttpResponse::Unauthorized().finish()
            },
    }
}

fn view_img_dispatch(db: web::Data<Mutex<Database>>, sess: Session, req: web::Path<String>) -> HttpResponse
{
    match db.lock()
    {
        Ok(mut db) =>
            view_img(&mut db, &sess, &req.into_inner()),
        Err(e) =>
            HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

fn logon(db: &mut Database, sess: &mut Session, req: &LogonRequest) -> HttpResponse
{
    if let Some(db_user) = db.utable.get(&req.uname)
    {
        if db_user.hpass == req.hpass
        {
            match auth::authorize_user(sess, req.uname.clone())
            {
                Ok(()) =>
                    HttpResponse::Ok().body(format!("Hello {}, nice to see you again.", req.uname)),
                Err(e) =>
                    HttpResponse::InternalServerError().body(format!("{:?}", e)),
            }
        }
        else
        {
            HttpResponse::Unauthorized().finish()
        }
    }
    else
    {
        HttpResponse::Unauthorized().finish()
    }
}

fn logon_dispatch(db: web::Data<Mutex<Database>>, mut sess: Session, req: web::Json<LogonRequest>) -> HttpResponse
{
    match db.lock()
    {
        Ok(mut db) =>
            logon(&mut db, &mut sess, &req.into_inner()),
        Err(e) =>
            HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

fn logoff(db: &mut Database, sess: &mut Session) -> HttpResponse
{
    sess.remove("auth-user");

    match db.icache.persist()
    {
        Ok(_) =>
            HttpResponse::Ok().body("Goodbye friend."),
        Err(e) =>
            HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

fn logoff_dispatch(db: web::Data<Mutex<Database>>, mut sess: Session) -> HttpResponse
{
    match db.lock()
    {
        Ok(mut db) =>
            logoff(&mut db, &mut sess),
        Err(e) =>
            HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

// ---- Helper(s) ----

fn file(f_name: &str) -> HttpResponse {
    match fs::read_to_string(f_name) {
        Ok(content) =>
            HttpResponse::Ok().body(content),
        Err(e) =>
            HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

// ---- Main ----

#[actix_web::main]
async fn main() -> io::Result<()>
{
    let mut db_base_path = std::env::current_dir()?;
    db_base_path.push("live-db");

    let img_store =
        web::Data::new(
            Mutex::new(
                Database {
                    utable: default_user_table!(),
                    icache: ImageTable::new(db_base_path),
                }));

    HttpServer::new(move || {
        App::new()
            .wrap(
                CookieSession::private(&SERV_PRIVATE_KEY)
                    .secure(false),
            )
           .app_data(
               web::JsonConfig::default()
                   .limit(1024*1024)
           )
            .app_data(img_store.clone())
            // User Endpoints
            .route("/logon",           web::post().to(logon_dispatch))
            .route("/logoff",          web::post().to(logoff_dispatch))
            .route("/add",             web::post().to(add_img_dispatch))
            .route("/remove",          web::delete().to(remove_img_dispatch))
            .route("/view/{image_id}", web::get().to(view_img_dispatch))
            // Web Endpoints
            .route("/",                web::get().to(|| {file("public/index.html")}))
            .route("/style.css",       web::get().to(|| {file("public/style.css")}))
            .route("*",                web::get().to(|| {file("public/404.html")}))
            .wrap(
                Cors::default()
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
