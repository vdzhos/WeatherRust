use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use ic_cdk_macros::{query, update};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use candid::{CandidType, Principal};

//-----structs-----

#[derive(Serialize, Deserialize, CandidType, Debug)]
struct WeatherRequest {
    token: String,
    requester_name: String,
    location: String,
    date: String,
}

#[derive(Serialize, Deserialize, CandidType, Debug)]
struct AverageWeatherOnRangeRequest {
    token: String,
    requester_name: String,
    location: String,
    start_date: String,
    end_date: String,
}

#[derive(Serialize, Deserialize, CandidType, Debug)]
struct CompareWeatherRequest {
    token: String,
    requester_name: String,
    location1: String,
    location2: String,
    date: String,
}

#[derive(Serialize, Deserialize, CandidType, Debug)]
struct WeatherStationsRequest {
    token: String,
    requester_name: String,
    location: String,
    date: String
}

#[derive(Serialize, Deserialize, CandidType, Debug)]
struct WeatherResponse {
    temp_c: f64,
    wind_kph: f64,
    pressure_mb: f64,
    humidity: f64,
}

#[derive(Serialize, Deserialize, CandidType, Debug)]
struct AverageWeatherResponse {
    avg_temp_c: f64,
    avg_wind_kph: f64,
    avg_pressure_mb: f64,
    avg_humidity: f64,
}

#[derive(Serialize, Deserialize, CandidType, Debug)]
struct LocationWeatherResponse {
    location_name: String,
    weather: WeatherResponse,
}

#[derive(Serialize, Deserialize, CandidType, Debug)]
struct CompareWeatherResponse {
    location1: LocationWeatherResponse,
    location2: LocationWeatherResponse,
}

#[derive(Serialize, Deserialize, CandidType, Debug)]
struct StationInfo {
    name: String,
    quality: u32,
}

#[derive(Serialize, Deserialize, CandidType, Debug)]
struct WeatherStationsResponse {
    stations: Vec<StationInfo>,
}

#[derive(Debug, Serialize)]
pub struct WindInfoResponse {
    wind_direction: String,
    wind_power: String,
}

#[derive(Debug, Serialize)]
pub struct UvRecommendation {
    sunglasses_recommendation: String,
    uvindex: f64,
}

#[derive(Debug, Serialize)]
pub struct PrecipitationInfoResponse {
    precipitation_mm: f64,
    precipitation_type: String,
}

#[derive(Debug, Serialize)]
pub struct FeelsLikeTemperatureResponse {
    actual_temp_c: f64,
    feels_like_c: f64,
}

#[derive(Debug, Serialize)]
pub struct SunTimesResponse {
    sunrise: String,
    sunset: String,
}

#[derive(Serialize, Deserialize)]
struct Context {
    bucket_start_time_index: usize,
    closing_price_index: usize,
}

//-----end structs-----

const API_TOKEN: &str = "3e2f4d6a5b8c9e1f1234abcd5678ef90";
const WEATHER_API_KEY: &str = "4WWQEDQX2PLN5W97DGKEDXAS3";
const WEATHER_API_HOST: &str = "weather-gateway.weather-rust-gateway.workers.dev";

fn build_http_request(url: String) -> CanisterHttpRequestArgument {
    let context = Context {
        bucket_start_time_index: 0,
        closing_price_index: 4,
    };

    CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        headers: vec![HttpHeader {
            name: "User-Agent".to_string(),
            value: "Rust Weather Canister".to_string(),
        }],
        body: None,
        max_response_bytes: None,
        transform: Some(TransformContext::new(transform, serde_json::to_vec(&context).unwrap())),
    }
}

#[query]
fn transform(raw: TransformArgs) -> HttpResponse {

    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];


    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers,
        ..Default::default()
    };

    if res.status == 200 {

        res.body = raw.response.body;
    } else {
        ic_cdk::api::print(format!("Received an error from coinbase: err = {:?}", raw));
    }
    res
}

#[update]
async fn weather_endpoint(request: WeatherRequest) -> String {
    if request.token != API_TOKEN {
        return "Invalid API token.".to_string();
    }

    match get_weather(&request.location, &request.date).await {
        Ok(weather) => serde_json::to_string(&weather).unwrap(),
        Err(e) => format!("Error fetching weather data: {}", e),
    }
}

async fn get_weather(location: &str, date: &str) -> Result<WeatherResponse, String> {
    let url = format!(
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}?unitGroup=metric&key={}&include=days",
        WEATHER_API_HOST, location, date, WEATHER_API_KEY
    );

    let request = build_http_request(url);

    match http_request(request).await {
        Ok((response,)) if response.status == 200 => {
            let str_body = String::from_utf8(response.body)
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let all_info: Value = serde_json::from_str(&str_body)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let result = WeatherResponse {
                temp_c: all_info["days"][0]["temp"].as_f64().unwrap_or(0.0),
                wind_kph: all_info["days"][0]["windspeed"].as_f64().unwrap_or(0.0),
                pressure_mb: all_info["days"][0]["pressure"].as_f64().unwrap_or(0.0),
                humidity: all_info["days"][0]["humidity"].as_f64().unwrap_or(0.0),
            };

            Ok(result)
        }
        Ok((response,)) => Err(format!("Non-200 response: {}", response.status)),
        Err((code, message)) => Err(format!("Request failed: {:?}, {}", code, message)),
    }
}

#[update]
async fn average_weather_on_range_endpoint(request: AverageWeatherOnRangeRequest) -> String {
    if request.token != API_TOKEN {
        return "Invalid API token.".to_string();
    }

    match get_average_weather_on_range(&request.location, &request.start_date, &request.end_date).await {
        Ok(weather) => serde_json::to_string(&weather).unwrap(),
        Err(e) => format!("Error fetching weather data: {}", e),
    }
}

async fn get_average_weather_on_range(location: &str, start_date: &str, end_date: &str) -> Result<AverageWeatherResponse, String> {
    let url = format!(
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}/{}?unitGroup=metric&key={}&include=days",
        WEATHER_API_HOST, location, start_date, end_date, WEATHER_API_KEY
    );

    let request = build_http_request(url);

    match http_request(request).await {
        Ok((response,)) if response.status == 200 => {
            let str_body = String::from_utf8(response.body)
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let all_info: Value = serde_json::from_str(&str_body)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let days = all_info["days"].as_array().ok_or("Invalid data format: 'days' not found")?;

            let mut total_temp = 0.0;
            let mut total_wind = 0.0;
            let mut total_pressure = 0.0;
            let mut total_humidity = 0.0;
            let mut count = 0.0;

            for day in days {
                total_temp += day["temp"].as_f64().unwrap_or(0.0);
                total_wind += day["windspeed"].as_f64().unwrap_or(0.0);
                total_pressure += day["pressure"].as_f64().unwrap_or(0.0);
                total_humidity += day["humidity"].as_f64().unwrap_or(0.0);
                count += 1.0;
            }

            if count == 0.0 {
                return Err("No data found for the given range.".to_string());
            }

            let result = AverageWeatherResponse {
                avg_temp_c: total_temp / count,
                avg_wind_kph: total_wind / count,
                avg_pressure_mb: total_pressure / count,
                avg_humidity: total_humidity / count,
            };

            Ok(result)
        }
        Ok((response,)) => Err(format!("Non-200 response: {}", response.status)),
        Err((code, message)) => Err(format!("Request failed: {:?}, {}", code, message)),
    }
}

#[update]
async fn compare_weather_endpoint(request: CompareWeatherRequest) -> String {
    if request.token != API_TOKEN {
        return "Invalid API token.".to_string();
    }

    let weather1 = get_weather(&request.location1, &request.date).await;
    let weather2 = get_weather(&request.location2, &request.date).await;

    match (weather1, weather2) {
        (Ok(w1), Ok(w2)) => serde_json::to_string(&CompareWeatherResponse {
            location1: LocationWeatherResponse {
                location_name: request.location1.clone(),
                weather: w1,
            },
            location2: LocationWeatherResponse {
                location_name: request.location2.clone(),
                weather: w2,
            },
        }).unwrap(),
        (Err(e1), _) => format!("Error fetching weather data for location1 ({}): {}", request.location1, e1),
        (_, Err(e2)) => format!("Error fetching weather data for location2 ({}): {}", request.location2, e2),
    }
}

#[update]
async fn get_weather_stations_endpoint(request: WeatherStationsRequest) -> String {
    if request.token != API_TOKEN {
        return "Invalid API token.".to_string();
    }

    match get_weather_stations(&request.location, &request.date).await {
        Ok(response) => serde_json::to_string(&response).unwrap(),
        Err(e) => format!("Error fetching stations: {}", e),
    }
}

async fn get_weather_stations(location: &str, date: &str) -> Result<WeatherStationsResponse, String> {
    let url = format!(
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}?unitGroup=metric&key={}&include=days",
        WEATHER_API_HOST, location, date, WEATHER_API_KEY
    );

    let request = build_http_request(url);

    match http_request(request).await {
        Ok((response,)) if response.status == 200 => {
            let str_body = String::from_utf8(response.body)
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let all_info: Value = serde_json::from_str(&str_body)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let stations_map = all_info["stations"].as_object()
                .ok_or("Invalid data format: 'stations' not found")?;

            let mut stations_list = Vec::new();

            for (station_id, station_data) in stations_map {
                if let Some(name) = station_data["name"].as_str() {
                    let quality = station_data["quality"].as_u64().unwrap_or(0) as u32;
                    stations_list.push(StationInfo {
                        name: name.to_string(),
                        quality,
                    });
                }
            }

            Ok(WeatherStationsResponse { stations: stations_list })
        }
        Ok((response,)) => Err(format!("Non-200 response: {}", response.status)),
        Err((code, message)) => Err(format!("Request failed: {:?}, {}", code, message)),
    }
}

//----------wind info------------

#[update]
async fn get_wind_info_endpoint(request: WeatherRequest) -> String {
    if request.token != API_TOKEN {
        return "Invalid API token.".to_string();
    }

    match get_wind_info(&request.location, &request.date).await {
        Ok(response) => serde_json::to_string(&response).unwrap(),
        Err(e) => format!("Error fetching wind info: {}", e),
    }
}

async fn get_wind_info(location: &str, date: &str) -> Result<WindInfoResponse, String> {
    let url = format!(
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}?unitGroup=metric&key={}&include=days",
        WEATHER_API_HOST, location, date, WEATHER_API_KEY
    );

    let request = build_http_request(url);

    match http_request(request).await {
        Ok((response,)) if response.status == 200 => {
            let str_body = String::from_utf8(response.body)
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let all_info: Value = serde_json::from_str(&str_body)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let wind_dir_degrees = all_info["days"][0]["winddir"].as_f64().unwrap_or(0.0);

            let wind_speed = all_info["days"][0]["windspeedmean"].as_f64()
                .or_else(|| all_info["days"][0]["windspeed"].as_f64())
                .unwrap_or(0.0);

            let wind_direction = get_wind_direction_text(wind_dir_degrees);

            let wind_power = get_wind_power_category(wind_speed);

            let result = WindInfoResponse {
                wind_direction,
                wind_power,
            };

            Ok(result)
        }
        Ok((response,)) => Err(format!("Non-200 response: {}", response.status)),
        Err((code, message)) => Err(format!("Request failed: {:?}, {}", code, message)),
    }
}

fn get_wind_direction_text(degrees: f64) -> String {
    let directions = [
        "Northern", "Northeastern", "Eastern", "Southeastern",
        "Southern", "Southwestern", "Western", "Northwestern"
    ];

    let normalized_degrees = (degrees % 360.0 + 360.0) % 360.0;

    let index = ((normalized_degrees + 22.5) % 360.0 / 45.0).floor() as usize;

    directions[index].to_string()
}

fn get_wind_power_category(speed_kph: f64) -> String {
    match speed_kph {
        s if s < 2.0 => "Windless",
        s if s < 12.0 => "Light wind",
        s if s < 29.0 => "Medium wind",
        s if s < 75.0 => "Strong wind",
        _ => "Hurricane wind",
    }.to_string()
}

//----------uv recommendation------------

#[update]
async fn get_uv_recommendation_endpoint(request: WeatherRequest) -> String {
    if request.token != API_TOKEN {
        return "Invalid API token.".to_string();
    }

    match get_uv_recommendation(&request.location, &request.date).await {
        Ok(response) => serde_json::to_string(&response).unwrap(),
        Err(e) => format!("Error fetching wind info: {}", e),
    }
}

async fn get_uv_recommendation(location: &str, date: &str) -> Result<UvRecommendation, String> {
    let url = format!(
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}?unitGroup=metric&key={}&include=days",
        WEATHER_API_HOST, location, date, WEATHER_API_KEY
    );

    let request = build_http_request(url);

    match http_request(request).await {
        Ok((response,)) if response.status == 200 => {
            let str_body = String::from_utf8(response.body)
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let all_info: Value = serde_json::from_str(&str_body)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let uvindex = all_info["days"][0]["uvindex"].as_f64().unwrap_or(0.0);

            let sunglasses_recommendation = get_sunglasses_recommendation(uvindex);

            let result = UvRecommendation {
                sunglasses_recommendation,
                uvindex,
            };

            Ok(result)
        }
        Ok((response,)) => Err(format!("Non-200 response: {}", response.status)),
        Err((code, message)) => Err(format!("Request failed: {:?}, {}", code, message)),
    }
}

fn get_sunglasses_recommendation(uvindex: f64) -> String {
    match uvindex {
        uv if uv < 3.0 => "Sunglasses are not necessary",
        uv if uv < 5.0 => "Sunglasses are recommended",
        _ => "Sunglasses are necessary",
    }.to_string()
}

#[update]
async fn get_precipitation_info_endpoint(request: WeatherRequest) -> String {
    if request.token != API_TOKEN {
        return "Invalid API token.".to_string();
    }

    match get_precipitation_info(&request.location, &request.date).await {
        Ok(response) => serde_json::to_string(&response).unwrap(),
        Err(e) => format!("Error fetching precipitation info: {}", e),
    }
}

async fn get_precipitation_info(location: &str, date: &str) -> Result<PrecipitationInfoResponse, String> {
    let url = format!(
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}?unitGroup=metric&key={}&include=days",
        WEATHER_API_HOST, location, date, WEATHER_API_KEY
    );

    let request = build_http_request(url);

    match http_request(request).await {
        Ok((response,)) if response.status == 200 => {
            let str_body = String::from_utf8(response.body)
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let all_info: Value = serde_json::from_str(&str_body)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let day = &all_info["days"][0];
            let precipitation_mm = day["precip"].as_f64().unwrap_or(0.0);
            let precipitation_type = day["preciptype"]
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("none")
                .to_string();

            Ok(PrecipitationInfoResponse {
                precipitation_mm,
                precipitation_type,
            })
        }
        Ok((response,)) => Err(format!("Non-200 response: {}", response.status)),
        Err((code, message)) => Err(format!("Request failed: {:?}, {}", code, message)),
    }
}

#[update]
async fn get_feels_like_temperature_endpoint(request: WeatherRequest) -> String {
    if request.token != API_TOKEN {
        return "Invalid API token.".to_string();
    }

    match get_feels_like_temperature(&request.location, &request.date).await {
        Ok(response) => serde_json::to_string(&response).unwrap(),
        Err(e) => format!("Error fetching 'feels like' temp: {}", e),
    }
}

async fn get_feels_like_temperature(location: &str, date: &str) -> Result<FeelsLikeTemperatureResponse, String> {
    let url = format!(
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}?unitGroup=metric&key={}&include=days",
        WEATHER_API_HOST, location, date, WEATHER_API_KEY
    );

    let request = build_http_request(url);

    match http_request(request).await {
        Ok((response,)) if response.status == 200 => {
            let str_body = String::from_utf8(response.body)
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let all_info: Value = serde_json::from_str(&str_body)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let day = &all_info["days"][0];
            let actual_temp_c = day["temp"].as_f64().unwrap_or(0.0);
            let feels_like_c = day["feelslike"].as_f64().unwrap_or(0.0);

            Ok(FeelsLikeTemperatureResponse {
                actual_temp_c,
                feels_like_c,
            })
        }
        Ok((response,)) => Err(format!("Non-200 response: {}", response.status)),
        Err((code, message)) => Err(format!("Request failed: {:?}, {}", code, message)),
    }
}

#[update]
async fn get_sun_times_endpoint(request: WeatherRequest) -> String {
    if request.token != API_TOKEN {
        return "Invalid API token.".to_string();
    }

    match get_sun_times(&request.location, &request.date).await {
        Ok(response) => serde_json::to_string(&response).unwrap(),
        Err(e) => format!("Error fetching sun times: {}", e),
    }
}

async fn get_sun_times(location: &str, date: &str) -> Result<SunTimesResponse, String> {
    let url = format!(
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}?unitGroup=metric&key={}&include=days",
        WEATHER_API_HOST, location, date, WEATHER_API_KEY
    );

    let request = build_http_request(url);

    match http_request(request).await {
        Ok((response,)) if response.status == 200 => {
            let str_body = String::from_utf8(response.body)
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let all_info: Value = serde_json::from_str(&str_body)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let sunrise = all_info["days"][0]["sunrise"].as_str().unwrap_or("unknown").to_string();
            let sunset = all_info["days"][0]["sunset"].as_str().unwrap_or("unknown").to_string();

            Ok(SunTimesResponse {
                sunrise,
                sunset,
            })
        }
        Ok((response,)) => Err(format!("Non-200 response: {}", response.status)),
        Err((code, message)) => Err(format!("Request failed: {:?}, {}", code, message)),
    }
}