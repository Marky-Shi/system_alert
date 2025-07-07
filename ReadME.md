# System Alert - Advanced macOS System Monitor

A high-performance, feature-rich system monitoring tool for macOS with special optimizations for Apple Silicon chips. This tool provides real-time system metrics with an intuitive terminal user interface, smart notifications, and comprehensive configuration options.

## 🚀 Key Features & Optimizations

### Performance Improvements
- **Async Architecture**: Non-blocking data collection using Tokio
- **Smart Caching**: PowerMetrics data cached for optimal performance
- **Efficient Memory Management**: Reduced allocations and optimized data structures
- **Selective Refresh**: Only refresh necessary system components
- **Optimized Build**: Release profile with LTO, size optimization, and panic=abort

### Enhanced User Experience
- **Four-Quadrant Layout**: Brand new sectioned layout with clear information grouping
- **Interactive Controls**: Keyboard navigation, notification toggle, manual refresh
- **Smart Notifications**: Configurable thresholds with cooldown periods
- **Visual Power Monitor**: Beautiful bordered display with progress bars
- **Minimal Mode**: Lightweight display for resource-constrained scenarios

### Advanced Features
- **Configuration Management**: TOML-based config files with CLI overrides
- **Apple Silicon Metrics**: E-cluster/P-cluster monitoring with power analysis
- **Temperature Monitoring**: Component temperature tracking with smart status indicators
- **Process Analysis**: Top processes by CPU usage with detailed information
- **Network Statistics**: Real-time network traffic monitoring
- **Real-time Power Statistics**: Dedicated power consumption analysis

## 📋 System Requirements

- **macOS**: 10.15+ (Optimized for Apple Silicon)
- **Root Privileges**: Required for accessing system metrics via `powermetrics`
- **Rust**: 1.70+ (Required for building from source)

## 🛠 Installation

### Install from Source
```bash
git clone https://github.com/yourusername/system-alert.git
cd system-alert
cargo build --release
```

### Quick Start
```bash
# Run with default settings
sudo cargo run

# Run with custom refresh rate
sudo cargo run -- --refresh 2

# Run in minimal mode
sudo cargo run -- --minimal

# Run with custom config
sudo cargo run -- --config custom-config.toml
```

## 🎮 Usage & Controls

### Command Line Options
```bash
sudo ./target/release/system-alert [options]

Options:
  -r, --refresh <seconds>     Set refresh rate (default: 1)
  -m, --minimal              Use minimal display mode
  -c, --config <file>        Specify custom config file
  -h, --help                 Show help message
  -V, --version              Show version information
```

### Interactive Controls
- **q** or **Ctrl+C**: Quit application
- **n**: Toggle notifications
- **r**: Force refresh

### Interface Layout
1. **🔵 CPU Section**: CPU core usage and Apple Silicon power metrics
2. **🔴 Power Monitor**: Real-time power consumption analysis with visual bars
3. **🟢 Memory Monitor**: Detailed memory statistics with usage visualization
4. **🟡 Temperature Monitor**: Component temperatures with smart status indicators
5. **🟣 Process Monitor**: Top processes sorted by CPU usage
6. **🔵 Network Monitor**: Real-time network traffic statistics

## ⚙️ Configuration Options

### Default Configuration
The application will create a `config.toml` file with sensible defaults:

```toml
# System Monitor Configuration
refresh_rate = 1
minimal_mode = false

[thresholds]
cpu_warning = 75.0
cpu_critical = 90.0
memory_warning = 75
memory_critical = 90
temperature_warning = 70.0
temperature_critical = 85.0

[display]
show_temperatures = true
show_network = true
show_processes = true
show_history = true
history_size = 60

[notifications]
enabled = true
cooldown_seconds = 30
```

### Configuration Options

#### Main Settings
- `refresh_rate`: Update interval in seconds
- `minimal_mode`: Enable simplified interface

#### Threshold Settings
- `cpu_warning/critical`: CPU usage alert thresholds (%)
- `memory_warning/critical`: Memory usage alert thresholds (%)
- `temperature_warning/critical`: Temperature alert thresholds (°C)

#### Display Settings
- `show_temperatures`: Enable temperature monitoring
- `show_network`: Enable network interface monitoring
- `show_processes`: Enable process monitoring
- `show_history`: Enable historical data tracking
- `history_size`: Number of data points to retain

#### Notification Settings
- `enabled`: Enable/disable system notifications
- `cooldown_seconds`: Minimum time between notifications

## 🏗 Architecture Overview

Optimized architecture with separation of concerns for better maintainability and performance:

```
src/
├── main.rs              # 带异步事件循环的应用程序入口点
├── cli.rs               # 命令行解析和输入处理
├── config.rs            # 配置管理 (TOML)
├── data_collector.rs    # 异步系统数据收集
├── ui.rs                # 终端用户界面 (TUI)
├── notification.rs      # 智能通知系统
├── history.rs           # 历史数据跟踪
├── types.rs             # 数据结构和类型
└── system_info.rs       # 传统兼容性模块
```

### Key Improvements
1. **Separation of Concerns**: Clear separation between UI, data collection, and business logic
2. **Async/Await**: Non-blocking operations throughout
3. **Error Handling**: Comprehensive error handling without panics
4. **Memory Efficiency**: Reduced allocations and better data structures
5. **Configurability**: Extensive configuration options
6. **Extensibility**: Modular design for easy feature addition

## 🔧 Performance Optimization

### Build Optimization
```toml
[profile.release]
strip = true           # 移除调试符号
opt-level = "z"        # 针对大小优化
lto = true            # 链接时优化
codegen-units = 1     # 单个代码生成单元以获得更好的优化
panic = "abort"       # 更小的二进制大小
```

### Runtime Optimization
- **PowerMetrics Caching**: Reduce expensive system calls
- **Selective Data Refresh**: Only update changed components
- **Efficient String Handling**: Minimize allocations in hot paths
- **Smart Rendering**: Only redraw when necessary
- **Async I/O**: Non-blocking system calls

## 🚨 Troubleshooting

### Common Issues

**Permission Denied**
```bash
# Make sure to run with sudo
sudo cargo run
```

**PowerMetrics Not Found**
```bash
# Verify powermetrics is available (macOS only)
which powermetrics
```

**High CPU Usage**
```bash
# Increase refresh rate to reduce system load
sudo cargo run -- --refresh 5
```

**Configuration Issues**
```bash
# Reset to defaults by removing config file
rm config.toml
sudo cargo run
```

## 📊 Apple Silicon Metrics

For Apple Silicon Macs, this tool provides detailed metrics:

- **E-Cluster**: Efficiency core usage and frequency
- **P-Cluster**: Performance core usage and frequency
- **Power Consumption**: CPU, GPU, ANE (Neural Engine), and total package power
- **Real-time Monitoring**: Live updates of power states

## 🤝 Contributing

Contributions are welcome! Feel free to submit issues, feature requests, or pull requests.

### Development Setup
```bash
git clone https://github.com/yourusername/system-alert.git
cd system-alert
cargo build
cargo test
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [TUI-rs](https://github.com/fdehau/tui-rs) for terminal interface
- Leverages [sysinfo](https://github.com/GuillaumeGomez/sysinfo) for system metrics
- Powered by [Tokio](https://tokio.rs/) for async runtime