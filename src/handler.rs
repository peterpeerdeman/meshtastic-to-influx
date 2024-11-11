use chrono::{DateTime, Utc};
use crate::models::NodeInfoReading;

pub fn handle_nodeinfo_packet(node_info: meshtastic::protobufs::NodeInfo, readings: &mut Vec<NodeInfoReading>) {
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

    // Extract position data if available
    let (latitude, longitude, altitude) = match &node_info.position {
        Some(pos) => (
            pos.latitude_i,
            pos.longitude_i,
            pos.altitude
        ),
        None => (0, 0, 0)
    };

    let reading = NodeInfoReading {
        time: DateTime::<Utc>::from_timestamp(node_info.last_heard as i64, 0).unwrap(),
        node_id,
        long_name,
        short_name,
        snr: node_info.snr,
        battery_level,
        voltage,
        channel_utilization: channel_util,
        air_util_tx: air_util,
        latitude,
        longitude,
        altitude,
        via_mqtt: node_info.via_mqtt,
        hops_away: node_info.hops_away
    };
    readings.push(reading.clone());
}
