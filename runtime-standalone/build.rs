// Copyright 2020-2024 Chameleon Network.
// SPDX-License-Identifier: GPL-3.0

fn main() {
    #[cfg(feature = "std")]
    substrate_wasm_builder::WasmBuilder::new()
        .with_current_project()
        .export_heap_base()
        .import_memory()
        .build();
}
