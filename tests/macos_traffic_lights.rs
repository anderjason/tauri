#[cfg(not(target_os = "macos"))]
fn main() {}

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
  run()
}

#[cfg(target_os = "macos")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
  use std::thread;
  use std::time::{Duration, Instant};

  use objc2_app_kit::{NSView, NSWindow, NSWindowButton};
  use tauri::test::{mock_context, noop_assets};
  use tauri::{
    ActivationPolicy, Builder, LogicalPosition, LogicalSize, Manager, TitleBarStyle,
    WebviewUrl, Wry,
  };

  const REQUESTED_X: f64 = 120.0;
  const REQUESTED_Y: f64 = 40.0;
  const TOLERANCE: f64 = 0.5;

  #[derive(Clone, Copy, Debug)]
  struct TrafficLightPositions {
    close_x: f64,
    mini_x: f64,
    zoom_x: f64,
    close_top_inset: f64,
    mini_top_inset: f64,
    zoom_top_inset: f64,
  }

  #[allow(deprecated)]
  fn pump_app(app: &mut tauri::App<Wry>) {
    app.run_iteration(|_, _| {});
  }

  #[allow(deprecated)]
  fn wait_until(
    app: &mut tauri::App<Wry>,
    timeout: Duration,
    mut predicate: impl FnMut() -> Result<bool, Box<dyn std::error::Error>>,
  ) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = Instant::now() + timeout;
    loop {
      app.run_iteration(|_, _| {});
      if predicate()? {
        return Ok(());
      }
      if Instant::now() >= deadline {
        return Err("timed out waiting for window state".into());
      }
      thread::sleep(Duration::from_millis(16));
    }
  }

  fn approx_eq(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() <= TOLERANCE
  }

  fn traffic_light_positions(
    window: &tauri::WebviewWindow<Wry>,
  ) -> Result<TrafficLightPositions, Box<dyn std::error::Error>> {
    unsafe {
      let ns_window_ptr = window.ns_window()?.cast::<NSWindow>();
      let ns_window = ns_window_ptr
        .as_ref()
        .ok_or("expected NSWindow pointer to be non-null")?;

      let close = ns_window
        .standardWindowButton(NSWindowButton::CloseButton)
        .ok_or("missing close button")?;
      let mini = ns_window
        .standardWindowButton(NSWindowButton::MiniaturizeButton)
        .ok_or("missing miniaturize button")?;
      let zoom = ns_window
        .standardWindowButton(NSWindowButton::ZoomButton)
        .ok_or("missing zoom button")?;
      let titlebar = close
        .superview()
        .and_then(|view| view.superview())
        .ok_or("missing titlebar container")?;
      let titlebar_height = NSView::frame(&titlebar).size.height;
      let close_frame = NSView::frame(&close);
      let mini_frame = NSView::frame(&mini);
      let zoom_frame = NSView::frame(&zoom);

      Ok(TrafficLightPositions {
        close_x: close_frame.origin.x,
        mini_x: mini_frame.origin.x,
        zoom_x: zoom_frame.origin.x,
        close_top_inset: titlebar_height - close_frame.origin.y - close_frame.size.height,
        mini_top_inset: titlebar_height - mini_frame.origin.y - mini_frame.size.height,
        zoom_top_inset: titlebar_height - zoom_frame.origin.y - zoom_frame.size.height,
      })
    }
  }

  let mut app = Builder::<Wry>::new()
    .enable_macos_default_menu(false)
    .build(mock_context::<Wry, _>(noop_assets()))?;
  app.set_activation_policy(ActivationPolicy::Regular);

  let window = tauri::WebviewWindowBuilder::new(&app, "traffic-lights", WebviewUrl::default())
    .title_bar_style(TitleBarStyle::Overlay)
    .hidden_title(true)
    .decorations(true)
    .visible(false)
    .inner_size(720.0, 520.0)
    .traffic_light_position(LogicalPosition::new(REQUESTED_X, REQUESTED_Y))
    .build()?;

  pump_app(&mut app);
  window.show()?;
  pump_app(&mut app);
  window.set_focus()?;

  wait_until(&mut app, Duration::from_secs(5), || {
    let positions = traffic_light_positions(&window)?;
    Ok(approx_eq(positions.close_x, REQUESTED_X))
  })?;

  let positions = traffic_light_positions(&window)?;
  assert!(
    approx_eq(positions.close_x, REQUESTED_X),
    "expected close button x ~= {REQUESTED_X}, got {:?}",
    positions
  );
  assert!(
    approx_eq(positions.mini_x, REQUESTED_X + 23.0),
    "expected miniaturize button x ~= {}, got {:?}",
    REQUESTED_X + 23.0,
    positions
  );
  assert!(
    approx_eq(positions.zoom_x, REQUESTED_X + 46.0),
    "expected zoom button x ~= {}, got {:?}",
    REQUESTED_X + 46.0,
    positions
  );
  assert!(
    approx_eq(positions.close_top_inset, REQUESTED_Y),
    "expected close button top inset ~= {REQUESTED_Y}, got {:?}",
    positions
  );
  assert!(
    approx_eq(positions.mini_top_inset, REQUESTED_Y),
    "expected miniaturize button top inset ~= {REQUESTED_Y}, got {:?}",
    positions
  );
  assert!(
    approx_eq(positions.zoom_top_inset, REQUESTED_Y),
    "expected zoom button top inset ~= {REQUESTED_Y}, got {:?}",
    positions
  );

  let initial_size = window.inner_size()?;
  thread::sleep(Duration::from_secs(1));
  window.set_size(LogicalSize::new(1260.0, 840.0))?;

  wait_until(&mut app, Duration::from_secs(5), || {
    let size = window.inner_size()?;
    let positions = traffic_light_positions(&window)?;
    Ok(size != initial_size && approx_eq(positions.close_x, REQUESTED_X))
  })?;

  thread::sleep(Duration::from_secs(1));

  let resized_positions = traffic_light_positions(&window)?;
  assert!(
    approx_eq(resized_positions.close_x, REQUESTED_X),
    "expected close button x to stay ~= {REQUESTED_X} after resize, got {:?}",
    resized_positions
  );
  assert!(
    approx_eq(resized_positions.mini_x, REQUESTED_X + 23.0),
    "expected miniaturize button x to stay ~= {} after resize, got {:?}",
    REQUESTED_X + 23.0,
    resized_positions
  );
  assert!(
    approx_eq(resized_positions.zoom_x, REQUESTED_X + 46.0),
    "expected zoom button x to stay ~= {} after resize, got {:?}",
    REQUESTED_X + 46.0,
    resized_positions
  );
  assert!(
    approx_eq(resized_positions.close_top_inset, REQUESTED_Y),
    "expected close button top inset to stay ~= {REQUESTED_Y} after resize, got {:?}",
    resized_positions
  );
  assert!(
    approx_eq(resized_positions.mini_top_inset, REQUESTED_Y),
    "expected miniaturize button top inset to stay ~= {REQUESTED_Y} after resize, got {:?}",
    resized_positions
  );
  assert!(
    approx_eq(resized_positions.zoom_top_inset, REQUESTED_Y),
    "expected zoom button top inset to stay ~= {REQUESTED_Y} after resize, got {:?}",
    resized_positions
  );

  window.close()?;
  let close_deadline = Instant::now() + Duration::from_secs(5);
  loop {
    pump_app(&mut app);
    if app.webview_windows().is_empty() {
      break;
    }
    if Instant::now() >= close_deadline {
      return Err("timed out waiting for window close".into());
    }
    thread::sleep(Duration::from_millis(16));
  }
  app.cleanup_before_exit();

  Ok(())
}
