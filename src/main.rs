#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::futures::TryStreamExt;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{fairing, Build, Rocket};
use rocket_db_pools::{sqlx, Connection, Database};

use rocket::http::Status;
use rocket::response::status::Created;
use std::result;
use std::time::SystemTime;

type Result<T, E = rocket::response::Debug<sqlx::Error>> = result::Result<T, E>;

#[derive(Database)]
#[database("thingsdb")]
struct ThingsDb(sqlx::SqlitePool);

#[get("/")]
fn index() -> Json<String> {
    Json("Hi there".to_string())
}

#[get("/things")]
async fn list(mut db: Connection<ThingsDb>) -> Result<Json<Vec<i64>>> {
    let ids = sqlx::query!("SELECT id FROM things")
        .fetch(&mut *db)
        .map_ok(|record| record.id)
        .try_collect::<Vec<_>>()
        .await?
        .into_iter().flatten()
        .collect();

    Ok(Json(ids))
}

#[post("/things", data = "<url>")]
async fn add_thing(mut db: Connection<ThingsDb>, url: &str) -> Result<Created<&str>> {
    sqlx::query!("INSERT INTO things (url) values (?)", url)
        .execute(&mut *db)
        .await?;
    Ok(Created::new("/things").body(url))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(ThingsDb::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .mount("/", routes![index, list, add_thing])
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match ThingsDb::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Thing {
    id: i64,
    url: String,
    added: SystemTime,
    tags: Vec<String>,
}
