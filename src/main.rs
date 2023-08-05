#[macro_use]
extern crate rocket;

use crate::AddThingError::DatabaseError;
use chrono::NaiveDateTime;
use rocket::fairing::AdHoc;
use rocket::futures::TryStreamExt;
use rocket::response::status::Created;
use rocket::response::Responder;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{fairing, Build, Rocket};
use rocket_db_pools::{sqlx, Connection, Database};
use std::result;

type Result<T, E = rocket::response::Debug<sqlx::Error>> = result::Result<T, E>;

#[derive(Database)]
#[database("thingsdb")]
struct ThingsDb(sqlx::SqlitePool);

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Thing {
    pub id: i64,
    pub url: String,
    pub added: NaiveDateTime,
    // pub tags: Vec<Tag>,
    type_: Type,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Tag {
    pub id: i64,
    pub tag: String,
}

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(rename_all = "lowercase")]
enum Type {
    Article,
    Youtube,
    Podcast,
    RSS,
    File,
}

#[get("/")]
fn index() -> Json<String> {
    Json("Hi there".to_string())
}

#[get("/things")]
async fn list(mut db: Connection<ThingsDb>) -> Result<Json<Vec<Thing>>> {
    let things = sqlx::query_as!(
        Thing,
        r#"SELECT things.id, url, added,type as "type_: Type" FROM things "#
    )
    .fetch(&mut *db)
    .try_collect::<Vec<_>>()
    .await?;

    Ok(Json(things))
}

#[derive(Responder)]
enum AddThingError {
    #[response(status = 400)]
    NotAValidURL(String),
    #[response(status = 500)]
    DatabaseError(String),
}

#[post("/things", data = "<url>")]
async fn add_thing(
    mut db: Connection<ThingsDb>,
    url: &str,
) -> result::Result<Created<&str>, AddThingError> {
    let parsed_url = url::Url::parse(url);
    if parsed_url.is_ok() {
        sqlx::query!("INSERT INTO things (url) values (?)", url)
            .execute(&mut *db)
            .await
            .map_err(|e| DatabaseError(format!("database error: {}", e).to_string()))?;
        Ok(Created::new("/things").body("Added".as_ref()))
    } else {
        Err(AddThingError::NotAValidURL(format!(
            "Not a URL: {}",
            url.to_string()
        )))
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
