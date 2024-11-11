use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;

#[derive(InfluxDbWriteable, Clone)]
pub struct NodeInfoReading {
    pub time: DateTime<Utc>,
    #[influxdb(tag)]
    pub node_id: String,
    #[influxdb(tag)] 
    pub long_name: String,
    #[influxdb(tag)]
    pub short_name: String,
    pub snr: f32,
    pub battery_level: u32,
    pub voltage: f32,
    pub channel_utilization: f32,
    pub air_util_tx: f32,
    pub latitude: i32,
    pub longitude: i32,
    pub altitude: i32,
    #[influxdb(tag)]
    pub via_mqtt: bool,
    #[influxdb(tag)]
    pub hops_away: u32
}
