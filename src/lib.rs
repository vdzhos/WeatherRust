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
struct WeatherResponse {
    temp_c: f64,
    wind_kph: f64,
    pressure_mb: f64,
    humidity: f64,
}


const API_TOKEN: &str = "3e2f4d6a5b8c9e1f1234abcd5678ef90";
const WEATHER_API_KEY: &str = "4Z665XTN8EYM7AYK38P7NXJM5";
const WEATHER_API_HOST: &str = "weather.visualcrossing.com";
