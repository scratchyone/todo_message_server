#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use argon2::{self, Config};
use postgres::{Client, NoTls};
use rocket::http::Method;
use rocket_contrib::json::Json;
use rocket_cors;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct UsersRequest {
    token: String,
}
#[derive(Serialize, Deserialize)]
struct UserRequest {
    token: String,
    username: String,
}
#[derive(Serialize, Deserialize)]
struct Todo {
    username: String,
    todo: String,
    done: bool,
    id: String,
}
#[get("/")]
fn index() -> &'static str {
    "online"
}
#[post("/users", format = "application/json", data = "<data>")]
fn users(data: Json<UsersRequest>) -> Json<serde_json::Value> {
    let mut client = Client::connect("host=db user=postgres password=example", NoTls).unwrap();
    if let Ok(_) = client.query_one(
        "SELECT * FROM tokens WHERE token = $1 AND username = 'admin'",
        &[&data.token],
    ) {
        let users: Vec<String> = client
            .query("SELECT username FROM users", &[])
            .unwrap()
            .iter()
            .map(|x| x.get("username"))
            .collect();
        Json(serde_json::json!({
            "error": false,
            "users": users
        }))
    } else {
        Json(serde_json::json!({
            "error": true,
            "error_message": "Invalid token"
        }))
    }
}
#[post("/user", format = "application/json", data = "<data>")]
fn user(data: Json<UserRequest>) -> Json<serde_json::Value> {
    let mut client = Client::connect("host=db user=postgres password=example", NoTls).unwrap();
    if let Ok(_) = client.query_one(
        "SELECT * FROM tokens WHERE token = $1 AND username = 'admin'",
        &[&data.token],
    ) {
        let users: Vec<Todo> = client
            .query(
                "SELECT * FROM todos WHERE username = $1 ORDER BY num",
                &[&data.username],
            )
            .unwrap()
            .iter()
            .map(|x| Todo {
                username: x.get("username"),
                todo: x.get("todo"),
                done: x.get("done"),
                id: x.get("id"),
            })
            .collect();
        Json(serde_json::json!({
            "error": false,
            "users": users
        }))
    } else {
        Json(serde_json::json!({
            "error": true,
            "error_message": "Invalid token"
        }))
    }
}
fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        "https://scratchyone.com",
        "https://www.scratchyone.com",
    ]);

    CorsOptions {
        // 5.
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

fn main() {
    thread::sleep(time::Duration::from_millis(2000));
    let mut client = Client::connect("host=db user=postgres password=example", NoTls).unwrap();
    let cfg = rocket::config::Config::build(rocket::config::Environment::Development)
        .port(80)
        .address("0.0.0.0")
        .unwrap();
    rocket::custom(cfg)
        .attach(make_cors())
        .mount("/", routes![users, index, user])
        .launch();
}
