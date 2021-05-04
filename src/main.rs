use std::io;
use std::fs;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_session::{
    CookieSession, Session
};
use actix_web::{
    App, http, HttpResponse, HttpServer, web,
};
use base64;
use serde::{
    Deserialize, Serialize,
};

#[cfg(test)]
mod server_test;

mod database;
use crate::database::{
    DiskCache, HBT,
    Table,
};

const SERV_PRIVATE_KEY: [u8; 32] = [0; 32];
macro_rules! default_user_table(
    () =>
    {{
        let mut utable = HBT::new();

        utable.set(String::from("test-user"),
                   User{ hpass: String::from("5f4dcc3b5aa765d61d8327deb882cf99") });

        utable
    }};
);

// ---- DataTypes ----

type UserKey = String;
type UserTable = HBT<UserKey, User>;

struct User
{
    // salt: String,
    hpass: String,
}

type ImageKey = String;
type ImageTable = DiskCache<ImageKey, Image>;

#[derive(Serialize)]
struct Image
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
    // public: Option<bool>,
    id: ImageKey,
    img: String,
}

#[derive(Deserialize)]
struct RmRequest
{
    id: ImageKey,
}

#[derive(Deserialize)]
struct ViewRequest
{
    id: ImageKey,
}

#[derive(Deserialize)]
struct LogonRequest
{
    uname: UserKey,
    hpass: String,
}

// ---- User Procedures ----

fn add_img(db: &mut Database, sess: &Session, req: &AddRequest) -> HttpResponse
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
            // TODO: Users should be able to specify public/private in the AddRequest
            public: false,
            // TODO: Fix below. This should be the id of the current user
            owner: String::from("User-Toke-Placeholder"),
            data: img_data,
        });

        HttpResponse::Ok().body(format!("Added {} to the database.", req.id))
    }
    else
    {
        HttpResponse::Conflict().body(format!("{} is already present in the database. Please use another id, or remove the existing value.", req.id))
    }
}

fn add_img_dispatch(db: web::Data<Mutex<Database>>, sess: Session, req: web::Json<AddRequest>) -> HttpResponse
{
    match db.lock()
    {
        Ok(mut db) =>
            add_img(&mut db, &sess, &req.into_inner()),
        Err(e) =>
            return HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

fn remove_img(db: &mut Database, req: &RmRequest) -> HttpResponse
{
    if db.icache.remove(&req.id).is_some()
    {
        HttpResponse::Ok().body(format!("Removed {} from the database.", req.id))
    }
    else
    {
        HttpResponse::NotAcceptable().body(format!("{} is not present in the database and cannot be removed.", req.id))
    }
}

fn remove_img_dispatch(db: web::Data<Mutex<Database>>, sess: Session, req: web::Json<RmRequest>) -> HttpResponse
{
    match db.lock()
    {
        Ok(mut db) =>
            remove_img(&mut db, &req.into_inner()),
        Err(e) =>
            return HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

fn view_img(db: &mut Database, sess: &Session, id: &String) -> HttpResponse
{
    if let Some(img) = db.icache.get(id)
    {
        HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "image/jpeg")
            .body(img.data.clone())
    }
    else
    {
        HttpResponse::NotFound().body(format!("We couldn't find {}", id))
    }
}

fn view_img_dispatch(db: web::Data<Mutex<Database>>, sess: Session, req: web::Path<String>) -> HttpResponse
{
    match db.lock()
    {
        Ok(mut db) =>
            view_img(&mut db, &sess, &req.into_inner()),
        Err(e) =>
            return HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

fn user_img_summary(db: web::Data<Mutex<Database>>) -> HttpResponse
{
    HttpResponse::NotImplemented().finish()
    // match db.lock()
    // {
    //     Ok(db) =>
    //         HttpResponse::Ok().body(format!("Your database contains ?? records ({} cached).", db.icache.len())),
    //     Err(e) =>
    //         return HttpResponse::InternalServerError().body(format!("{:?}", e)),
    // }
}

// ---- Admin Procedures ----

fn admin_img_summary(db: web::Data<Mutex<Database>>) -> HttpResponse
{
    HttpResponse::NotImplemented().finish()
    // match db.lock()
    // {
    //     Ok(db) =>
    //         HttpResponse::Ok().body(format!("The database contains ?? records ({} cached).", db.icache.len())),
    //     Err(e) =>
    //         return HttpResponse::InternalServerError().body(format!("{:?}", e)),
    // }
}

fn admin_db_persist(db: web::Data<Mutex<Database>>) -> HttpResponse
{
    HttpResponse::NotImplemented().finish()
}

fn admin_db_restore(db: web::Data<Mutex<Database>>) -> HttpResponse
{
    HttpResponse::NotImplemented().finish()
}

// ---- User & Admin Procedures ----

fn logon(db: &mut Database, sess: &mut Session, req: &LogonRequest) -> HttpResponse
{
    // TODO:
    // Hello down there!
    if let Some(db_user) = db.utable.get(&req.uname)
    {
        if db_user.hpass == req.hpass
        {
            match sess.set("auth-uname", req.uname.clone())
            {
                Ok(()) =>
                    // Hiiii!
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
            return HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

fn logoff(db: &mut Database, sess: &mut Session) -> HttpResponse
{
    sess.remove("auth-uname");

    HttpResponse::Ok().body("Goodbye friend.")
}

fn logoff_dispatch(db: web::Data<Mutex<Database>>, mut sess: Session) -> HttpResponse
{
    match db.lock()
    {
        Ok(mut db) =>
            logoff(&mut db, &mut sess),
        Err(e) =>
            return HttpResponse::InternalServerError().body(format!("{:?}", e)),
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
    let db_base_path = std::env::current_dir()?;
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
            .app_data(img_store.clone())
            // User Endpoints
            .route("/add",             web::post().to(add_img_dispatch))
            .route("/remove",          web::delete().to(remove_img_dispatch))
            .route("/summary",         web::get().to(user_img_summary))
            .route("/view/{image_id}", web::get().to(view_img_dispatch))
            // Admin Endponts
            .route("/admin/summary",   web::get().to(admin_img_summary))
            .route("/admin/backup",    web::get().to(admin_db_persist))
            .route("/admin/restore",   web::get().to(admin_db_restore))
            // User & Admin Endpoints
            .route("/logon",           web::post().to(logon_dispatch))
            .route("/logoff",          web::post().to(logoff_dispatch))
            // Web Endpoints
            .route("/",                web::get().to(|| {file("public/index.html")}))
            .route("/style.css",       web::get().to(|| {file("public/style.css")}))
            .route("*",                web::get().to(|| {file("public/404.html")}))
            .wrap(
                Cors::default()
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
