use std::{
    io,
    sync::{Arc, Mutex},
};

use actix_web::{
    get,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};

use crate::db::Db;

#[get("/")]
async fn hello(_db: web::Data<Arc<Mutex<Db>>>) -> impl Responder {
    return HttpResponse::Ok().body("backend");
}

#[derive(serde::Deserialize, Debug)]
struct MeasurementsQueryByDate {
    device_id: u32,
    from_date: u64,
    to_date: u64,
}

#[get("/api/measurements/by_date")]
async fn api_measurements_by_date(
    query: web::Query<MeasurementsQueryByDate>,
    db: web::Data<Arc<Mutex<Db>>>,
) -> io::Result<impl Responder> {
    if let Ok(mut db) = db.lock() {
        if let Ok(res) = db.measurements_byte_date(query.device_id, query.from_date, query.to_date)
        {
            return Ok(web::Json(res));
        }
    }
    Err(io::Error::new(io::ErrorKind::BrokenPipe, "".to_string()))
}

#[get("/api/measurements/all")]
async fn api_measurements_all(db: web::Data<Arc<Mutex<Db>>>) -> io::Result<impl Responder> {
    if let Ok(mut db) = db.lock() {
        if let Ok(res) = db.all_measurements() {
            return Ok(web::Json(res));
        }
    }
    Err(io::Error::new(io::ErrorKind::BrokenPipe, "".to_string()))
}

#[get("/api/known_devices")]
async fn api_known_devices(db: web::Data<Arc<Mutex<Db>>>) -> io::Result<impl Responder> {
    if let Ok(mut db) = db.lock() {
        if let Ok(res) = db.known_devices() {
            return Ok(web::Json(res));
        }
    }
    Err(io::Error::new(io::ErrorKind::BrokenPipe, "".to_string()))
}

pub async fn new_http_server(db: Arc<Mutex<Db>>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .service(hello)
            .service(api_measurements_by_date)
            .service(api_measurements_all)
            .service(api_known_devices)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
