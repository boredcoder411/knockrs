use warp::Filter;
use std::collections::HashMap;
use serde::Deserialize;
use serde_json;
use std::fs;

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

    let hi = warp::path("hello")
        .and(warp::path::param())
        .and(warp::header("user-agent"))
        .and(warp::header("host"))
        .map(move |param: String, agent: String, host: String| {
            println!("param: {}, agent: {}, host: {}", param, agent, host);
            let host = host.split(":").collect::<Vec<&str>>()[0].to_string();
            domain_map.get(&host).unwrap_or(&"No domain found".to_string()).to_string()
        });
    

    warp::serve(hi)
        .run(([127, 0, 0, 1], port.into()))
        .await;
}