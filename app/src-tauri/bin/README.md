# WinDivert runtime binaries (bundled at build time)

NetTamer links against the **WinDivert** import library (`#[link(name = "WinDivert")]`)
and at runtime requires the matching driver + DLL to be present next to the
executable:

- `WinDivert.dll`  — user-mode library
- `WinDivert.sys`  — signed kernel driver (loaded via WFP; needs administrator)

Requirements:
- WinDivert **1.4.x** (or newer, matching the FFI signatures in
  `src/windivert/ffi.rs`).
- Must be copied into this directory (or the final binary output dir) as part of
  the release packaging step. Do **not** download them from the network at
  runtime — only ship the official, signed binaries.

The `cargo build` / `cargo check` step only needs the import library
(`WinDivert.lib`); the `.dll`/`.sys` are required only when the app actually
opens a WinDivert handle (i.e. `apply_throttle_policy` / monitoring start).
