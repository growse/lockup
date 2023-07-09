#[macro_use]
extern crate rocket;

use std::error::Error;
use std::path::Path;
use std::result;
use std::sync::Mutex;
use std::time::Instant;

use rocket::http::ext::IntoCollection;
use rocket::{Build, Ignite, Rocket, State};
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use url::Url;

struct Config {
    db: Mutex<Connection>,
}

type Result<T> = result::Result<T, Box<dyn Error>>;

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
struct SuccessJson(&'static str);

#[get("/")]
fn index() -> SuccessJson {
    SuccessJson("Hello, world!")
}

#[get("/things")]
fn list() -> SuccessJson {
    SuccessJson("hi")
}

#[launch]
fn rocket() -> _ {
    let db = init_db(Path::new("lockup.sqlite")).expect("Bzzt");
    let config = Config { db: Mutex::new(db) };
    rocket::build()
        .mount("/", routes![index, list])
        .manage(config)
}

fn init_db(path: &Path) -> Result<Connection> {
    // let migrations = Migrations::new(vec![M::up(
    //     "CREATE TABLE things(id int not null, url text not null, added text not null);",
    // )]);
    let mut conn = Connection::open(path)?;
    // conn.pragma_update(None, "journal_mode", &"WAL")?;
    // migrations.to_latest(&mut conn)?;
    Ok(conn)
}

#[derive(Debug)]
struct Thing {
    id: i32,
    url: Url,
    added: Instant,
    tags: Vec<String>,
}
