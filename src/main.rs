#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use] extern crate log;
use rocket::{response::content, State};

use juniper::{
    Context, EmptyMutation, RootNode,
};

#[derive(Clone)]
struct Ctx {
    db: sled::Db,
}

impl Context for Ctx {}

#[derive(Clone, juniper::GraphQLObject)]
struct Recipe {
    title: String,
    nb_person: i32,
    ingredients: Vec<String>,
    steps: Vec<String>,
}

struct Query;

#[juniper::object(Context = Ctx)]
impl Query {
    fn list_recipes(context: &Ctx) -> &Vec<Recipe> {
        &context.db.scan_prefix("recipes/").
    }
}

type Schema = RootNode<'static, Query, EmptyMutation<Ctx>>;

#[rocket::get("/")]
fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
}

#[rocket::get("/graphql?<request>")]
fn get_graphql_handler(
    context: State<Ctx>,
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &context)
}

#[rocket::post("/graphql", data = "<request>")]
fn post_graphql_handler(
    context: State<Ctx>,
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &context)
}

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db = sled::open("shef.db").expect("Can't open the DB :((((");

    info!("shef.top started!");
    rocket::ignite()
        .manage(Ctx { db })
        .manage(Schema::new(Query, EmptyMutation::new()))
        .mount(
            "/",
            rocket::routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        .launch();
}
