use byte_unit::{Byte, ByteUnit};
use chrono::{DateTime, NaiveDateTime, Utc};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use colored::Colorize;
use discord::Client;
use discord_sdk::activity::{ActivityArgs, ActivityKind, Assets, IntoTimestamp, Timestamps};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};
use sysinfo::{System, SystemExt};
use tokio::{task::block_in_place, time::Interval};

mod discord;

struct Ts(u64);
impl IntoTimestamp for Ts {
    fn into_timestamp(self) -> i64 {
        self.0 as i64
    }
}

/// The task to update Discord with system info
async fn update_discord(client: &Client, system: &mut System) {
    // Fetch system status
    system.refresh_cpu();
    system.refresh_memory();

    let processors = system.processors().len();
    let total_memory = Byte::from_unit(system.total_memory() as f64, ByteUnit::KB).unwrap();
    let total_memory = total_memory.get_adjusted_unit(ByteUnit::GB);
    let used_memory = Byte::from_unit(system.used_memory() as f64, ByteUnit::KB).unwrap();
    let used_memory = used_memory.get_adjusted_unit(ByteUnit::GB);
    let used_percent = used_memory.get_value() / total_memory.get_value();
    let load = system.load_average();
    let boot_time = system.boot_time();
    let os_name = system.name();
    let os_version = system.os_version();
    let kernel_version = system.kernel_version();

    // Build rich presence
    let rp = discord_sdk::activity::ActivityBuilder::default()
        .state(format!(
            "CORES: {} | MEM: {}",
            processors,
            total_memory.to_string()
        ))
        .details(format!("CPU: {:.2}% | RAM: {:.2}%", load.one, used_percent));

    // .timestamps(None, None);

    // Detect the image string to use
    let image_string;
    if cfg!(windows) {
        image_string = "os-windows";
    } else if cfg!(linux) {
        image_string = "os-linux";
    } else {
        image_string =match os_type::current_platform().os_type {
            os_type::OSType::OSX => "os_unix",
            os_type::OSType::Unknown => "os_unix",
            _ => "os-linux",
        };        
    }

    // Build activity
    let mut activity: ActivityArgs = rp.into();
    let inner = activity.activity.as_mut().unwrap();
    inner.assets = Some(Assets {
        large_image: Some(image_string.to_string()),
        large_text: Some(format!(
            "{} {}",
            os_name.unwrap_or("Unknown".to_string()),
            os_version.unwrap_or("".to_string())
        )),
        small_image: Some("dot-blue".to_string()),
        small_text: kernel_version,
    });
    // inner.kind = ActivityKind::Streaming;
    // inner.timestamps = Some(Timestamps {
    //     start: boot_time as i64,
    //     end: 0,
    // });

    // Update Discord
    client.discord.update_activity(activity).await.unwrap();
}

/// Wrapper for the discord updater. This handles the event loop
async fn rpc_runner_run(running: &Arc<AtomicBool>, interval: &mut Interval, client: &Client) {
    // Set up a system info reader
    let mut system = System::default();

    // Event loop
    while running.load(Ordering::SeqCst) {
        // Tick to the next period of the interval
        interval.tick().await;

        // Update discord
        update_discord(client, &mut system).await;
    }
}

#[tokio::main]
async fn main() {
    let matches = App::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("interval")
                .long("interval")
                .short("i")
                .default_value("1")
                .help("Number of seconds to wait between updates")
                .required(false)
                .takes_value(true),
        )
        .get_matches();

    // Get arg data
    let interval_int: u64 = matches
        .value_of("interval")
        .unwrap()
        .parse()
        .expect("Interval must be an integer");

    // Set up the update interval
    let mut interval = tokio::time::interval(Duration::from_secs(interval_int));

    // Set up and connect to Discord
    let discord_client = discord::make_client(discord_sdk::Subscriptions::ACTIVITY).await;
    let mut activity_events = discord_client.wheel.activity();

    // Create the arc for sig handling
    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    }

    // Spawn a handler for discord RPC events incoming
    tokio::task::spawn(async move {
        while let Ok(ae) = activity_events.0.recv().await {
            tracing::info!(event = ?ae, "received activity event");
        }
    });

    // Main event loop
    println!("{}", "Press CTRL+C to stop".blue());
    rpc_runner_run(&running, &mut interval, &discord_client).await;

    // Clean up and shut down
    println!("{}", "Shutting down".blue());
    discord_client.discord.disconnect().await;
}
