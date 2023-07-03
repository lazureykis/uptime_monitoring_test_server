use actix_web::http::StatusCode;
use actix_web::{get, head, post, web, App, HttpResponse, HttpServer, Responder};
use std::str::FromStr;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub enum ResponseStatus {
    HttpStatus(StatusCode),
    /// Delay in seconds
    Delay(u64),
}

impl ResponseStatus {
    pub fn from_str(value: &str) -> Result<Self, String> {
        if value.starts_with("delay-") {
            let parts = value.split('-').collect::<Vec<_>>();
            if parts.len() == 2 {
                let delay = parts[1].parse::<u64>().map_err(|e| format!("{:?}", e))?;
                return Ok(ResponseStatus::Delay(delay));
            }
        }

        StatusCode::from_str(value)
            .map(|status| ResponseStatus::HttpStatus(status))
            .map_err(|e| format!("Invalid status: {e}"))
    }
}

type AppStateData = web::Data<Mutex<ResponseStatus>>;

#[head("/")]
async fn head_respond_with_status(response: AppStateData) -> impl Responder {
    let response = response.lock().unwrap().clone();
    process_request(response).await
}

#[get("/")]
async fn get_respond_with_status(response: AppStateData) -> impl Responder {
    let response = response.lock().unwrap().clone();
    process_request(response).await
}

async fn process_request(status: ResponseStatus) -> HttpResponse {
    match status {
        ResponseStatus::HttpStatus(status) => HttpResponse::build(status).finish(),
        ResponseStatus::Delay(seconds) => {
            tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
            HttpResponse::Ok().body(format!("Slept for {seconds} sec\n"))
        }
    }
}

#[post("/{new_status}")]
async fn set_status(response: AppStateData, path: web::Path<(String,)>) -> impl Responder {
    match ResponseStatus::from_str(&path.into_inner().0) {
        Ok(new_response) => {
            let mut response = response.lock().unwrap();
            *response = new_response;
            HttpResponse::Ok().body(format!("Status changed to {response:?}\n"))
        }
        Err(e) => HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let shared_state = web::Data::new(Mutex::new(ResponseStatus::HttpStatus(StatusCode::OK)));

    println!("USAGE:\ncurl -X POST http://localhost:5555/200\ncurl -X POST http://localhost:5555/500\ncurl -X POST http://localhost:5555/delay-20");

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
