use std::{thread, time};

use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::str::FromStr;

use chrono::prelude::*;
use clap::crate_authors;
use clap::crate_description;
use clap::crate_name;
use clap::crate_version;
use clap::{App, Arg};
use csv::Writer;
use hueclient::bridge::Bridge;
use indicatif::{ProgressBar, ProgressStyle};
use spa::{calc_sunrise_and_set, SunriseAndSet};
use url::Url;

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
struct WeatherClouds {
    all: i64,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
struct WeatherMain {
    feels_like: f64,
    humidity: f64,
    pressure: f64,
    temp: f64,
    temp_max: f64,
    temp_min: f64,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
struct WeatherDescription {
    id: i64,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
struct WeatherWind {
    deg: Option<i64>,
    speed: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
struct WeatherRain {
    #[serde(rename(serialize = "1h"))]
    one_h: Option<i64>,
    #[serde(rename(serialize = "3h"))]
    three_h: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
struct WeatherSnow {
    #[serde(rename(serialize = "1h"))]
    one_h: Option<i64>,
    #[serde(rename(serialize = "3h"))]
    three_h: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct WeatherResponse {
    main: WeatherMain,
    visibility: i64,
    weather: Vec<WeatherDescription>,
    clouds: Option<WeatherClouds>,
    wind: Option<WeatherWind>,
    rain: Option<WeatherRain>,
    snow: Option<WeatherSnow>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OutputRow {
    timestamp: String,
    weather_main_feels_like: f64,
    weather_main_humidity: f64,
    weather_main_pressure: f64,
    weather_main_temp: f64,
    weather_main_temp_max: f64,
    weather_main_temp_min: f64,
    weather_visibility: i64,
    weather_weather_0_id: Option<i64>,
    weather_clouds_all: Option<i64>,
    weather_wind_deg: Option<i64>,
    weather_wind_speed: Option<f64>,
    weather_rain_1h: Option<i64>,
    weather_rain_3h: Option<i64>,
    weather_snow_1h: Option<i64>,
    weather_snow_3h: Option<i64>,
    light_name: String,
    light_modelid: String,
    light_swversion: String,
    light_uniqueid: String,
    light_state_on: bool,
    light_state_bri: u8,
    light_state_hue: u16,
    light_state_sat: u8,
    light_state_ct: Option<u16>,
    light_state_xy_0: Option<f32>,
    light_state_xy_1: Option<f32>,
    id: usize,
    daylight: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new(crate_name!())
        .bin_name(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("hue-user")
                .short("u")
                .long("hue-user")
                .required(true)
                .takes_value(true)
                .help("Username to use on the hue hub to interact with the API"),
        )
        .arg(
            Arg::with_name("openweather-api-key")
                .short("w")
                .long("openweather-api-key")
                .required(true)
                .takes_value(true)
                .help("API key to use to interact with the OpenWeather API"),
        )
        .arg(
            Arg::with_name("longitude")
                .short("l")
                .long("lon")
                .takes_value(true)
                .required(true)
                .help("The longitude where the readings are being taken from"),
        )
        .arg(
            Arg::with_name("poll")
                .short("p")
                .long("poll-every-seconds")
                .takes_value(true)
                .default_value("60")
                .help("Poll every seconds"),
        )
        .arg(
            Arg::with_name("latitude")
                .short("a")
                .long("lat")
                .takes_value(true)
                .required(true)
                .help("The latitude where the readings are being taken from"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .required(false)
                .help("Where to write the CSV to")
                .default_value("/dev/stdout"),
        )
        .get_matches();

    let openweather_api = matches.value_of("openweather-api-key").unwrap();
    let hue_user = matches.value_of("hue-user").unwrap();
    let output = matches.value_of("output").unwrap_or("/dev/stdout");
    let lat = f64::from_str(matches.value_of("latitude").unwrap())?;
    let lon = f64::from_str(matches.value_of("longitude").unwrap())?;
    let poll_seconds = i64::from_str(matches.value_of("poll").unwrap())?;

    let is_stdout = output == "/dev/stdout";
    let path = Path::new(output);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)?;

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(is_stdout || !path.is_file() || path.metadata()?.len() == 0)
        .from_writer(file);

    let bridge = Bridge::discover().unwrap().with_user(hue_user.into());

    poll(
        openweather_api,
        lat,
        lon,
        is_stdout,
        &mut csv_writer,
        &bridge,
        poll_seconds,
    )?;

    Ok(())
}

fn poll(
    openweather_api: &str,
    lat: f64,
    lon: f64,
    is_stdout: bool,
    mut csv_writer: &mut Writer<File>,
    bridge: &Bridge,
    poll_every_seconds: i64,
) -> Result<(), Box<dyn Error>> {
    let progress = progress_bar(is_stdout);

    for i in 0.. {
        if i % poll_every_seconds == 0 {
            output_lights(
                &mut csv_writer,
                &bridge,
                &progress,
                lat,
                lon,
                openweather_api,
            )?;
        }

        progress.set_message(&format!("Sleeping for {} sec", poll_every_seconds));
        progress.tick();
        thread::sleep(time::Duration::from_secs(1));
    }

    Ok(())
}

fn progress_bar(is_stdout: bool) -> ProgressBar {
    let progress = if is_stdout {
        ProgressBar::hidden()
    } else {
        ProgressBar::new_spinner()
    };

    progress.set_style(ProgressStyle::default_bar().template("{spinner} {elapsed_precise} {msg}"));
    progress.set_message("Starting event loop");
    progress.tick();
    progress
}

fn output_lights(
    file_writer: &mut Writer<File>,
    bridge: &Bridge,
    progress: &ProgressBar,
    lat: f64,
    lon: f64,
    openweather_api_key: &str,
) -> Result<(), Box<dyn Error>> {
    progress.set_message("Polling hub");
    progress.tick();

    let daylight = match calc_sunrise_and_set(Utc::now(), lat, lon)? {
        SunriseAndSet::PolarNight => false,
        SunriseAndSet::PolarDay => true,
        SunriseAndSet::Daylight(rise, set) => rise <= Utc::now() && Utc::now() <= set,
    };

    let weather: WeatherResponse = reqwest::blocking::get(Url::parse_with_params(
        "https://api.openweathermap.org/data/2.5/weather",
        &[
            ("lat", lat.to_string()),
            ("lon", lon.to_string()),
            ("appid", openweather_api_key.to_string()),
        ],
    )?)?
    .json()?;

    for light in &bridge.get_all_lights().unwrap() {
        progress.set_message(&format!(
            "Reading light {} ({})",
            &light.light.name, &light.id
        ));
        progress.tick();

        file_writer.serialize(OutputRow {
            timestamp: Local::now().to_rfc3339(),
            weather_main_feels_like: weather.main.feels_like,
            weather_main_humidity: weather.main.humidity,
            weather_main_pressure: weather.main.pressure,
            weather_main_temp: weather.main.temp,
            weather_main_temp_max: weather.main.temp_max,
            weather_main_temp_min: weather.main.temp_min,
            weather_visibility: weather.visibility,
            weather_weather_0_id: weather.weather.get(0).map(|x| x.id),
            weather_clouds_all: weather.clouds.map(|x| x.all),
            weather_wind_deg: weather.wind.and_then(|x| x.deg),
            weather_wind_speed: weather.wind.and_then(|x| x.speed),
            weather_rain_1h: weather.rain.and_then(|x| x.one_h),
            weather_rain_3h: weather.rain.and_then(|x| x.three_h),
            weather_snow_1h: weather.snow.and_then(|x| x.one_h),
            weather_snow_3h: weather.snow.and_then(|x| x.three_h),
            light_name: light.light.name.clone(),
            light_modelid: light.light.modelid.clone(),
            light_swversion: light.light.swversion.clone(),
            light_uniqueid: light.light.uniqueid.clone(),
            light_state_on: light.light.state.on,
            light_state_bri: light.light.state.bri,
            light_state_hue: light.light.state.hue,
            light_state_sat: light.light.state.sat,
            light_state_ct: light.light.state.ct,
            light_state_xy_0: light.light.state.xy.map(|x| x.0),
            light_state_xy_1: light.light.state.xy.map(|x| x.1),
            id: light.id,
            daylight,
        })?;
        file_writer.flush()?;
    }

    Ok(())
}
