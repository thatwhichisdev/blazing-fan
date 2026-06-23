use sysinfo::{
    Components, CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, Networks,
    ProcessRefreshKind, RefreshKind, System,
};

pub struct SystemFetcher {
    system: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    refresh: RefreshKind,
}

#[derive(Default, Clone)]
pub struct SystemInformation {
    pub hostname: String,
}

#[derive(Default, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub cpu_tmp: f32,
    pub mem_usage: u64,
    pub swap_usage: u64,
}

impl SystemFetcher {
    pub fn new() -> Self {
        let processes_refresh = ProcessRefreshKind::nothing().with_tasks();
        let cpu_refresh = CpuRefreshKind::nothing().with_cpu_usage().with_frequency();
        let mem_refresh = MemoryRefreshKind::nothing().with_ram().with_swap();
        let refresh = RefreshKind::nothing()
            .with_processes(processes_refresh)
            .with_cpu(cpu_refresh)
            .with_memory(mem_refresh);

        let system = System::new_with_specifics(refresh);
        let components = Components::new_with_refreshed_list();

        let disk_refreshes = DiskRefreshKind::nothing().with_storage().with_io_usage();
        let disks = Disks::new_with_refreshed_list_specifics(disk_refreshes);

        let networks = Networks::new_with_refreshed_list();

        Self {
            system,
            disks,
            networks,
            components,
            refresh,
        }
    }

    pub fn fetch_info(&mut self) -> SystemInformation {
        let hostname = System::host_name().expect("hostname is not present on the system");

        SystemInformation { hostname }
    }

    pub fn fetch_metrics(&mut self) -> SystemMetrics {
        self.system.refresh_specifics(self.refresh);
        self.disks.refresh(false);
        self.components.refresh(false);

        let cpu_usage = self.system.global_cpu_usage();
        let cpu_tmp = self
            .components
            .iter_mut()
            .find(|c| c.id() == Some("hwmon0_1"))
            .unwrap()
            .temperature()
            .unwrap_or(0.0);

        let mem_usage = self.system.used_memory();
        let swap_usage = self.system.used_swap();

        for disk in &self.disks {
            tracing::info!(
                "{:?} - disk, {:?} - kind, {:?} - total space, {:?} - available space",
                disk.name(),
                disk.kind(),
                disk.total_space(),
                disk.available_space()
            );
        }

        for (interface, data) in &self.networks {
            tracing::info!(
                "{:?} - interface, {:?} - received, {:?} - transmitted",
                interface,
                data.received(),
                data.transmitted()
            );
        }

        for component in &self.components {
            tracing::info!(
                "{:?} - id, {:?} - label, {:?} - temperature",
                component.id(),
                component.label(),
                component.temperature()
            );
        }

        SystemMetrics {
            cpu_usage,
            cpu_tmp,
            mem_usage,
            swap_usage,
        }
    }
}
