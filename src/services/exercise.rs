use rocket::{ form::Form, fs::NamedFile, http::Status, request::{FromRequest,Outcome}, response::stream::{ReaderStream, TextStream}, tokio::{net::TcpStream, time::{interval, Duration}}, Request, State };
use rocket_db_pools::Connection;
use rocket_db_pools::sqlx::{self, Row};
use core::fmt;
use std::{ collections::HashMap, fmt::Display, io, net::SocketAddr, path::{ Path, PathBuf }, sync::{ atomic::{AtomicUsize, Ordering}, Arc, Mutex }, thread };
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[get("/index")]
pub fn index() -> String {
    let count = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for _ in 0..10 {
        let count = Arc::clone(&count);
        let handle = thread::spawn(move || {
            let mut num = count.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let res = *count.lock().unwrap();
    res.to_string()
}

// get query传参
struct FormatList(Vec<String>);
impl fmt::Display for FormatList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self.0
            .iter()
            .enumerate()
            .map(|(index, value)| { format!("{}: {}", index, value) })
            .collect::<Vec<String>>();
        write!(f, "[{}]", str.join(", "))
    }
}
#[get("/str?<str>")]
pub fn get_str(str: Option<String>) -> String {
    if let Some(str) = str {
        let vec = str.split(",").map(String::from).collect::<Vec<String>>();
        let list = FormatList(vec);
        format!("{}", list)
    } else {
        String::from("empty")
    }
}

// post body传参
#[post("/post-with-body", data = "<body_data>")]
pub fn post_with_body(body_data: String) -> String {
    body_data
}

// post form传参
#[derive(FromForm, Debug)]
pub struct PostForm {
    #[field(default = "rust")]
    lang: String,
    body: String,
}
impl Display for PostForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lang: {}, body: {}", self.lang, self.body)
    }
}
#[post("/post", data = "<form_data>")]
pub fn post(form_data: Form<PostForm>) -> String {
    form_data.to_string()
}

// post json传参
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PostJson {
    lang: String,
    body: String,
}
impl Display for PostJson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lang: {}, body: {}", self.lang, self.body)
    }
}
#[post("/post-json", data = "<json_data>")]
pub fn post_json(json_data: Json<PostJson>) -> String {
    json_data.to_string()
}

// get 静态文件
// 简易方法
// use rocket::fs::FileServer;
// #[launch]
// fn rocket() -> _ {
//     rocket::build()
//          // serve files from `/www/static` at path `/public`
//         .mount("/public", FileServer::from("/www/static"))
// }
#[get("/pull-file/<file..>")]
pub async fn pull_file(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).await.ok()
}

// 常用request方法及常用response
#[derive(Debug, Serialize)]
pub struct GlobalHeaders {
    pub token: String
}
#[derive(Debug)]
pub enum HeaderError {
    TokenMiss,
    BadToken
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for GlobalHeaders {
    type Error = HeaderError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        fn is_valid(key: &str) -> bool {
            key == "123"
        }

        match req.headers().get_one("token") {
            None => Outcome::Error((Status::new(555), Self::Error::TokenMiss)),
            Some(key) if is_valid(key) => Outcome::Success(GlobalHeaders{token: key.to_string()}),
            Some(_) => Outcome::Error((Status::new(555), Self::Error::BadToken)),
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct NormalResponse {
    id: i32,
    name: String,
    json_data: HashMap<String,String>,
    headers: GlobalHeaders
}

#[post("/normal-request/<id>?<name>", format="application/json", data = "<json_data>")]
pub async fn normal_request(
    id: i32,
    name: Option<String>,
    json_data: Json<HashMap<String,String>>,
    headers: GlobalHeaders
) -> Json<NormalResponse> {
    Json(NormalResponse {
        id,
        name: name.unwrap_or(String::from("")),
        json_data: json_data.into_inner(),
        headers
    })
}

#[catch(555)]
pub fn token_error() -> &'static str {
    "token error"
}
#[catch(500)]
pub fn internal_server_error() -> &'static str {
    "internal server error"
}

// 公共状态
pub struct HitCount {
    pub count: AtomicUsize
}
#[get("/count")]
pub fn count(hit_count: &State<HitCount>) -> String {
    let current_count = hit_count.count.load(Ordering::Relaxed);
    hit_count.count.fetch_add(1, Ordering::Relaxed);
    format!("hit count: {}", current_count)
}


// 读取mysql
#[get("/user/<id>")]
pub async fn get_user(mut db: Connection<crate::MysqlLogs>, id: i64) -> Option<String> {
    sqlx::query("SELECT * FROM sys_user WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut **db).await
        .unwrap()
        .map(|row| row.get("nickname"))
}

// stream
#[get("/stream")]
pub async fn stream() -> io::Result<ReaderStream![TcpStream]> {
    let addr = SocketAddr::from(([127,0,0,1], 8000));
    let stream = TcpStream::connect(addr).await?;
    Ok(ReaderStream::one(stream))
}

#[get("/infinite-hello")]
pub fn infinite_hello() -> TextStream![&'static str] {
    TextStream! {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            yield "hello";
            interval.tick().await;
        }
    }
}