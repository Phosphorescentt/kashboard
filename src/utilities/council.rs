use reqwest::{
    self,
    cookie::Jar,
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct BinData {
    #[serde(alias = "d")]
    data: Vec<VeoliaService>,
}

#[derive(Serialize, Deserialize)]
struct VeoliaService {
    __type: String,
    #[serde(alias = "ServiceHeaders")]
    pub service_headers: Vec<ServiceHeader>,
    #[serde(alias = "ServiceName")]
    service_name: String,
}

#[derive(Serialize, Deserialize)]
struct ServiceHeader {
    #[serde(alias = "TaskType")]
    task_type: String,
    #[serde(alias = "Last")]
    last: String,
    #[serde(alias = "Next")]
    next: String,
    #[serde(alias = "ScheduleDescription")]
    schedule_description: String,
}

pub async fn get_bin_data() -> BinData {
    let jar = Arc::new(Jar::default());

    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(jar.clone())
        .build()
        .unwrap();

    // Do this one just to get the cookies
    let _r1 = client
        .get("https://gis.stalbans.gov.uk/NoticeBoard9/NoticeBoard.aspx")
        .send()
        .await;

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );

    let resp = client.post("https://gis.stalbans.gov.uk/NoticeBoard9/VeoliaProxy.NoticeBoard.asmx/GetServicesByUprnAndNoticeBoard")
        .body(format!("{{\"uprn\":{},\"noticeBoard\":\"default\"}}", env::var("BINS_UPRN").unwrap()))
        .headers(headers)
        .send()
        .await
        .unwrap();

    let data: BinData = serde_json::from_str(resp.text().await.unwrap().as_str()).unwrap();
    return data;
}
