use crate::core::sysinfo::SysInfo;

pub trait OtelPort {
    fn record_sys_info(&self, sys_info: &SysInfo);
}
