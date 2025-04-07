# System Alert

A comprehensive system monitoring tool for MacOS, with special optimizations for Apple Silicon chips.

## Features

- **System Information**: Display OS details, kernel version, hostname, and CPU architecture
- **CPU Monitoring**: Real-time CPU usage tracking and power metrics for Apple Silicon (E-cluster and P-cluster)
- **Memory Monitoring**: Track memory usage with visual gauge representation
- **Network Monitoring**: Monitor network interface traffic
- **Temperature Monitoring**: Display component temperatures
- **Disk & Process Monitoring**: Track disk usage and process information
- **Notification System**: Get alerts when system resources exceed thresholds
- **Configurable Settings**: Customize refresh rates and display options
- **Internationalization**: Support for multiple languages

## Requirements

- MacOS operating system
- Root privileges (for accessing certain system metrics)
- For Apple Silicon metrics: Apple M-series chip

## Installation

Clone the repository and build the project:

```bash
git clone https://github.com/yourusername/system-alert.git
cd system-alert
cargo build --release
```

Useage
```bash
sudo cargo run
```

- -r, --refresh <SECONDS> : Set the refresh rate in seconds (default: 1)
- -m, --minimal : Use minimal display mode
- -c, --config <FILE> : Specify a custom configuration file