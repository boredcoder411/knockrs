use warp::{Filter, Reply};
use std::{collections::HashMap, str::FromStr};
use serde::Deserialize;
use serde_json;
use std::fs;
use reqwest::Client;

#[derive(Deserialize)]
struct ConfigData {
    port: u16,
    domain_map: HashMap<String, String>,
}

#[tokio::main]
async fn main() {
    let config = fs::read_to_string("config.json").expect("Unable to read file");
    let config_deser = serde_json::from_str::<ConfigData>(&config).expect("Unable to deserialize");
    let port = config_deser.port;
    let domain_map = config_deser.domain_map;

    // Move domain_map inside the filter to avoid ownership issues
    let domain_map_filter = warp::any().map(move || domain_map.clone());

    let hi = warp::path("hello")
        .and(warp::path::param())
        .and(warp::header("user-agent"))
        .and(warp::header("host"))
        .and(domain_map_filter)
        .and_then(
            |param: String, agent: String, host: String, domain_map: HashMap<String, String>| async move {
                println!("param: {}, agent: {}, host: {}", param, agent, host);
                let host = host.split(":").collect::<Vec<&str>>()[0].to_string();
                if let Some(target) = domain_map.get(&host) {
                    // forward request
                    let client = Client::new();
                    let url = format!("http://{}/{}", target, param);
                    let request = client.get(&url).header("User-Agent", agent.clone()).build().unwrap();
                    match client.execute(request).await {
                        Ok(response) => {
                            let status = response.status();
                            let status_str = status.as_str();
                            let body = response.text().await.unwrap_or_else(|_| "Error".into());
                            Ok::<_, warp::Rejection>(warp::reply::with_status(body, warp::http::StatusCode::from_str(status_str).unwrap()).into_response())
                        },
                        Err(_) => Ok(warp::reply::with_status::<&'static str>("Error".into(), warp::http::StatusCode::INTERNAL_SERVER_ERROR).into_response()),
                    }
                } else {
                    Ok(warp::reply::with_status(warp::reply::html("Not Found"), warp::http::StatusCode::NOT_FOUND).into_response())
                }
            }
        );

    warp::serve(hi)
        .run(([127, 0, 0, 1], port))
        .await;
}
