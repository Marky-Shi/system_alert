#[derive(Default, Debug)]
pub struct CPUMetrics {
    pub e_cluster_active: i32,
    pub p_cluster_active: i32,
    pub e_cluster_freq_mhz: i32,
    pub p_cluster_freq_mhz: i32,
    pub cpu_w: f64,
    pub gpu_w: f64,
    pub ane_w: f64,
    pub package_w: f64,
}

impl std::fmt::Display for CPUMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "e_cluster_active: {}\n p_cluster_active: {}\n e_cluster_freq_mhz: {}\n p_cluster_freq_mhz: {}\n cpu_w: {}\n gpu_w: {}\n ane_w: {}\n package_w: {}\n",
        self.e_cluster_active,
        self.p_cluster_active,
        self.p_cluster_freq_mhz,
        self.p_cluster_freq_mhz,
        self.cpu_w,
        self.gpu_w,
        self.ane_w,
        self.package_w,
        )
    }
}
