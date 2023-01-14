use actix_web::http::StatusCode;
use actix_web::{get, head, post, web, App, HttpResponse, HttpServer, Responder};
use std::str::FromStr;
use std::sync::Mutex;

struct AppState {
    pub status: Mutex<StatusCode>,
}

#[head("/")]
async fn head_respond_with_status(data: web::Data<AppState>) -> impl Responder {
    let status = data.status.lock().unwrap();
    HttpResponse::build(*status).body(format!("{status}"))
}

#[get("/")]
async fn get_respond_with_status(data: web::Data<AppState>) -> impl Responder {
    let status = data.status.lock().unwrap();
    HttpResponse::build(*status).body(format!("{status}"))
}

#[post("/{new_status}")]
async fn set_status(data: web::Data<AppState>, path: web::Path<(String,)>) -> impl Responder {
    match StatusCode::from_str(&path.into_inner().0) {
        Ok(new_status) => {
            let mut status = data.status.lock().unwrap();
            *status = new_status;
            HttpResponse::Ok().finish()
        }
        Err(_) => HttpResponse::UnprocessableEntity().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let shared_state = web::Data::new(AppState {
        status: Mutex::new(StatusCode::OK),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(shared_state.clone())
            .service(get_respond_with_status)
            .service(head_respond_with_status)
            .service(set_status)
    })
    .bind(("0.0.0.0", 5555))?
    .run()
    .await
}
