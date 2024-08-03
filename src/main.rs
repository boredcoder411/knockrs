use warp::Filter;
use std::collections::HashMap;
use serde::Deserialize;
use serde_json;
use std::fs;
use reqwest;

#[derive(Deserialize)]
struct ConfigData {
    port: u16,
    domain_map: HashMap<String, String>,
}

#[tokio::main]
async fn main() {
    // Read the configuration file
    let config = fs::read_to_string("config.json").expect("Unable to read file");
    let config_deser: ConfigData = serde_json::from_str(&config).expect("Unable to deserialize");
    let port = config_deser.port;
    let domain_map = config_deser.domain_map.clone();

    // Define the warp filter
    let domain_map_filter = warp::any().map(move || domain_map.clone());
    let hi = warp::path::full()
        .and(warp::header("user-agent"))
        .and(warp::header("host"))
        .and(domain_map_filter)
        .and_then(handle_request);

    // Run the warp server
    warp::serve(hi)
        .run(([127, 0, 0, 1], port))
        .await;
}

// The request handler
async fn handle_request(
    path: warp::path::FullPath,
    agent: String,
    host: String,
    domain_map: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let host = host.split(":").collect::<Vec<&str>>()[0].to_string();
    println!("Path: {}, agent: {}, host: {}", path.as_str(), agent, host);

    // Get the forwarding address from the domain map
    if let Some(forward_address) = format!("http://localhost:{}{}", domain_map.get(&host).unwrap_or(&"".to_string()), path.as_str()).parse::<String>().ok() {
        // Forward the request and return the response
        match forward_request(&forward_address).await {
            Ok(response_body) => Ok(warp::reply::Response::new(response_body.into())),
            Err(err) => {
                eprintln!("Error forwarding request: {:?}", err);
                Ok(warp::reply::Response::new("Failed to forward request".into()))
            }
        }
    } else {
        // Return an error response if no domain is found
        Ok(warp::reply::Response::new("No domain found".into()))
    }
}

// The function to forward requests
async fn forward_request(forward_address: &str) -> Result<String, reqwest::Error> {
    // Create a new reqwest client
    let client = reqwest::Client::new();

    // Make a GET request to the forwarding address
    let response = client.get(forward_address).send().await?;

    // Read the response body as a string
    let body = response.text().await?;

    // Return the response body
    Ok(body)
}
