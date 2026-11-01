#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![deny(clippy::all)]

pub const SEL_WINDOW_CAPACITY: usize = 16;
pub const INPUT_FEATURE_DIM: usize = 8;
pub const HIDDEN_LAYER_DIM: usize = 12;
pub const OUTPUT_CLASS_DIM: usize = 5;
pub const TENSOR_ARENA_BYTES: usize = 2048;
pub const MAX_HEARTBEAT_AGE_SECS: u32 = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureMode {
    Nominal = 0,
    BmcHungDaemonOrI2cStall = 1,
    ThermalExcursionEarlyWarning = 2,
    HostMemoryUncorrectableStorm = 3,
    PsuVrmRailLatchupCritical = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairAction {
    None = 0,
    BmcWarmReset = 1,
    FanPwmBoostAndPidReset = 2,
    HostGracefulReboot = 3,
    SystemDcPowerCycle = 4,
}

impl RepairAction {
    #[must_use]
    pub const fn is_customer_impactful(self) -> bool {
        matches!(self, Self::HostGracefulReboot | Self::SystemDcPowerCycle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerTelemetry {
    pub active_containers: u16,
    pub has_stateful_workloads: bool,
    pub last_heartbeat_secs: u32,
    pub node_drained: bool,
}

impl ContainerTelemetry {
    #[must_use]
    pub const fn requires_customer_clearance(&self, current_bmc_time_secs: u32) -> bool {
        let age = current_bmc_time_secs.saturating_sub(self.last_heartbeat_secs);
        if age > MAX_HEARTBEAT_AGE_SECS {
            return true;
        }
        if self.node_drained && self.active_containers == 0 {
            return false;
        }
        self.active_containers > 0 || self.has_stateful_workloads
    }
}
