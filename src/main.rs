mod server;
mod words;

use uuid::Uuid;
use actix_web::{get, web::{self, Json}, App, HttpServer, Responder, HttpResponse, http::{StatusCode, header::{self, HeaderValue}}, cookie::Cookie, HttpRequest};
use std::{io, sync::Mutex};
use lazy_static::lazy_static;

lazy_static! {
    static ref SERVER: Mutex<server::Server> = Mutex::new(server::Server::new());
    static ref SUBMIT_ERROR: fn() -> Json<SubmitResponse> = || Json(
        SubmitResponse {
            error: true,
            win: "".into()
        }
    );
}

const HOST: &'static str = "127.0.0.1";
const PORT: u16 = 8080;

#[derive(serde_derive::Serialize)]
struct SubmitResponse {
    win: String,
    error: bool
}

#[get("/submit/{guess}")]
async fn submit_guess(request: HttpRequest, params: web::Path<String>) -> impl Responder {
    let mut server_ref = SERVER.lock().unwrap();
    let key = params.into_inner();
    let uuid_wrap = server::get_player_uuid(&request);

    if uuid_wrap.is_none() {
        return SUBMIT_ERROR();
    }

    let uuid = uuid_wrap.unwrap();

    if !server_ref.has_lobby(&uuid) {
        return SUBMIT_ERROR();
    }
    server_ref.player_submit_move(&uuid, key);
    let lobby = server_ref.get_player(&uuid).lobby.as_ref().unwrap().clone();
    if let Some(player) = server_ref.any_winners(&lobby) {
        server_ref.end_game(&lobby);
        return Json(
            SubmitResponse {
                win: server_ref.get_player(&player).name.clone(),
                error: false
            }
        );
    }
    Json( SubmitResponse { win: "".into(), error: false } )
}

#[get("/join/{lobby_id}")]
async fn join_lobby_id(request: HttpRequest, params: web::Path<String>) -> impl Responder {
    let mut server_ref = SERVER.lock().unwrap();
    let key = params.into_inner();
    let uuid_wrap = server::get_player_uuid(&request);

    if uuid_wrap.is_none() {
        return format!("{}", false);
    }

    let uuid = uuid_wrap.unwrap();

    if !server_ref.does_lobby_exist(&key) || !server_ref.is_player_valid(&uuid)
    {
        return format!("{}", false);
    }

    let player_ref = server_ref.get_player_mut(&uuid);
    player_ref.lobby = Some(key);

    format!("{}", true)
}

#[get("/init")]
async fn init(request: HttpRequest) -> impl Responder 
{
    let mut resp = HttpResponse::new(StatusCode::OK);
    let mut server_ref = SERVER.lock().unwrap();
    if let Some(uuid) = server::get_player_uuid(&request) {
        if server_ref.is_player_valid(&uuid) {
            return resp;
        }
    }
    let player_id = Uuid::new_v4();
    server_ref.init_player(&player_id);
    let header_val = HeaderValue::from_str(
        &Cookie::new("userid", &player_id.to_string()).to_string()
    ).expect("Fail to create header_value");
    resp.headers_mut().insert(header::SET_COOKIE, header_val);
    resp
}

#[get("/square/{num}")]
async fn index(params: web::Path<i32>) -> impl Responder {
    let num = params.into_inner();
    format!("{}", num*num)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new( || App::new()
                     .service(index)
                     .service(join_lobby_id)
                     .service(init)
                     )
        .bind((HOST, PORT))?
        .run()
        .await
}
