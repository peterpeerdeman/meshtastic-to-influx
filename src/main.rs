/// This example connects to a TCP port on the radio, and prints out all received packets.
/// This can be used with a simulated radio via the Meshtastic Docker firmware image.
/// https://meshtastic.org/docs/software/linux-native#usage-with-docker
extern crate meshtastic;

use std::io::{self, BufRead};
use std::time::SystemTime;

use meshtastic::api::StreamApi;
use meshtastic::utils;
use influxdb::{Client, Error, InfluxDbWriteable, ReadQuery, Timestamp};
use chrono::{DateTime, Utc};

/// Set up the logger to output to stdout  
/// **Note:** the invokation of this function is commented out in main by default.
fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Uncomment this to enable logging
    //setup_logger()?;

    let client = Client::new("http://localhost:8086", "meshtastic");
    let stream_api = StreamApi::new();

    let address = "192.168.117.176:4403".to_string();

    let tcp_stream = utils::stream::build_tcp_stream(address).await?;
    let (mut decoded_listener, stream_api) = stream_api.connect(tcp_stream).await;

    let config_id = utils::generate_rand_id();
    let stream_api = stream_api.configure(config_id).await?;

    // This loop can be broken with ctrl+c, or by unpowering the radio.
    while let Some(decoded_packet) = decoded_listener.recv().await {
        // Only process and display NodeInfo packets
        handle_from_radio_packet(decoded_packet, &client)
    }

    // Note that in this specific example, this will only be called when
    // the radio is disconnected, as the above loop will never exit.
    // Typically you would allow the user to manually kill the loop,
    // for example with tokio::select!.
    let _stream_api = stream_api.disconnect().await?;

    Ok(())
}


/// A helper function to handle packets coming directly from the radio connection.
/// The Meshtastic `PhoneAPI` will return decoded `FromRadio` packets, which
/// can then be handled based on their payload variant. Note that the payload
/// variant can be `None`, in which case the packet should be ignored.
fn handle_from_radio_packet(from_radio_packet: meshtastic::protobufs::FromRadio, client: &Client) {
    // Remove `None` variants to get the payload variant
    let payload_variant = match from_radio_packet.payload_variant {
        Some(payload_variant) => payload_variant,
        None => {
            //println!("Received FromRadio packet with no payload variant, not handling...");
            return;
        }
    };

    // `FromRadio` packets can be differentiated based on their payload variant,
    // which in Rust is represented as an enum. This means the payload variant
    // can be matched on, and the appropriate user-defined action can be taken.
    match payload_variant {
        //meshtastic::protobufs::from_radio::PayloadVariant::Channel(channel) => {
        //    println!("Received channel packet: {:?}", channel);
        //}
        meshtastic::protobufs::from_radio::PayloadVariant::NodeInfo(node_info) => {
            handle_nodeinfo_packet(node_info, &client)
            // println!("Received node info packet: {:?}", node_info);
        }
        //meshtastic::protobufs::from_radio::PayloadVariant::Packet(mesh_packet) => {
        //    println!("Received mesh packet");
        //}
        _ => {
            //println!("Received other FromRadio packet, not handling...");
            return;
        }
    };
}

#[derive(InfluxDbWriteable, Clone)]
struct NodeInfoReading {
    time: DateTime<Utc>,
    #[influxdb(tag)]
    node_id: String,
    #[influxdb(tag)] 
    long_name: String,
    #[influxdb(tag)]
    short_name: String,
    snr: f32,
    battery_level: u32,
    voltage: f32,
    channel_utilization: f32,
    air_util_tx: f32,
    #[influxdb(tag)]
    hops_away: u32
}

fn handle_nodeinfo_packet(node_info: meshtastic::protobufs::NodeInfo, client: &Client) {
    // Extract user info if available
    let (node_id, long_name, short_name) = match &node_info.user {
        Some(user) => (
            user.id.clone(),
            user.long_name.clone(),
            user.short_name.clone()
        ),
        None => (
            String::from("unknown"),
            String::from("unknown"),
            String::from("unknown")
        )
    };

    // Extract device metrics if available
    let (battery_level, voltage, channel_util, air_util) = match &node_info.device_metrics {
        Some(metrics) => (
            metrics.battery_level,
            metrics.voltage,
            metrics.channel_utilization,
            metrics.air_util_tx
        ),
        None => (0, 0.0, 0.0, 0.0)
    };

    let reading = NodeInfoReading {
        time: Utc::now(),
        node_id,
        long_name,
        short_name,
        snr: node_info.snr,
        battery_level,
        voltage,
        channel_utilization: channel_util,
        air_util_tx: air_util,
        hops_away: node_info.hops_away
    };

    let readings = vec![reading.into_query("node_info")];
    println!("{:?}",readings);
    // client.query(readings).await.unwrap();
}
