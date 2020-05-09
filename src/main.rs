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
struct AddRequest {
    token: String,
    message: String,
    block: bool,
}
#[get("/")]
fn index() -> &'static str {
    "online"
}
#[get("/messages", format = "application/json")]
fn messages() -> Json<serde_json::Value> {
    let mut client = Client::connect("host=db user=postgres password=example", NoTls).unwrap();
    let messages = client.query("SELECT * FROM messages", &[]).unwrap();
    let mut msgs = vec![];
    for msg in messages {
        msgs.push(serde_json::json!(
            {"text":msg.get::<&str, String>("message"),
            "uuid":msg.get::<&str, String>("uuid"),
            "block": msg.get::<&str, bool>("block")}
        ));
    }
    Json(serde_json::json!({ "messages": msgs }))
}
#[post("/add_message", format = "application/json", data = "<data>")]
fn add_message(data: Json<AddRequest>) -> Json<serde_json::Value> {
    let mut client = Client::connect("host=db user=postgres password=example", NoTls).unwrap();
    if let Ok(_) = client.query_one(
        "SELECT * FROM tokens WHERE token = $1 AND username = 'admin'",
        &[&data.token],
    ) {
        let uuid = Uuid::new_v4().to_string();
        client.execute("DELETE FROM messages", &[]).unwrap();
        if data.message != "" {
            client
                .execute(
                    "INSERT INTO messages VALUES ($1, $2, $3)",
                    &[&data.message, &uuid, &data.block],
                )
                .unwrap();
        }
        Json(serde_json::json!({
            "error": false
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
    client
        .batch_execute(
            "
CREATE TABLE IF NOT EXISTS messages (
    message text,
    uuid text,
    block boolean
);",
        )
        .unwrap();
    let cfg = rocket::config::Config::build(rocket::config::Environment::Development)
        .port(80)
        .address("0.0.0.0")
        .unwrap();
    rocket::custom(cfg)
        .attach(make_cors())
        .mount("/", routes![messages, index, add_message])
        .launch();
}
