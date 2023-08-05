#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::futures::TryStreamExt;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{fairing, Build, Rocket, Request};
use rocket_db_pools::{sqlx, Connection, Database};
use rocket::response::status::{BadRequest, Created};
use std::result;
use std::time::SystemTime;
use rocket::response::Responder;
use crate::AddThingError::DatabaseError;

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

#[derive(Responder)]
enum AddThingError {
    #[response(status = 400)]
    NotAValidURL(String),
    #[response(status = 500)]
    DatabaseError(String),
}

#[post("/things", data = "<url>")]
async fn add_thing(mut db: Connection<ThingsDb>, url: &str) -> result::Result<Created<&str>, AddThingError> {
    let parsed_url = url::Url::parse(url);
    if parsed_url.is_ok() {
        sqlx::query!("INSERT INTO things (url) values (?)",url)
            .execute(&mut *db)
            .await.map_err(|e| DatabaseError("database error".to_string()))?;
        Ok(Created::new("/things").body("Added".as_ref()))
    } else {
        Err(AddThingError::NotAValidURL( format!("Not a URL: {}",url.to_string())))
    }
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
