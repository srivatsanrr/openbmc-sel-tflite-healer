#!/usr/bin/env bash
# Aspeed AST2700 OpenBMC Reference Platform QEMU Launch & SEL Injection Harness
set -euo pipefail

MODE="${1:-auto}"
IMGDIR="${IMGDIR:-./ast2700-artifacts}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

if [ "${MODE}" = "ast2700-evb" ] || { [ "${MODE}" = "auto" ] && command -v qemu-system-aarch64 >/dev/null 2>&1 && [ -f "${IMGDIR}/image-bmc" ]; }; then
    exec qemu-system-aarch64 \
        -M ast2700-evb -smp 4 -m 2G \
        -device loader,force-raw=on,addr=0x400000000,file="${IMGDIR}/u-boot.bin" \
        -device loader,force-raw=on,addr=0x430000000,file="${IMGDIR}/bl31.bin" \
        -device loader,force-raw=on,addr=0x430080000,file="${IMGDIR}/optee/tee-raw.bin" \
        -device loader,cpu-num=0,addr=0x430000000 \
        -device loader,cpu-num=1,addr=0x430000000 \
        -device loader,cpu-num=2,addr=0x430000000 \
        -device loader,cpu-num=3,addr=0x430000000 \
        -drive file="${IMGDIR}/image-bmc",format=raw,if=mtd \
        -nographic -serial mon:stdio
else
    echo "[AST2700-QEMU] Compiling and running standalone AST2700 MMIO & INT8 TFLite behavioral simulator..."
    cc -O2 -std=c11 "${REPO_ROOT}/qa/ast2700_healer_sim.c" -o "${REPO_ROOT}/qa/ast2700_healer_sim"
    exec "${REPO_ROOT}/qa/ast2700_healer_sim"
fi
