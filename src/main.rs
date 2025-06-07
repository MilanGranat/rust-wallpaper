use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::{thread, time::Duration};

use std::time::SystemTime;
use chrono::{Datelike, Local, Timelike};
use serde::Deserialize;
use reqwest::blocking::get;

use dirs::config_dir;

// ---- WEATHER CONFIG STRUCTS ----
#[derive(Deserialize)]
struct WallpaperConfigItem {
   start_hour: i32,
   end_hour: i32,
   start_month: i32,
   end_month: i32,
   wallpaper_path: String,
   weather: Weather,
}

#[derive(Deserialize, PartialEq, Debug)]
enum Weather {
    Clear,
    Cloudy,
    Overcast,
    Rain,
    Thunderstorm,
    Snow,
    Fog
}

#[derive(Deserialize)]
struct WallpaperConfig {
    items: Vec<WallpaperConfigItem>,
    api_key: String,
    location: String,
}

// ---- WEATHER API STRUCTS ----
//
#[derive(Debug, Deserialize)]
struct WeatherApiResponse {
    current: Current,
}

#[derive(Debug, Deserialize)]
struct Current {
    condition: Condition,
}

// text is not used anymore, using codes now, but left it here for possible future use
#[derive(Debug, Deserialize)]
struct Condition {
    text: String,
    code: i32,
}

// get .config folder of current user and append our app path
fn get_config_path() -> Option<String> {
    config_dir().map(|mut path| {
        path.push("rust-wallpaper/wallpaper_config.json");
        path.to_str().unwrap_or_default().to_string()
    })
}

// ---- MAIN ----
fn main() {
    let mut last_modified: Option<SystemTime> = None;
    let mut config: Option<WallpaperConfig> = None;
    let mut current_condition: Weather = Weather::Clear;
    let mut current_wallpaper: String = String::from("");

    let config_path = get_config_path().expect("Could not determine config path");

    loop {
        // check loaded file & reload if needed
        match fs::metadata(&config_path) {
            Ok(metadata) => {
                match metadata.modified() {
                    Ok(modified_time) => {
                        let reload_needed = match last_modified {
                            Some(prev_time) => modified_time > prev_time,
                            None => true,
                        };

                        if reload_needed {
                            println!("Reloading config...");
                            match load_config_with_timestamp(&config_path) {
                                Ok((new_config, new_time)) => {
                                    config = Some(new_config);
                                    last_modified = Some(new_time);
                                }
                                Err(e) => {
                                    eprintln!("Failed to reload config: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Couldn't get file modification time: {}", e);
                    }
                }
            }
            Err(_) => {
                eprintln!("Config file not found.");
            }
        }

        // apply config
        if let Some(ref conf) = config {
            if conf.api_key.is_empty() || conf.location.is_empty() {
                current_condition = Weather::Clear;
            } else {
                match fetch_weather_condition(&conf.api_key, &conf.location) {
                        Ok(condition) => {current_condition = condition;},
                        Err(e) => eprintln!("Failed to fetch weather: {}", e),
                }
            }

            let wallpaper_path = get_best_wallpaper_match(&current_condition, conf);

            if !wallpaper_path.is_empty() {
                if (current_wallpaper != wallpaper_path) {
                    set_wallpaper(&wallpaper_path);
                    current_wallpaper = wallpaper_path;
                }
            } else {
                println!("No match");
            }
            thread::sleep(Duration::from_secs(300)); // 5 minutes
        }
    }
}

// fn to find the best wallpaper match for current month, hour and weather conditions.
// if doesnt find exact match, or file is missing, tries to find for similar weather, then for
// less similar weather, if nothing found, adds hour. If doesnt find for any hour, adds month
fn get_best_wallpaper_match(weather: &Weather, conf: &WallpaperConfig) -> String {

    let now = Local::now();
    let mut now_month = now.month() as i32;
    let mut now_hour = now.hour() as i32;
    println!("Checking wallpaper for month {} and hour {} and weather {:?}", now_month, now_hour, &weather);

    for m in 1..12 {
        for h in 1..24 {
            let any_match : Vec<&WallpaperConfigItem> = conf.items.iter()
                .filter(|item| {
                    let month_match = if item.start_month > item.end_month {
                        item.start_month <= now_month || item.end_month >= now_month
                    } else {
                        item.start_month <= now_month && item.end_month >= now_month
                    };

                    let hour_match = if item.start_hour > item.end_hour {
                        item.start_hour <= now_hour || item.end_hour >= now_hour
                    } else {
                        item.start_hour <= now_hour && item.end_hour >= now_hour
                    };

                    month_match && hour_match
                }).collect();

            if !any_match.is_empty() {
                let exact_match = any_match.iter().find(|item| {
                    item.weather == *weather
                });

                if let Some(item) = exact_match {
                    if Path::new(&item.wallpaper_path).exists() {
                        return item.wallpaper_path.clone();
                    }
                    println!("file for exact match not found {}", &item.wallpaper_path);
                }
                println!("no exact match found with existing file found");

                let similar_match = any_match.iter()
                    .filter(|item| is_similar_weather(&item.weather, weather))
                    .find(|item| Path::new(&item.wallpaper_path).exists());

                if let Some(item) = similar_match {
                    return item.wallpaper_path.clone();
                }
                println!("no similar weather file found");

                let less_similar_match = any_match.iter()
                    .filter(|item| is_less_similar_weather(&item.weather, weather))
                    .find(|item| Path::new(&item.wallpaper_path).exists());

                if let Some(item) = less_similar_match {
                    return item.wallpaper_path.clone();
                }

                println!("no less similar weather file found");
                let clear_match = any_match.iter().find(|item| {
                    item.weather == Weather::Clear
                });

                if let Some(item) = clear_match {
                    if Path::new(&item.wallpaper_path).exists() {
                        return item.wallpaper_path.clone();
                    }
                    println!("No clear default found");
                }
            }
            print!("no match found for this hour, adding");
            now_hour = now_hour + 1;
            if (now_hour > 23) {
                now_hour = 0;
            }

        } 
        println!("no match found for this month, adding");
        now_month = now_month + 1;
        if (now_month > 12) {
            now_month = 1;
        }
    }

    String::from("")
}

// loads config and timestamp of config modification, so that we can check for modification during
// runtim
fn load_config_with_timestamp(path: &str) -> Result<(WallpaperConfig, SystemTime), io::Error> {
    if !Path::new(path).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Config file not found"));
    }

    let metadata = fs::metadata(path)?;
    let modified_time = metadata.modified()?;

    let contents = fs::read_to_string(path)?;
    let config: WallpaperConfig = serde_json::from_str(&contents)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok((config, modified_time))
}

// tries to set the wallpaper given, sets for both light and dark mode
fn set_wallpaper(path: &str) {
    if !Path::new(path).exists() {
        eprintln!("Wallpaper file not found: {}", path);
        return;
    }

    let uri = format!("file://{}", path);
    // Set both light and dark wallpaper
    let keys = ["picture-uri", "picture-uri-dark"];
    for key in &keys {
        let result = Command::new("gsettings")
            .args(&["set", "org.gnome.desktop.background", key, &uri])
            .status();

        match result {
            Ok(status) if status.success() => {
                println!("Set {} to: {}", key, path);
            }
            Ok(status) => {
                eprintln!("gsettings failed setting {} with exit code: {}", key, status);
            }
            Err(e) => {
                eprintln!("Failed to execute gsettings for {}: {}", key, e);
            }
        }
    }
}

// gets weather condition for current location from service
fn fetch_weather_condition(api_key: &str, city: &str) -> Result<Weather, Box<dyn std::error::Error>> {
    let url = format!(
        "http://api.weatherapi.com/v1/current.json?key={}&q={}&aqi=no",
        api_key, city
    );

    let response = get(&url)?.json::<WeatherApiResponse>()?;

    Ok(map_weather_code_to_enum(response.current.condition.code))
}

// maps the weather condition code to one of the closest enum conditions
fn map_weather_code_to_enum(code: i32) -> Weather {
    match code {
        1000 => Weather::Clear, // Sunny/Clear
        1003 => Weather::Cloudy, // Partly cloudy
        1006 | 1063 | 1066 | 1069 => Weather::Cloudy,
        1009 => Weather::Overcast,
        1030 | 1135 | 1147 => Weather::Fog, // Mist, Fog, Freezing fog

        // Rain-related codes
        1150 | 1153 | 1168 | 1171 |
        1180 | 1183 | 1186 | 1189 | 1192 | 1195 |
        1198 | 1201 |
        1240 | 1243 | 1246  => Weather::Rain,

        // Thunderstorm-related codes
        1087 | 1273 | 1276 => Weather::Thunderstorm,

        // Snow-related codes
        1114 | 1117 |
        1204 | 1207 |
        1210 | 1213 | 1216 | 1219 | 1222 | 1225 |
        1237 |
        1255 | 1258 |
        1261 | 1264 |
        1279 | 1282 => Weather::Snow,

        // Sleet and freezing drizzle – treat as Snow for simplicity
        1072 | 1249 | 1252 => Weather::Snow,

        _ => Weather::Clear, // default fallback
    }
}

// map of more similar weather conditions (for example: cloudy is similar to clear and overcast, but not very similar to rain)
fn is_similar_weather(a: &Weather, b: &Weather) -> bool {
    use Weather::*;
    matches!(
        (a, b),
        (Clear, Cloudy) | (Cloudy, Clear)
        | (Cloudy, Overcast) | (Overcast, Cloudy)
        | (Rain, Thunderstorm) | (Thunderstorm, Rain)
        | (Snow, Cloudy) | (Cloudy, Snow)
        | (Fog, Overcast) | (Overcast, Fog)
    )
}

// maps a bit less similar weather conditions (for example: cloudy is similar even to rainy in this scenario)
fn is_less_similar_weather(a: &Weather, b: &Weather) -> bool {
    use Weather::*;
    matches!(
        (a, b),
        (Clear, Overcast) | (Overcast, Clear)
        | (Cloudy, Rain) | (Rain, Cloudy)
        | (Cloudy, Thunderstorm) | (Thunderstorm, Cloudy)
        | (Snow, Clear) | (Clear, Snow)
        | (Rain, Overcast) | (Overcast, Rain)
        | (Fog, Overcast) | (Overcast, Fog)
    )
}
