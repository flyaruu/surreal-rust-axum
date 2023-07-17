use std::{net::SocketAddr, sync::{Mutex, Arc}};

use axum::{Router, routing::get, Server, response::IntoResponse, Json, extract::Path};
use http::StatusCode;
use serde::{Serialize, Deserialize};
use simplehttp::simplehttp_reqwest::SimpleHttpClientReqwest;
use surrealdb_http::surreal::{SurrealStatementReply, SurrealDbClient};
use tokio::task::spawn_blocking;

#[derive(Serialize,Deserialize,Debug)]
struct Actor {
    first_name: String,
    last_name: String,
    id: String,
}

#[tokio::main]
async fn main() {
    let surreal_client = spawn_blocking(|| {
        let mut surreal_client = SurrealDbClient::new("root", "root", "http://localhost:8000", "myns", "mydb", SimpleHttpClientReqwest::new_reqwest().unwrap());
        let mut actor: SurrealStatementReply<Actor> = surreal_client.query_single(&format!("SELECT * FROM actor WHERE id=actor:{}",41)).unwrap();
        println!("Thing: {:?}",actor);
        surreal_client
    }).await.unwrap();
    let client = Arc::new(Mutex::new(surreal_client));
    let router = Router::new()
    .with_state(client)
        .route("/actor/:actor_id", get(query_actor));
    
    Server::bind(&SocketAddr::from(([127,0,0,1],8080)))
        .serve(router.into_make_service())
        .await
        .unwrap();

}

async fn query_actor(Path(actor_id): Path<String>)->impl IntoResponse {
    let id: i32 = actor_id.parse().unwrap();
    let actor = spawn_blocking(move || {
        let mut surreal_client = SurrealDbClient::new("root", "root", "http://localhost:8000", "myns", "mydb", SimpleHttpClientReqwest::new_reqwest().unwrap());
    
        let mut actor: SurrealStatementReply<Actor> = surreal_client.query_single(&format!("SELECT * FROM actor WHERE id=actor:{}",id)).unwrap();
        actor.result.pop().unwrap()
    
    }).await.unwrap();
    (StatusCode::OK,Json(actor))
}