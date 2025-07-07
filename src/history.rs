use crate::types::SystemData;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct HistoryData {
    pub cpu_history: VecDeque<f32>,
    pub memory_history: VecDeque<u16>,
    pub network_rx_history: VecDeque<u64>,
    pub network_tx_history: VecDeque<u64>,
    pub temperature_history: VecDeque<f32>,
    max_size: usize,
}

impl HistoryData {
    pub fn new(max_size: usize) -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(max_size),
            memory_history: VecDeque::with_capacity(max_size),
            network_rx_history: VecDeque::with_capacity(max_size),
            network_tx_history: VecDeque::with_capacity(max_size),
            temperature_history: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn update_from_system_data(&mut self, data: &SystemData) {
        self.add_cpu_usage(data.cpu_info.average_usage);
        self.add_memory_usage(data.memory_info.usage_percentage);
        
        // Sum up all network interfaces
        let total_rx = data.network_info.iter().map(|ni| ni.bytes_received).sum();
        let total_tx = data.network_info.iter().map(|ni| ni.bytes_transmitted).sum();
        self.add_network_rx(total_rx);
        self.add_network_tx(total_tx);

        // Average temperature
        if !data.temperature_info.is_empty() {
            let avg_temp = data.temperature_info.iter()
                .map(|t| t.temperature)
                .sum::<f32>() / data.temperature_info.len() as f32;
            self.add_temperature(avg_temp);
        }
    }
    
    pub fn add_cpu_usage(&mut self, usage: f32) {
        if self.cpu_history.len() >= self.max_size {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(usage);
    }
    
    pub fn add_memory_usage(&mut self, usage: u16) {
        if self.memory_history.len() >= self.max_size {
            self.memory_history.pop_front();
        }
        self.memory_history.push_back(usage);
    }
    
    pub fn add_network_rx(&mut self, bytes: u64) {
        if self.network_rx_history.len() >= self.max_size {
            self.network_rx_history.pop_front();
        }
        self.network_rx_history.push_back(bytes);
    }
    
    pub fn add_network_tx(&mut self, bytes: u64) {
        if self.network_tx_history.len() >= self.max_size {
            self.network_tx_history.pop_front();
        }
        self.network_tx_history.push_back(bytes);
    }

    pub fn add_temperature(&mut self, temp: f32) {
        if self.temperature_history.len() >= self.max_size {
            self.temperature_history.pop_front();
        }
        self.temperature_history.push_back(temp);
    }

    #[allow(dead_code)]
    fn add_to_deque<T>(&mut self, deque: &mut VecDeque<T>, value: T) {
        if deque.len() >= self.max_size {
            deque.pop_front();
        }
        deque.push_back(value);
    }

    pub fn get_cpu_trend(&self) -> Option<f32> {
        if self.cpu_history.len() < 2 {
            return None;
        }
        let recent = self.cpu_history.iter().rev().take(5).sum::<f32>() / 5.0;
        let older = self.cpu_history.iter().take(5).sum::<f32>() / 5.0;
        Some(recent - older)
    }

    pub fn get_memory_trend(&self) -> Option<f32> {
        if self.memory_history.len() < 2 {
            return None;
        }
        let recent = self.memory_history.iter().rev().take(5).sum::<u16>() as f32 / 5.0;
        let older = self.memory_history.iter().take(5).sum::<u16>() as f32 / 5.0;
        Some(recent - older)
    }
}