#[macro_use]
extern crate rocket;

use crate::AddThingError::DatabaseError;
use chrono::NaiveDateTime;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::futures::TryStreamExt;
use rocket::response::status::{Created, NoContent};
use rocket::response::Responder;
use rocket::serde::{Deserialize, Serialize};
use rocket::{fairing, Build, Rocket};
use rocket_db_pools::{sqlx, Connection, Database};
use rocket_dyn_templates::{context, Template};
use std::result;
use rocket::fs::FileServer;

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
    Rss,
    File,
}

#[get("/")]
async fn index(mut db: Connection<ThingsDb>) -> Result<Template> {
    let things = sqlx::query_as!(
        Thing,
        r#"SELECT things.id, url, added as "added: _", type as "type_: Type" FROM things "#
    )
    .fetch(&mut **db)
    .try_collect::<Vec<_>>()
    .await?;
    Ok(Template::render("index", context! {things: things}))
}

#[derive(Responder)]
enum AddThingError {
    #[response(status = 400)]
    NotAValidURL(String),
    #[response(status = 500)]
    DatabaseError(String),
}

#[derive(FromForm)]
struct AddThingForm<'a> {
    url: &'a str,
}

#[post("/things", data = "<add_thing_form>")]
async fn add_thing(
    mut db: Connection<ThingsDb>,
    add_thing_form: Form<AddThingForm<'_>>,
) -> result::Result<Created<Template>, AddThingError> {
    let parsed_url = url::Url::parse(add_thing_form.url);
    if parsed_url.is_ok() {
        sqlx::query!("INSERT INTO things (url) values (?)", add_thing_form.url)
            .execute(&mut **db)
            .await
            .map_err(|e| DatabaseError(format!("database error: {}", e)))?;
        Ok(Created::new("/things").body(Template::render(
            "thingrow",
            context! {url: add_thing_form.url, id:77},
        )))
    } else {
        Err(AddThingError::NotAValidURL(format!(
            "Not a URL: {}",
            add_thing_form.url
        )))
    }
}

#[delete("/things/<id>")]
async fn delete_thing(mut db: Connection<ThingsDb>, id: i32) -> Result<NoContent> {
    sqlx::query!("DELETE FROM things WHERE id=?", id)
        .execute(&mut **db)
        .await?;
    Ok(NoContent)
}

#[get("/healthz")]
async fn healthz(mut db: Connection<ThingsDb>) -> Result<NoContent> {
    sqlx::query("SELECT TIME()").execute(&mut **db).await?;
    Ok(NoContent)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/static",FileServer::from("static"))
        .attach(ThingsDb::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .attach(Template::fairing())
        .mount("/", routes![index, healthz, add_thing, delete_thing])
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
