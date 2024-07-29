use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;

// Custom error type
#[derive(Debug)]
enum ApiError {
    DatabaseError(sqlx::Error),
    ExternalApiError(reqwest::Error),
    NotFound,
}

// Implement IntoResponse for ApiError
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ),
            ApiError::ExternalApiError(e) => (
                StatusCode::BAD_GATEWAY,
                format!("External API error: {}", e),
            ),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
        };

        (
            status,
            Json(ErrorResponse {
                error: error_message,
            }),
        )
            .into_response()
    }
}

// Error response struct
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/weather", get(weather))
        .with_state(Arc::new(pool));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello_world() -> &'static str {
    "Hello, World!"
}

async fn weather(
    Query(params): Query<WeatherQuery>,
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<WeatherResponse>, ApiError> {
    let lat_long = get_lat_long(&pool, &params.city).await?;
    let weather = fetch_weather(lat_long).await?;
    Ok(Json(weather))
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

#[derive(Deserialize, Serialize, Debug)]
struct WeatherResponse {
    latitude: f64,
    longitude: f64,
    timezone: String,
    hourly: Hourly,
}

#[derive(Deserialize, Serialize, Debug)]
struct Hourly {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
}

async fn get_lat_long(pool: &PgPool, city: &str) -> Result<LatLong, ApiError> {
    let result =
        sqlx::query_as::<_, LatLong>("SELECT latitude, longitude FROM cities WHERE name = $1")
            .bind(city)
            .fetch_optional(pool)
            .await
            .map_err(ApiError::DatabaseError)?;

    if let Some(lat_long) = result {
        return Ok(lat_long);
    }

    let lat_long = fetch_lat_long(city).await?;

    sqlx::query("INSERT INTO cities (name, latitude, longitude) VALUES ($1, $2, $3)")
        .bind(city)
        .bind(lat_long.latitude)
        .bind(lat_long.longitude)
        .execute(pool)
        .await
        .map_err(ApiError::DatabaseError)?;

    Ok(lat_long)
}

async fn fetch_lat_long(city: &str) -> Result<LatLong, ApiError> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );
    let response = reqwest::get(&url)
        .await
        .map_err(ApiError::ExternalApiError)?
        .json::<GeoResponse>()
        .await
        .map_err(ApiError::ExternalApiError)?;

    response.results.get(0).cloned().ok_or(ApiError::NotFound)
}

async fn fetch_weather(lat_long: LatLong) -> Result<WeatherResponse, ApiError> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
        lat_long.latitude, lat_long.longitude
    );
    let response = reqwest::get(&url)
        .await
        .map_err(ApiError::ExternalApiError)?
        .json::<WeatherResponse>()
        .await
        .map_err(ApiError::ExternalApiError)?;
    Ok(response)
}
