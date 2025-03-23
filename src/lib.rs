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

const API_TOKEN: &str = "3e2f4d6a5b8c9e1f1234abcd5678ef90";
const WEATHER_API_KEY: &str = "4Z665XTN8EYM7AYK38P7NXJM5";
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
