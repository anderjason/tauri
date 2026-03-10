# Reposition Traffic Lights

This branch carries the macOS traffic-light repositioning fix on top of
upstream `dev`.

The intended behavior is:

- `WebviewWindowBuilder::traffic_light_position(...)` updates both the host
  window builder and the webview builder
- macOS regression coverage verifies the button positions before and after
  resize

Primary patch sites:

- `crates/tauri/src/webview/webview_window.rs`
- `crates/tauri/src/window/mod.rs`
- `crates/tauri/tests/macos_traffic_lights.rs`
