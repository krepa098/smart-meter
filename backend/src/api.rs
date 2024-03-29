use std::{
    io,
    sync::{Arc, Mutex},
};

use actix_cors::Cors;
use actix_web::{
    get,
    http::header,
    middleware, put,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};

use crate::db::{models, Db};
use common::req::MeasurementInfo;

#[get("/")]
async fn hello(_db: web::Data<Arc<Mutex<Db>>>) -> impl Responder {
    HttpResponse::Ok().body("backend")
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

#[derive(serde::Deserialize, Debug)]
struct MeasurementsInfoQuery {
    device_id: u32,
}

#[get("/api/measurements/info")]
async fn api_measurements_info(
    query: web::Query<MeasurementsInfoQuery>,
    db: web::Data<Arc<Mutex<Db>>>,
) -> io::Result<impl Responder> {
    dbg!(&query);
    if let Ok(mut db) = db.lock() {
        if let Ok(res) = db.measurement_info(query.device_id) {
            return Ok(web::Json(MeasurementInfo {
                device_id: query.device_id as i32,
                from_timestamp: res.0,
                to_timestamp: res.1,
                count: res.2,
            }));
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

#[derive(serde::Deserialize, Debug)]
struct SetDeviceNameParams {
    device_id: u32,
    name: String,
}

#[put("/api/device_name")]
async fn api_set_device_name(
    query: web::Query<SetDeviceNameParams>,
    db: web::Data<Arc<Mutex<Db>>>,
) -> io::Result<impl Responder> {
    dbg!(&query);
    if let Ok(mut db) = db.lock() {
        if db
            .update_device_name(&models::DeviceName {
                device_id: query.device_id as i32,
                name: query.name.clone(),
            })
            .is_ok()
        {
            return Ok(HttpResponse::Ok());
        }
    }
    Err(io::Error::new(io::ErrorKind::BrokenPipe, "".to_string()))
}

#[derive(serde::Deserialize, Debug)]
struct DeviceNameParams {
    device_id: u32,
}

#[get("/api/device_name")]
async fn api_device_name(
    query: web::Query<DeviceNameParams>,
    db: web::Data<Arc<Mutex<Db>>>,
) -> io::Result<impl Responder> {
    dbg!(&query);
    if let Ok(mut db) = db.lock() {
        if let Ok(res) = db.device_name(query.device_id) {
            return Ok(web::Json(res.name));
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "".to_string()))
}

pub async fn new_http_server(db: Arc<Mutex<Db>>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .wrap(middleware::Compress::default())
            .service(hello)
            .service(api_measurements_by_date)
            .service(api_measurements_all)
            .service(api_measurements_info)
            .service(api_known_devices)
            .service(api_set_device_name)
            .service(api_device_name)
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "PUT"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
