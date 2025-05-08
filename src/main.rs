use axum::extract::State;
use axum::{
    routing::get,
    Router,
    extract::Query,
};

use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Deserialize)]
struct WeatherQuery {
    city: String,
}

#[derive(Deserialize, Debug)]
struct GeoResponse {
    results: Vec<LatLong>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct LatLong {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    latitude: f64,
    longitude: f64,
    timezone: String,
    hourly: Hourly,
}

#[derive(Deserialize, Debug)]
struct Hourly {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
}

type MyCache = HashMap<String, LatLong>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let mut hmap : MyCache = HashMap::new();
    let lcache = Arc::new(Mutex::new(hmap));

    let app = Router::new()
            .route("/", get(root))
            .route("/weather", get(weather))
            .with_state(lcache);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn weather(params: Query<WeatherQuery>, State(lcache): State<Arc<Mutex<MyCache>>>) -> Result<String, String> {
    let lat_long = get_latlong(lcache.clone(), &params.city).await.map_err(|e| e.to_string())?;
    let weather = fetch_weather(lat_long).await.map_err(|e| e.to_string())?;
    Ok(format!("Weather for {}: {:?}", params.city, weather))
}

async fn get_latlong(lcache: Arc<Mutex<MyCache>>, city: &str) -> Result<LatLong, Box<dyn std::error::Error>> {
    {
        let lock = lcache.lock().unwrap();
        match lock.get(city) {
            Some(v) => return Ok(v.clone()),
            _ => (),
        }
    };
    // city not found in the cache: let's get it from the web

    println!("City {city} not found in cache");
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );
    let response = reqwest::get(&url).await?.json::<GeoResponse>().await?;
    match response.results.first() {
        Some(v) => {
            let mut lock = lcache.lock().unwrap();
            lock.insert(city.to_string(), v.clone());
            Ok(v.clone())
        },
        None => Err("No results found".into()),
    }
}

async fn fetch_weather(lat_long: LatLong) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
        lat_long.latitude, lat_long.longitude
    );
    let response = reqwest::get(&url).await?.json::<WeatherResponse>().await?;
    Ok(response)
}
