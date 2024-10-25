#[macro_use]
extern crate rocket;
use std::sync::atomic::AtomicUsize;
use rocket_db_pools::{sqlx::{self}, Database};
use services::{ exercise, stream_works, ws_works };
mod services;

#[derive(Database)]
#[database("platform_logs")]
pub struct MysqlLogs(sqlx::MySqlPool);

#[launch]
fn rocket() -> _ {
    rocket
        ::build()
        .attach(MysqlLogs::init())
        .mount(
            "/api",
            routes![
                exercise::index,
                exercise::get_str,
                exercise::post,
                exercise::post_json,
                exercise::post_with_body,
                exercise::normal_request,
                exercise::count,
                exercise::get_user,
                exercise::stream,
                exercise::infinite_hello,
            ]
        )
        .mount("/api/stream", routes![
            stream_works::hello,
            stream_works::byte,
            stream_works::build_maze_full,
            stream_works::build_maze
        ])
        .mount("/api/socket", routes![
            ws_works::build_maze // ws://localhost:8000/api/socket/maze
        ])
        .mount("/files", routes![exercise::pull_file])
        .register("/", catchers![exercise::token_error, exercise::internal_server_error])
        .manage(exercise::HitCount { count: AtomicUsize::new(0) })
}
