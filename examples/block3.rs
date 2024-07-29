use axum::{
    extract::{Query, State},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    // Run migrations
    sqlx::migrate!("examples/migrations/block3")
        .run(&pool)
        .await?;

    // Initialize the router
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/weather", get(weather))
        .with_state(Arc::new(pool));

    // Set up the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Server running on http://{}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Handler function for the root path
async fn hello_world() -> &'static str {
    "Hello, World!"
}

// Handler function for the weather path
async fn weather(
    Query(params): Query<WeatherQuery>,
    State(pool): State<Arc<PgPool>>,
) -> Result<String, String> {
    let lat_long = get_lat_long(&pool, &params.city)
        .await
        .map_err(|e| e.to_string())?;
    let weather = fetch_weather(lat_long).await.map_err(|e| e.to_string())?;
    Ok(format!("Weather for {}: {:?}", params.city, weather))
}

#[derive(Deserialize)]
struct WeatherQuery {
    city: String,
}

#[derive(Deserialize, Debug)]
struct GeoResponse {
    results: Vec<LatLong>,
}

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
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

async fn get_lat_long(pool: &PgPool, city: &str) -> Result<LatLong, Box<dyn std::error::Error>> {
    // Try to get from database first
    let result =
        sqlx::query_as::<_, LatLong>("SELECT latitude, longitude FROM cities WHERE name = $1")
            .bind(city)
            .fetch_optional(pool)
            .await?;

    if let Some(lat_long) = result {
        return Ok(lat_long);
    }

    // If not in database, fetch from API
    let lat_long = fetch_lat_long(city).await?;

    // Insert into database
    sqlx::query("INSERT INTO cities (name, latitude, longitude) VALUES ($1, $2, $3)")
        .bind(city)
        .bind(lat_long.latitude)
        .bind(lat_long.longitude)
        .execute(pool)
        .await?;

    Ok(lat_long)
}

async fn fetch_lat_long(city: &str) -> Result<LatLong, Box<dyn std::error::Error>> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );
    let response = reqwest::get(&url).await?.json::<GeoResponse>().await?;
    response
        .results
        .get(0)
        .cloned()
        .ok_or_else(|| "No results found".into())
}

async fn fetch_weather(lat_long: LatLong) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
        lat_long.latitude, lat_long.longitude
    );
    let response = reqwest::get(&url).await?.json::<WeatherResponse>().await?;
    Ok(response)
}
