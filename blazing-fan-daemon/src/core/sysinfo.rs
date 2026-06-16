use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

pub struct SystemFetcher {
    sys: System,
    components: Components,
    refresh: RefreshKind,
}

pub struct SysInfo {
    pub cpu_usage: f32,
    pub cpu_tmp: f32,
    pub mem_usage: u64,
}

impl SystemFetcher {
    pub fn new() -> Self {
        let cpu_refresh = CpuRefreshKind::nothing().with_cpu_usage().with_frequency();
        let mem_refresh = MemoryRefreshKind::nothing().with_ram();
        let refresh = RefreshKind::nothing()
            .without_processes()
            .with_cpu(cpu_refresh)
            .with_memory(mem_refresh);

        let sys = System::new_with_specifics(refresh);
        let components = Components::new_with_refreshed_list();

        Self {
            sys,
            components,
            refresh,
        }
    }

    pub fn fetch(&mut self) -> SysInfo {
        self.sys.refresh_specifics(self.refresh);
        self.components.refresh(false);

        let cpu_usage = self.sys.global_cpu_usage();
        let mem_usage = self.sys.used_memory();
        let cpu_tmp = self
            .components
            .iter_mut()
            .find(|c| c.id() == Some("hwmon0_1"))
            .unwrap()
            .temperature()
            .unwrap_or(0.0);

        SysInfo {
            cpu_usage,
            cpu_tmp,
            mem_usage,
        }
    }
}
