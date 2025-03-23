use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use ic_cdk_macros::{query, update};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use candid::{CandidType, Principal};


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

const API_TOKEN: &str = "3e2f4d6a5b8c9e1f1234abcd5678ef90";
const WEATHER_API_KEY: &str = "UXD6LQ27FMXMXZ4D5444KD6FV";
const WEATHER_API_HOST: &str = "weather.visualcrossing.com";

fn build_http_request(url: String) -> CanisterHttpRequestArgument {
    CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        headers: vec![HttpHeader {
            name: "User-Agent".to_string(),
            value: "Rust Weather Canister".to_string(),
        }],
        body: None,
        max_response_bytes: None,
        transform: None,
    }
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
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}?unitGroup=metric&key={}",
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
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}/{}?unitGroup=metric&key={}",
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
        "https://{}/VisualCrossingWebServices/rest/services/timeline/{}/{}?unitGroup=metric&key={}",
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