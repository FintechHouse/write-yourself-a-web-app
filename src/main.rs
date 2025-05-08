use axum::{
    routing::get,
    Router,
    extract::{Query,State},
};

use serde::{Deserialize, Serialize};

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

// Write your code here.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db : sled::Db = sled::open("my_db").unwrap();

    let app = Router::new()
            .route("/", get(root))
            .route("/weather", get(weather))
            .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn weather(Query(params): Query<WeatherQuery>, State(db) : State<sled::Db>) -> Result<String, String> {
    let lat_long = get_latlong(db, &params.city).await.map_err(|e| e.to_string())?;
    let weather = fetch_weather(lat_long).await.map_err(|e| e.to_string())?;
    Ok(format!("Weather for {}: {:?}", params.city, weather))
}

async fn get_latlong(db: sled::Db, city: &str) -> Result<LatLong, Box<dyn std::error::Error>> {
    if let Some(latl) = db.get(city.as_bytes()).unwrap() {
        println!("City {city} found in the cache");
        let lstr : &str = std::str::from_utf8(&latl).unwrap();
        let r : LatLong = serde_json::from_str(lstr).unwrap();
        Ok(r)
    } else {
        println!("City {city} NOT found in the cache. Going web!!!");
        let url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
            city
        );
        let response = reqwest::get(&url).await?.json::<GeoResponse>().await?;
        if let Some(res) = response.results.first() {
            let val = serde_json::to_string(&res).unwrap();
            let _ = db.insert(city, val.as_bytes()).unwrap();
            Ok(res.clone())
        } else {
            Err("No results found".into())
        }
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
