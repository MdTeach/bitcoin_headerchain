check:
	CC=clang CFLAGS_riscv32im_risc0_zkvm_elf="-target riscv32-unknown-elf" cargo check

build:
	CC=clang CFLAGS_riscv32im_risc0_zkvm_elf="-target riscv32-unknown-elf" cargo build

run:
	RISC0_DEV_MODE=0 RUST_LOG="[executor]=info" CC=clang CFLAGS_riscv32im_risc0_zkvm_elf="-target riscv32-unknown-elf" cargo run

run-fast:
	RISC0_DEV_MODE=0 RUST_LOG="[executor]=info" CC=clang CFLAGS_riscv32im_risc0_zkvm_elf="-target riscv32-unknown-elf" cargo run --release

trace:
	RISC0_DEV_MODE=1 RUST_LOG="[executor]=info" CC=clang CFLAGS_riscv32im_risc0_zkvm_elf="-target riscv32-unknown-elf" cargo run
