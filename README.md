# meshtastic to influxdb

Retrieve connected nodes data from meshtastic node and load these into influxdb once

## used env variables

| Variable          | Description                                                           |
| ----------------- | --------------------------------------------------------------------- |
| INFLUXDB_URL      | URL of the InfluxDB server (default: http://localhost:8086)           |
| INFLUXDB_DATABASE | Name of the InfluxDB database to use (default: meshtastic)            |
| MESHTASTIC_URL    | URL and port of the Meshtastic device (default: 192.168.117.176:4403) |
| RUST_LOG          | use INFO to enable logging INFO messages                              |

