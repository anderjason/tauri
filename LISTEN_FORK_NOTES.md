# Listen Fork Notes

This fork keeps macOS traffic-light positioning alive for `WebviewWindow` by
propagating `traffic_light_position(...)` to both the host window builder and
the webview builder.

Key behavior:

- the host Tao window path receives the inset
- the Wry webview attributes still receive the same inset
- the regression test locks the expected macOS button geometry before and after
  resize

When rebasing, inspect:

- `src/webview/webview_window.rs`
- `src/window/mod.rs`
- `tests/macos_traffic_lights.rs`

Validation:

- run `cargo test --features test --test macos_traffic_lights`
- rebuild `listen-app`
