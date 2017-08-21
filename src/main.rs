extern crate iron;
extern crate router;
extern crate staticfile;
extern crate mount;
#[macro_use] extern crate diesel;
extern crate dotenv;
#[macro_use] extern crate diesel_codegen;

use std::path::Path;
use std::fs::File;
use std::io::Read;

use iron::prelude::*;
use iron::status;
use iron::headers::ContentType;
use iron::modifiers::Header;
use router::Router;
use staticfile::Static;
use mount::Mount;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod models;

use models::{Post, NewPost};

pub fn create_post<'a>(conn: &PgConnection, title: &'a str, body: &'a str) -> Post {
    use schema::posts;

    let new_post = NewPost {
        title: title,
        body: body,
        published: true
    };

    diesel::insert(&new_post).into(posts::table)
        .get_result(conn)
        .expect("Error saving new post")
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn index(_req: &mut Request) -> IronResult<Response> {
    let mut file = File::open("public/index.html").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    Ok(Response::with((status::Ok, Header(ContentType::html()), contents)))
}

fn main() {
    let mut router = Router::new();
    router.get("/", index, "index");

    use schema::posts::dsl::*;

    let connection = establish_connection();
    let results = posts.filter(published.eq(true))
        .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.title);
        println!("----------\n");
        println!("{}", post.body);
    }

    let post = create_post(&connection, "title", "kees");

    let mut mount = Mount::new();
    mount
        .mount("/", router)
        .mount("/public/", Static::new(Path::new("public/")));

    Iron::new(mount).http("localhost:3000").unwrap();
}
