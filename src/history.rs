use std::collections::VecDeque;

const MAX_HISTORY_SIZE: usize = 60; // 保存60个数据点

pub struct HistoryData {
    pub cpu_history: VecDeque<f32>,
    pub memory_history: VecDeque<u16>,
    pub network_rx_history: VecDeque<u64>,
    pub network_tx_history: VecDeque<u64>,
}

impl HistoryData {
    pub fn new() -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            memory_history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            network_rx_history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            network_tx_history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
        }
    }
    
    pub fn add_cpu_usage(&mut self, usage: f32) {
        if self.cpu_history.len() >= MAX_HISTORY_SIZE {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(usage);
    }
    
    pub fn add_memory_usage(&mut self, usage: u16) {
        if self.memory_history.len() >= MAX_HISTORY_SIZE {
            self.memory_history.pop_front();
        }
        self.memory_history.push_back(usage);
    }
    
    pub fn add_network_rx(&mut self, bytes: u64) {
        if self.network_rx_history.len() >= MAX_HISTORY_SIZE {
            self.network_rx_history.pop_front();
        }
        self.network_rx_history.push_back(bytes);
    }
    
    pub fn add_network_tx(&mut self, bytes: u64) {
        if self.network_tx_history.len() >= MAX_HISTORY_SIZE {
            self.network_tx_history.pop_front();
        }
        self.network_tx_history.push_back(bytes);
    }
}