# OpenBMC AST2700 Embedded Rust TFLite SEL Self-Healing Engine

`openbmc-sel-tflite-healer` is a `no_std` Embedded Rust TFLite INT8 SEL failure correlation and container-aware OpenBMC self-healing engine for the Aspeed AST2700 platform.

## Architecture Overview

- **SelHistoryWindow**: Manages rolling System Event Log (SEL) history windows for feature extraction.
- **TfliteMicroInterpreter**: Executes quantized INT8 TFLite Micro models within a 2048-byte static SRAM arena for real-time failure correlation.
- **SelfHealingCoordinator**: Container-aware clearance policy gate ensuring safe automated remediation and recovery actions.
