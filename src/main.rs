use actix_web::{App, HttpResponse, HttpServer, Responder, get};

#[get("/")]
fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

fn main() {
    HttpServer::new(|| {
        App::new()
            .service(index)
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
