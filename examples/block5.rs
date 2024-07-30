use axum::{
    async_trait,
    extract::{FromRequestParts, Query, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

struct User;

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|header| header.to_str().ok());

        if let Some(auth_header) = auth_header {
            if auth_header.starts_with("Basic ") {
                let credentials = auth_header.trim_start_matches("Basic ");
                let decoded = general_purpose::STANDARD
                    .decode(credentials)
                    .map_err(|_| ApiError::Unauthorized)?;
                let decoded_str = String::from_utf8(decoded).map_err(|_| ApiError::Unauthorized)?;
                let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();

                if parts.len() == 2 && parts[0] == "forecast" && parts[1] == "forecast" {
                    return Ok(User);
                }
            }
        }

        Err(ApiError::Unauthorized)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/weather", get(weather))
        .route("/stats", get(stats))
        .with_state(Arc::new(pool));

    println!("Server running on http://0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    Ok(axum::serve(listener, app).await?)
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

async fn stats(_: User, State(pool): State<Arc<PgPool>>) -> Result<Json<StatsResponse>, ApiError> {
    let cities = get_last_cities(&pool).await?;
    Ok(Json(StatsResponse { cities }))
}

#[derive(Deserialize)]
struct WeatherQuery {
    city: String,
}

#[derive(Serialize)]
struct StatsResponse {
    cities: Vec<String>,
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

async fn get_last_cities(pool: &PgPool) -> Result<Vec<String>, ApiError> {
    let cities = sqlx::query_scalar("SELECT name FROM cities ORDER BY id DESC LIMIT 10")
        .fetch_all(pool)
        .await
        .map_err(ApiError::DatabaseError)?;
    Ok(cities)
}

#[derive(Debug)]
enum ApiError {
    DatabaseError(sqlx::Error),
    ExternalApiError(reqwest::Error),
    NotFound,
    Unauthorized,
}

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
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
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

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}
