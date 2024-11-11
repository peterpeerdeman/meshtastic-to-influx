extern crate meshtastic;

mod models;
mod handler;
use crate::models::NodeInfoReading;
use crate::handler::handle_nodeinfo_packet;

use meshtastic::api::StreamApi;
use meshtastic::utils;
use influxdb::Client;
use influxdb::InfluxDbWriteable;

use structured_logger::{async_json::new_writer, Builder};
use tokio::time::{sleep, Duration};

fn handle_from_radio_packet(from_radio_packet: meshtastic::protobufs::FromRadio, readings: &mut Vec<NodeInfoReading>) {
    let payload_variant = match from_radio_packet.payload_variant {
        Some(payload_variant) => payload_variant,
        None => {
            return;
        }
    };

    if let meshtastic::protobufs::from_radio::PayloadVariant::NodeInfo(node_info) = payload_variant {
        if node_info.last_heard == 0 {
            return;
        }
        // NodeInfo { num: 3663944616, user: None, position: Some(Position { latitude_i: 526477974, longitude_i: 50414422, altitude: 10, time: 0, location_source: LocUnset, altitude_source: AltUnset, timestamp: 0, timestamp_millis_adjust: 0, altitude_hae: 0, altitude_geoidal_separation: 0, pdop: 0, hdop: 0, vdop: 0, gps_accuracy: 0, ground_speed: 0, ground_track: 0, fix_quality: 0, fix_type: 0, sats_in_view: 0, sensor_id: 0, next_update: 0, seq_number: 0, precision_bits: 0 }), snr: 0.0, last_heard: 1731338908, device_metrics: Some(DeviceMetrics { battery_level: 101, voltage: 4.306, channel_utilization: 1.6433334, air_util_tx: 0.6466389 }), channel: 0, via_mqtt: true, hops_away: 0 }
        handle_nodeinfo_packet(node_info, readings)
    };
}

async fn push_queries_to_influx(client: Client, readings: Vec<NodeInfoReading>) {
    let queries: Vec<_> = readings.iter().map(|r| r.clone().into_query("node_info")).collect();
    client.query(queries).await.unwrap();
    log::info!(target="metrics", number_of_nodeinfo = readings.len(); "Successfully submitted node_info readings to InfluxDB");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get environment variables
    let influxdb_url = std::env::var("INFLUXDB_URL")
        .unwrap_or_else(|_| "http://localhost:8086".to_string());
    let influxdb_database = std::env::var("INFLUXDB_DATABASE")
        .unwrap_or_else(|_| "meshtastic".to_string());
    let meshtastic_url = std::env::var("MESHTASTIC_URL")
        .unwrap_or_else(|_| "192.168.117.176:4403".to_string());

    let client = Client::new(influxdb_url, influxdb_database);
    let stream_api = StreamApi::new();
    let mut readings: Vec<NodeInfoReading> = Vec::new();

    let tcp_stream = utils::stream::build_tcp_stream(meshtastic_url).await?;
    let (mut decoded_listener, stream_api) = stream_api.connect(tcp_stream).await;

    let config_id = utils::generate_rand_id();
    let stream_api = stream_api.configure(config_id).await?;

    loop {
        tokio::select! {
            Some(decoded_packet) = decoded_listener.recv() => {
                // Only process and display NodeInfo packets
                handle_from_radio_packet(decoded_packet, &mut readings)
            }
            _ = sleep(Duration::from_secs(3)) => {
                break;
            }
        }
    }

    let _stream_api = stream_api.disconnect().await?;

    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    push_queries_to_influx(client, readings).await;

    Ok(())
}
