use std::{
    io,
    sync::{Arc, Mutex},
};

use actix_cors::Cors;
use actix_web::{
    get,
    http::header,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};

use crate::db::Db;

#[get("/")]
async fn hello(_db: web::Data<Arc<Mutex<Db>>>) -> impl Responder {
    return HttpResponse::Ok().body("backend");
}

#[get("/test")]
async fn test(_db: web::Data<Arc<Mutex<Db>>>) -> impl Responder {
    return HttpResponse::Ok().body("{ \"backend\": \"hello\" }");
}

#[derive(serde::Deserialize, Debug)]
struct MeasurementsQueryByDate {
    device_id: u32,
    from_date: Option<u64>,
    to_date: Option<u64>,
    measurement_types: u32,
    limit: u32,
}

#[get("/api/measurements/by_date")]
async fn api_measurements_by_date(
    query: web::Query<MeasurementsQueryByDate>,
    db: web::Data<Arc<Mutex<Db>>>,
) -> io::Result<impl Responder> {
    dbg!(&query);
    if let Ok(mut db) = db.lock() {
        if let Ok(res) = db.measurements_by_date(
            query.device_id,
            query.from_date,
            query.to_date,
            query.measurement_types,
            query.limit,
        ) {
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

#[get("/api/devices")]
async fn api_known_devices(db: web::Data<Arc<Mutex<Db>>>) -> io::Result<impl Responder> {
    if let Ok(mut db) = db.lock() {
        if let Ok(res) = db.devices() {
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
            .service(test)
            .service(api_measurements_by_date)
            .service(api_measurements_all)
            .service(api_known_devices)
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:8080")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
