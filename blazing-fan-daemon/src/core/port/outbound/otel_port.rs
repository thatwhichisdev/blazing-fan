pub trait OtelPort {
    fn record_cpu_tmp(&self, value: f64);

    fn record_cpu_load(&self, value: f64);

    fn record_mem_usg(&self, value: u64);
}
