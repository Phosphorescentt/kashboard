use crate::TEMPLATES;
use actix_web::{get, HttpResponse, Responder};
use tera::Context;

use crate::utilities::council;
use crate::utilities::octopus;

#[get("")]
async fn bins() -> impl Responder {
    let mut context = Context::new();

    let bin_data: council::BinData = council::get_bin_data().await;
    context.insert("bin_data", &bin_data);

    let octopus_data: octopus::OctopusData = octopus::get_last_24h_electricity_consumption().await;
    context.insert("octopus_data", &octopus_data);

    let content = TEMPLATES.render("utilities.html", &context).unwrap();
    return HttpResponse::Ok().body(content);
}
