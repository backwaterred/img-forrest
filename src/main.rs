use std::collections::HashMap;
use std::io;
use std::fs;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_session::{
    CookieSession, Session
};
use actix_web::{
    App, HttpResponse, HttpServer, web,
};
use base64;
use serde::{
    Deserialize, Serialize,
};

#[cfg(test)]
mod server_test;

const SERV_PRIVATE_KEY: [u8; 32] = [0; 32];
macro_rules! default_user_table(
    () =>
    {{
        let mut utable = HashMap::new();

        utable.insert(String::from("test-user"),
                      User{ hpass: String::from("5f4dcc3b5aa765d61d8327deb882cf99") });

        utable
    }};
);

// ---- DataTypes ----

type UserToken = String;
type UserTable = HashMap<UserToken, User>;

struct User
{
    // salt: String,
    hpass: String,
}

type ImageToken = String;
type ImageCache = HashMap<ImageToken, Image>;
//type ImageCache = HashMap<UserToken, HashMap<ImageToken, Image>>;

struct Image
{
    public: bool,
    owner: UserToken,
    data: Vec<u8>,
}

struct Database
{
    utable: UserTable,
    icache: ImageCache,
}

#[derive(Deserialize)]
struct AddRequest
{
    id: ImageToken,
    img: String,
}

#[derive(Deserialize)]
struct RmRequest
{
    id: ImageToken,
}
#[derive(Deserialize)]
struct LogonRequest
{
    uname: UserToken,
    hpass: String,
}

// ---- User Procedures ----

fn add_img(db: &mut Database, req: &AddRequest) -> HttpResponse
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

        db.icache.insert(req.id.clone(), Image {
            public: false,
            owner: String::from("User-Token-Placeholder"),
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
            add_img(&mut db, &req.into_inner()),
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
    let img_store =
        web::Data::new(
            Mutex::new(
                Database {
                    utable: default_user_table!(),
                    icache: ImageCache::new(),
                }));

    HttpServer::new(move || {
        App::new()
            .wrap(
                CookieSession::private(&SERV_PRIVATE_KEY)
                    .secure(false),
            )
            .app_data(img_store.clone())
            // User Endpoints
            .route("/add",           web::post().to(add_img_dispatch))
            .route("/remove",        web::delete().to(remove_img_dispatch))
            .route("/summary",       web::get().to(user_img_summary))
            // Admin Endponts
            .route("/admin/summary", web::get().to(admin_img_summary))
            .route("/admin/backup",  web::get().to(admin_db_persist))
            .route("/admin/restore", web::get().to(admin_db_restore))
            // User & Admin Endpoints
            .route("/logon",         web::post().to(logon_dispatch))
            .route("/logoff",        web::post().to(logoff_dispatch))
            // Web Endpoints
            .route("/",              web::get().to(|| {file("public/index.html")}))
            .route("/style.css",     web::get().to(|| {file("public/style.css")}))
            .wrap(
                Cors::default()
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
