use std::sync::{Arc, Mutex};

use actix_web::{
    get, post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};

use crate::db::Db;

#[get("/")]
async fn hello(db: web::Data<Arc<Mutex<Db>>>) -> impl Responder {
    if let Ok(mut db) = db.lock() {
        let res = db.all_measurements();
        let mut json = String::new();

        for r in &res {
            json += &serde_json::to_string_pretty(&r).unwrap();
        }

        return HttpResponse::Ok().body(json);
    }
    HttpResponse::Ok().body("json")
}

pub async fn new_http_server(db: Arc<Mutex<Db>>) -> std::io::Result<()> {
    HttpServer::new(move || App::new().app_data(Data::new(db.clone())).service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
