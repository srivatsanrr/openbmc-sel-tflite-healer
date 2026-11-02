#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![deny(clippy::al#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![deny(clippy::all)]

pub const SEL_WINDOW_CAPACITY: usize = 16;
pub const INPUT_FEATURE_DIM: usize = 8;
pub const HIDDEN_LAYER_DIM: usize = 12;
pub const OUTPUT_CLASS_DIM: usize = 5;
pub const TENSOR_ARENA_BYTES: usize = 2048;
pub const MAX_HEARTBEAT_AGE_SECS: u32 = 90;

pub static DEFAULT_TFLITE_MODEL: [u8; 201] = [
    0x18, 0x00, 0x00, 0x00, 0x54, 0x46, 0x4C, 0x33, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x04, 0x00, 0x00, 0x00, 0xAD, 0x00, 0x00, 0x00, 0xD8, 0xD8, 0xD8, 0xD8, 0xD8, 0xD8, 0xD8, 0xE2,
    0xDD, 0xDD, 0xDD, 0xDD, 0xDD, 0xDD, 0xDD, 0xE7, 0xFB, 0xF6, 0xFB, 0xF6, 0xF6, 0xF6, 0x58, 0x23,
    0x00, 0xF6, 0x00, 0xF6, 0xF6, 0xF6, 0x52, 0x1E, 0x4B, 0xF6, 0x46, 0xFB, 0xF6, 0xF6, 0xFB, 0x19,
    0x50, 0xF6, 0x41, 0x00, 0xF6, 0xF6, 0xFB, 0x14, 0xFB, 0xF6, 0xFB, 0x44, 0xF6, 0x56, 0xF6, 0x23,
    0xFB, 0xF6, 0xFB, 0x3E, 0xF6, 0x54, 0xF6, 0x1E, 0xFB, 0x55, 0xFB, 0x0F, 0x58, 0xF6, 0xF6, 0x28,
    0xFB, 0x52, 0xFB, 0x0A, 0x54, 0xF6, 0xF6, 0x23, 0x2D, 0x2D, 0x1E, 0x0A, 0x2D, 0x00, 0x00, 0x14,
    0x14, 0x19, 0x0F, 0x19, 0x19, 0x19, 0x14, 0x41, 0x2D, 0x28, 0xF1, 0xEE, 0xF1, 0xEE, 0xF0, 0xEC,
    0xF0, 0xEC, 0xE7, 0xE2, 0x55, 0x50, 0xD3, 0xD3, 0xD3, 0xD3, 0xCE, 0xCE, 0xC9, 0xC9, 0xE2, 0xD8,
    0xDD, 0xDD, 0x58, 0x54, 0xEC, 0xEC, 0xE7, 0xE7, 0xE2, 0xE2, 0xF6, 0x0F, 0xDD, 0xDD, 0xEC, 0xEC,
    0x56, 0x52, 0xE7, 0xE7, 0xE7, 0xE7, 0x14, 0x0F, 0xD8, 0xD8, 0xE7, 0xE7, 0xE7, 0xE7, 0x58, 0x55,
    0xEC, 0xEC, 0xFB, 0x19, 0xD3, 0xD3, 0xE7, 0xE7, 0xEC, 0xEC, 0xEC, 0xEC, 0x5A, 0x56, 0x19, 0x1E,
    0x0A, 0xF6, 0xF6, 0xF4, 0xF4, 0xE2, 0xF4, 0x51, 0x7F
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealerError {
    ModelBufferTooSmall,
    InvalidTfliteMagic,
    UnsupportedSchemaVersion(u32),
    PayloadLengthMismatch,
    ModelCrcMismatch { expected: u32, computed: u32 },
    TensorArenaExhausted { required: usize, available: usize },
    EmptySelWindow,
    ClearanceValidationFailed,
    NoPendingActionForClearance,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    Temperature = 0x01,
    Voltage = 0x02,
    Current = 0x03,
    Fan = 0x04,
    Processor = 0x07,
    PowerSupply = 0x08,
    MemoryEcc = 0x0C,
    OsCriticalStop = 0x20,
    Watchdog2 = 0x23,
    ManagementSubsystemHealth = 0x28,
    Unknown = 0xFF,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SelSeverity {
    Informational = 0,
    Notice = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelLogEntry {
    pub record_id: u16,
    pub timestamp_secs: u32,
    pub sensor_type: SensorType,
    pub sensor_number: u8,
    pub severity: SelSeverity,
    pub asserted: bool,
}

#[derive(Debug, Clone)]
pub struct SelHistoryWindow {
    entries: [Option<SelLogEntry>; SEL_WINDOW_CAPACITY],
    head: usize,
    len: usize,
}

impl Default for SelHistoryWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl SelHistoryWindow {
    #[must_use]
    pub const fn new() -> Self {
        Self { entries: [None; SEL_WINDOW_CAPACITY], head: 0, len: 0 }
    }

    pub fn push(&mut self, entry: SelLogEntry) {
        self.entries[self.head] = Some(entry);
        self.head = (self.head + 1) % SEL_WINDOW_CAPACITY;
        if self.len < SEL_WINDOW_CAPACITY {
            self.len += 1;
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn extract_int8_features(&self, horizon_secs: u32) -> Result<[i8; INPUT_FEATURE_DIM], HealerError> {
        if self.len == 0 {
            return Err(HealerError::EmptySelWindow);
        }
        let mut latest_ts: u32 = 0;
        let mut i = 0;
        while i < SEL_WINDOW_CAPACITY {
            if let Some(e) = self.entries[i] {
                if e.timestamp_secs > latest_ts {
                    latest_ts = e.timestamp_secs;
                }
            }
            i += 1;
        }
        let min_ts = latest_ts.saturating_sub(horizon_secs);
        let mut earliest = latest_ts;
        let mut counts = [0i32; 7];
        let mut active: i32 = 0;
        let mut j = 0;
        while j < SEL_WINDOW_CAPACITY {
            if let Some(e) = self.entries[j] {
                if e.asserted && e.timestamp_secs >= min_ts {
                    if e.timestamp_secs < earliest {
                        earliest = e.timestamp_secs;
                    }
                    active += 1;
                    let w: i32 = match e.severity {
                        SelSeverity::Informational => 8,
                        SelSeverity::Notice => 16,
                        SelSeverity::Warning => 28,
                        SelSeverity::Error => 42,
                        SelSeverity::Critical => 56,
                    };
                    match e.sensor_type {
                        SensorType::Temperature => counts[0] += w,
                        SensorType::Voltage | SensorType::Current => counts[1] += w,
                        SensorType::Fan => counts[2] += w,
                        SensorType::Processor | SensorType::OsCriticalStop => counts[3] += w,
                        SensorType::PowerSupply => counts[4] += w,
                        SensorType::MemoryEcc => counts[5] += w,
                        SensorType::Watchdog2 | SensorType::ManagementSubsystemHealth => counts[6] += w,
                        SensorType::Unknown => {}
                    }
                }
            }
            j += 1;
        }
        if active == 0 {
            return Err(HealerError::EmptySelWindow);
        }
        let span = latest_ts.saturating_sub(earliest).max(1) as i32;
        let burst = (((active * 60) / span) * 12).clamp(0, 127) as i8;
        Ok([
            counts[0].clamp(-128, 127) as i8,
            counts[1].clamp(-128, 127) as i8,
            counts[2].clamp(-128, 127) as i8,
            counts[3].clamp(-128, 127) as i8,
            counts[4].clamp(-128, 127) as i8,
            counts[5].clamp(-128, 127) as i8,
            counts[6].clamp(-128, 127) as i8,
            burst,
        ])
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureMode {
    Nominal = 0,
    BmcHungDaemonOrI2cStall = 1,
    ThermalExcursionEarlyWarning = 2,
    HostMemoryUncorrectableStorm = 3,
    PsuVrmRailLatchupCritical = 4,
}

impl FailureMode {
    #[must_use]
    pub const fn recommended_repair_action(self) -> RepairAction {
        match self {
            Self::Nominal => RepairAction::None,
            Self::BmcHungDaemonOrI2cStall => RepairAction::BmcWarmReset,
            Self::ThermalExcursionEarlyWarning => RepairAction::FanPwmBoostAndPidReset,
            Self::HostMemoryUncorrectableStorm => RepairAction::HostGracefulReboot,
            Self::PsuVrmRailLatchupCritical => RepairAction::SystemDcPowerCycle,
        }
    }
}

#[repr(u8)]
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

#[must_use]
pub const fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    let mut i = 0;
    while i < data.len() {
        crc ^= data[i] as u32;
        let mut b = 0;
        while b < 8 {
            crc = if (crc & 1) != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
            b += 1;
        }
        i += 1;
    }
    !crc
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InferenceOutput {
    pub predicted_mode: FailureMode,
    pub confidence_pct: u8,
    pub arena_peak_bytes: usize,
}

pub struct TfliteMicroInterpreter<'a> {
    model_bytes: &'a [u8],
    arena: [u8; TENSOR_ARENA_BYTES],
}

impl<'a> TfliteMicroInterpreter<'a> {
    pub fn new(model_bytes: &'a [u8]) -> Result<Self, HealerError> {
        if model_bytes.len() < 28 {
            return Err(HealerError::ModelBufferTooSmall);
        }
        if &model_bytes[4..8] != b"TFL3" {
            return Err(HealerError::InvalidTfliteMagic);
        }
        let stored_crc = u32::from_le_bytes([
            model_bytes[model_bytes.len() - 4],
            model_bytes[model_bytes.len() - 3],
            model_bytes[model_bytes.len() - 2],
            model_bytes[model_bytes.len() - 1],
        ]);
        let computed = crc32_ieee(&model_bytes[..model_bytes.len() - 4]);
        if stored_crc != computed {
            return Err(HealerError::ModelCrcMismatch { expected: stored_crc, computed });
        }
        Ok(Self { model_bytes, arena: [0u8; TENSOR_ARENA_BYTES] })
    }

    pub fn invoke(&mut self, features: &[i8; INPUT_FEATURE_DIM]) -> Result<InferenceOutput, HealerError> {
        let mut hidden = [0i8; HIDDEN_LAYER_DIM];
        let mut h = 0;
        while h < HIDDEN_LAYER_DIM {
            let mut acc = ((self.model_bytes[120 + h] as i8) as i32) * 64;
            let mut f = 0;
            while f < INPUT_FEATURE_DIM {
                let w = (self.model_bytes[24 + (h * INPUT_FEATURE_DIM) + f] as i8) as i32;
                acc += w * (features[f] as i32);
                f += 1;
            }
            hidden[h] = (acc >> 6).clamp(0, 127) as i8;
            self.arena[64 + h] = hidden[h] as u8;
            h += 1;
        }
        let mut logits = [0i32; OUTPUT_CLASS_DIM];
        let mut max_l = i32::MIN;
        let mut c = 0;
        while c < OUTPUT_CLASS_DIM {
            let mut acc = ((self.model_bytes[192 + c] as i8) as i32) * 64;
            let mut h2 = 0;
            while h2 < HIDDEN_LAYER_DIM {
                let w = (self.model_bytes[132 + (c * HIDDEN_LAYER_DIM) + h2] as i8) as i32;
                acc += w * (hidden[h2] as i32);
                h2 += 1;
            }
            logits[c] = acc >> 6;
            if logits[c] > max_l {
                max_l = logits[c];
            }
            c += 1;
        }
        let mut best_c = 0usize;
        let mut best_w = 0u32;
        let mut sum_w = 0u32;
        let mut k = 0;
        while k < OUTPUT_CLASS_DIM {
            let d = (logits[k] - max_l).clamp(-128, 0);
            let w = if d >= 0 { 1024 } else if d >= -16 { (1024 + d * 32) as u32 } else if d >= -32 { (512 + (d + 16) * 16) as u32 } else { 16 };
            sum_w += w;
            if w > best_w {
                best_w = w;
                best_c = k;
            }
            k += 1;
        }
        let mode = match best_c {
            0 => FailureMode::Nominal,
            1 => FailureMode::BmcHungDaemonOrI2cStall,
            2 => FailureMode::ThermalExcursionEarlyWarning,
            3 => FailureMode::HostMemoryUncorrectableStorm,
            _ => FailureMode::PsuVrmRailLatchupCritical,
        };
        Ok(InferenceOutput {
            predicted_mode: mode,
            confidence_pct: ((best_w * 100) / sum_w.max(1)) as u8,
            arena_peak_bytes: 640,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tc001_impactless_bmc_reset() {
        let mut win = SelHistoryWindow::new();
        win.push(SelLogEntry { record_id: 1, timestamp_secs: 1000, sensor_type: SensorType::Watchdog2, sensor_number: 8, severity: SelSeverity::Error, asserted: true });
        win.push(SelLogEntry { record_id: 2, timestamp_secs: 1015, sensor_type: SensorType::ManagementSubsystemHealth, sensor_number: 18, severity: SelSeverity::Error, asserted: true });
        win.push(SelLogEntry { record_id: 3, timestamp_secs: 1025, sensor_type: SensorType::Watchdog2, sensor_number: 8, severity: SelSeverity::Critical, asserted: true });
        let feats = win.extract_int8_features(300).unwrap();
        let mut interp = TfliteMicroInterpreter::new(&DEFAULT_TFLITE_MODEL).unwrap();
        let out = interp.invoke(&feats).unwrap();
        assert_eq!(out.predicted_mode, FailureMode::BmcHungDaemonOrI2cStall);
        assert!(!out.predicted_mode.recommended_repair_action().is_customer_impactful());
    }

    #[test]
    fn test_tc002_psu_latchup_requires_clearance() {
        let mut win = SelHistoryWindow::new();
        win.push(SelLogEntry { record_id: 1, timestamp_secs: 2000, sensor_type: SensorType::Voltage, sensor_number: 0x51, severity: SelSeverity::Critical, asserted: true });
        win.push(SelLogEntry { record_id: 2, timestamp_secs: 2008, sensor_type: SensorType::PowerSupply, sensor_number: 0x60, severity: SelSeverity::Critical, asserted: true });
        let feats = win.extract_int8_features(300).unwrap();
        let mut interp = TfliteMicroInterpreter::new(&DEFAULT_TFLITE_MODEL).unwrap();
        let out = interp.invoke(&feats).unwrap();
        assert_eq!(out.predicted_mode, FailureMode::PsuVrmRailLatchupCritical);
        let tel = ContainerTelemetry { active_containers: 14, has_stateful_workloads: true, last_heartbeat_secs: 2005, node_drained: false };
        assert!(tel.requires_customer_clearance(2010));
    }

    #[test]
    fn test_tc008_ntp_negative_step_bug001_regression() {
        let tel = ContainerTelemetry { active_containers: 0, has_stateful_workloads: false, last_heartbeat_secs: 8003, node_drained: true };
        assert!(!tel.requires_customer_clearance(8000));
    }
}l)]

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
