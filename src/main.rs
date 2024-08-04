use warp::Filter;
use std::collections::HashMap;
use serde::Deserialize;
use serde_json;
use std::fs;
use reqwest;

#[derive(Deserialize, Debug)]
struct ConfigData {
    port: u16,
    domain_map: HashMap<String, String>,
}

#[tokio::main]
async fn main() {
    // Read the configuration file
    println!("Reading configuration file...");
    let config = fs::read_to_string("config.json").expect("Unable to read file");
    println!("Config file content: {}", config);
    let config_deser: ConfigData = serde_json::from_str(&config).expect("Unable to deserialize");
    println!("Deserialized config: {:?}", config_deser);
    let port = config_deser.port;
    let domain_map = config_deser.domain_map.clone();
    println!("Port: {}, Domain map: {:?}", port, domain_map);

    // Define the warp filter
    println!("Defining warp filter...");
    let domain_map_filter = warp::any().map(move || domain_map.clone());
    let hi = warp::path::full()
        .and(warp::header("user-agent"))
        .and(warp::header("host"))
        .and(domain_map_filter)
        .and_then(handle_request);

    // Run the warp server
    println!("Starting warp server on port {}...", port);
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
    println!("Received request - Path: {}, Agent: {}, Host: {}", path.as_str(), agent, host);
    let host = host.split(":").collect::<Vec<&str>>()[0].to_string();
    println!("Processed host: {}", host);

    // Get the forwarding address from the domain map
    if let Some(forward_address) = domain_map.get(&host) {
        let full_forward_address = format!("http://localhost:{}{}", forward_address, path.as_str());
        println!("Forwarding address: {}", full_forward_address);
        
        // Forward the request and return the response
        match forward_request(&full_forward_address).await {
            Ok(response_body) => {
                println!("Forward request successful, response body: {}", response_body);
                Ok(warp::reply::Response::new(response_body.into()))
            },
            Err(err) => {
                eprintln!("Error forwarding request: {:?}", err);
                Ok(warp::reply::Response::new("Failed to forward request".into()))
            }
        }
    } else {
        // Return an error response if no domain is found
        println!("No domain found for host: {}", host);
        Ok(warp::reply::Response::new("No domain found".into()))
    }
}

// The function to forward requests
async fn forward_request(forward_address: &str) -> Result<String, reqwest::Error> {
    println!("Forwarding request to address: {}", forward_address);

    // Create a new reqwest client
    let client = reqwest::Client::new();

    // Make a GET request to the forwarding address
    let response = client.get(forward_address).send().await?;
    println!("Received response with status: {}", response.status());

    // Read the response body as a string
    let body = response.text().await?;
    println!("Response body: {}", body);

    // Return the response body
    Ok(body)
}

