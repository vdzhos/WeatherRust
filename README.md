# WeatherRust Project - README

## Overview

The **WeatherRust** project is a decentralized application (DApp) built on the **Internet Computer Protocol (ICP)**. This project utilizes **Rust** for its backend implementation and provides a set of endpoints to access weather data via a weather API ([visualcrossing API docs](https://www.visualcrossing.com/resources/documentation/weather-api/timeline-weather-api/)). The main objective of the WeatherRust project is to provide weather information in various formats and for different locations, offering a rich set of weather-related services to end-users.

## Functionality

This project is implemented as a set of **canister** endpoints, each responsible for querying a different type of weather data from the external weather API. The canisters are called through **Candid UI**, which exposes each endpoint as a service that can be invoked with the appropriate request parameters.

### Features

The WeatherRust service supports the following functionalities:

1. **Weather Information for a Single Date**:  
   Users can query the weather for a specific location and date. This endpoint returns data like temperature, wind speed, pressure, and humidity.

2. **Average Weather Information for a Range**:  
   Users can query the average weather for a location over a range of dates. This endpoint will provide average temperature, wind speed, pressure, and humidity over the specified range.

3. **Compare Weather for Two Locations**:  
   Users can compare the weather between two different locations for a given date. This endpoint will return weather data for both locations side by side, making it easy to compare their conditions.

4. **Weather Stations**:  
   This endpoint provides information on the weather stations available in a given location on a specific date.

5. **Wind Information**:  
   Fetches detailed wind information, including wind direction and strength, based on the requested location and date.

6. **UV Index and Sunglasses Recommendation**:  
   Based on the UV index, this endpoint recommends whether sunglasses are necessary for a given location and date.

7. **Precipitation Information**:  
   Provides information on precipitation levels and types, including rain or snow, for a specific location on a given date.

8. **Feels Like Temperature**:  
   This endpoint returns the "feels-like" temperature, which is the temperature adjusted for factors such as wind chill and humidity, providing a more accurate representation of how the temperature is experienced.

9. **Sun Times**:  
   Provides the sunrise and sunset times for a given location and date.

### Candid UI

The Candid UI is the interface through which users can interact with the canisters. It provides the following services:

- **weather_endpoint**:
    - Accepts the following fields: `token`, `requester_name`, `location`, `date`.
    - Returns a string with the weather data for the requested location and date.

- **average_weather_on_range_endpoint**:
    - Accepts: `token`, `requester_name`, `location`, `start_date`, `end_date`.
    - Returns the average weather data over the specified date range.

- **compare_weather_endpoint**:
    - Accepts: `token`, `requester_name`, `location1`, `location2`, `date`.
    - Returns weather data comparing two locations for a given date.

- **get_weather_stations_endpoint**:
    - Accepts: `token`, `requester_name`, `location`, `date`.
    - Returns information about available weather stations for the given location and date.

- **get_wind_info_endpoint**:
    - Accepts: `token`, `requester_name`, `location`, `date`.
    - Returns detailed wind information for a specified location and date.

- **get_uv_recommendation_endpoint**:
    - Accepts: `token`, `requester_name`, `location`, `date`.
    - Returns the UV index and sunglasses recommendation based on the weather for the given location and date.

- **get_precipitation_info_endpoint**:
    - Accepts: `token`, `requester_name`, `location`, `date`.
    - Returns the precipitation information for a specific location on a given date.

- **get_feels_like_temperature_endpoint**:
    - Accepts: `token`, `requester_name`, `location`, `date`.
    - Returns the "feels like" temperature for the given location and date.

- **get_sun_times_endpoint**:
    - Accepts: `token`, `requester_name`, `location`, `date`.
    - Returns the sunrise and sunset times for the given location and date.

## How It Works

1. **Canisters**:  
   The backend is implemented using **Rust**, and the canisters are compiled to **WebAssembly (Wasm)**. The canisters are deployed on the **Internet Computer** (ICP). The canisters expose various endpoints to handle weather data requests.

2. **HTTP Requests**:  
   The canisters make HTTP requests to an external weather API to fetch the necessary weather data. The data is then processed and formatted into a suitable response that is sent back to the requester.

3. **Data Processing**:  
   The data returned from the API is parsed into structured formats using **serde** and **serde_json**. The weather data, once parsed, is serialized into the appropriate response format (e.g., `WeatherResponse`, `AverageWeatherResponse`, etc.).

4. **Authentication**:  
   The endpoints require a valid API token for authentication, ensuring that only authorized requests are processed.

5. **Candid Interface**:  
   The Candid interface makes it easy to interact with the canisters by defining the inputs and outputs for each service. This enables users to invoke the services via simple record-based queries.
