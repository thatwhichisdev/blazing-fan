use sysinfo::{
    Components, CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, Networks,
    ProcessRefreshKind, RefreshKind, System,
};

use crate::core::config::SystemConfig;

static TEMP_SENSOR: &'static str = "hwmon0_1";

pub struct SystemFetcher {
    system: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    refresh: RefreshKind,
    config: SystemConfig,
}

#[derive(Default, Clone)]
pub struct SystemInformation {
    pub hostname: String,
}

#[derive(Default, Clone)]
pub struct CpuTelemetry {
    pub usage: f32,
    pub temp: f32,
}

#[derive(Default, Clone)]
pub struct MemoryTelemetry {
    pub ram_usage: u64,
    pub swap_usage: u64,
}

#[derive(Default, Clone)]
pub struct DiskTelemetry {
    pub name: String,
    pub total: u64,
    pub available: u64,
}

#[derive(Default, Clone)]
pub struct NetworkInterfaceTelemetry {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
}

#[derive(Default, Clone)]
pub struct BladeTelemetry {
    pub cpu: CpuTelemetry,
    pub memory: MemoryTelemetry,
    pub disks: Vec<DiskTelemetry>,
    pub networks: Vec<NetworkInterfaceTelemetry>,
}

impl SystemFetcher {
    pub fn new(config: &SystemConfig) -> Self {
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
            config: config.clone(),
        }
    }

    pub fn fetch_info(&mut self) -> SystemInformation {
        let hostname = System::host_name().expect("hostname is not present on the system");

        SystemInformation { hostname }
    }

    pub fn fetch_metrics(&mut self) -> BladeTelemetry {
        self.system.refresh_specifics(self.refresh);
        self.disks.refresh(false);
        self.networks.refresh(false);
        self.components.refresh(false);

        let cpu_usage = self.system.global_cpu_usage();
        let cpu_tmp = self
            .components
            .iter_mut()
            .find(|c| c.id() == Some(TEMP_SENSOR))
            .unwrap()
            .temperature()
            .unwrap_or(0.0);

        tracing::info!("cpu temp {}", cpu_tmp);

        let cpu_telemetry = CpuTelemetry {
            usage: cpu_usage,
            temp: cpu_tmp,
        };

        let ram_usage = self.system.used_memory();
        let swap_usage = self.system.used_swap();
        let memory_telemetry = MemoryTelemetry {
            ram_usage,
            swap_usage,
        };

        let mut disks_telemetry = vec![];
        for disk in &self.disks {
            tracing::info!(
                "disk {} with total {} and available {}",
                disk.name().to_os_string().into_string().unwrap(),
                disk.total_space(),
                disk.available_space()
            );

            // todo: is there a better way to map &OsStr into String?
            let disk_name = disk.name().to_os_string().into_string().unwrap();
            if self.config.disks.contains(&disk_name) {
                disks_telemetry.push(DiskTelemetry {
                    name: disk_name,
                    total: disk.total_space(),
                    available: disk.available_space(),
                });
            }
        }

        let mut networks_telemetry = vec![];
        for (interface, data) in &self.networks {
            if self.config.networks.contains(interface) {
                networks_telemetry.push(NetworkInterfaceTelemetry {
                    name: interface.clone(),
                    received: data.received(),
                    transmitted: data.transmitted(),
                });
            }
        }

        BladeTelemetry {
            cpu: cpu_telemetry,
            memory: memory_telemetry,
            disks: disks_telemetry,
            networks: networks_telemetry,
        }
    }
}
