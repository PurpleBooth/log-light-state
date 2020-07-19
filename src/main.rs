use std::error::Error;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::{thread, time};

use chrono::prelude::*;
use clap::crate_authors;
use clap::crate_description;
use clap::crate_name;
use clap::crate_version;
use clap::{App, Arg};
use csv::Writer;
use hueclient::bridge::Bridge;
use indicatif::{ProgressBar, ProgressStyle};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new(crate_name!())
        .bin_name(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("hue-user")
                .required(true)
                .help("Username to use on the hue hub to interact with the API"),
        )
        .arg(
            Arg::with_name("output")
                .required(false)
                .help("Where to write the CSV to")
                .default_value("/dev/stdout"),
        )
        .get_matches();

    let hue_user = matches.value_of("hue-user").unwrap();
    let output = matches.value_of("output").unwrap_or("/dev/stdout");
    let is_stdout = output == "/dev/stdout";

    let path = Path::new(output);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)?;

    let mut csv_writer = csv::Writer::from_writer(file);
    let bridge = Bridge::discover().unwrap().with_user(hue_user.into());

    let headers = &[
        "timestamp",
        "id",
        "light.name",
        "light.modelid",
        "light.swversion",
        "light.uniqueid",
        "light.state.on",
        "light.state.bri",
        "light.state.hue",
        "light.state.sat",
        "light.state.ct",
        "light.state.xy.0",
        "light.state.xy.1",
    ];

    if is_stdout || !path.is_file() {
        csv_writer.write_record(headers)?;
        csv_writer.flush()?;
    }

    let progress = if is_stdout {
        ProgressBar::hidden()
    } else {
        ProgressBar::new_spinner()
    };

    progress.set_style(ProgressStyle::default_bar().template("{spinner} {elapsed_precise} {msg}"));
    progress.set_message("Starting event loop");
    progress.tick();

    for i in 0.. {
        if i % 60 == 0 {
            output_lights(&mut csv_writer, &bridge, &progress)?;
        }

        progress.set_message(&format!("Sleeping for {} sec", 60));
        progress.tick();
        thread::sleep(time::Duration::from_secs(1));
    }

    Ok(())
}

fn output_lights(
    file_writer: &mut Writer<File>,
    bridge: &Bridge,
    progress: &ProgressBar,
) -> Result<(), Box<dyn Error>> {
    progress.set_message("Polling hub");
    progress.tick();

    for light in &bridge.get_all_lights().unwrap() {
        progress.set_message(&format!(
            "Reading light {} ({})",
            &light.light.name, &light.id
        ));
        progress.tick();

        let row = &[
            &Local::now().to_rfc3339(),
            &format!("{}", light.id),
            &light.light.name,
            &light.light.modelid,
            &light.light.swversion,
            &light.light.uniqueid,
            &light.light.state.on.to_string(),
            &light.light.state.bri.to_string(),
            &light.light.state.hue.to_string(),
            &light.light.state.sat.to_string(),
            &light
                .light
                .state
                .ct
                .map_or_else(|| "NaN".into(), |z| z.to_string()),
            &light
                .light
                .state
                .xy
                .map_or_else(|| "NaN".into(), |x| x.0.to_string()),
            &light
                .light
                .state
                .xy
                .map_or_else(|| "NaN".into(), |x| x.1.to_string()),
        ];

        file_writer.write_record(row)?;
        file_writer.flush()?;
    }

    Ok(())
}
