#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_comparisons)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_must_use)]
use std::collections::HashMap;
use std::f64::consts::TAU;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant,SystemTime,UNIX_EPOCH};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};
use x11rb::rust_connection::RustConnection;

use tao::{
 event::Event,
 event_loop::{ControlFlow, EventLoopBuilder},
 };
use tray_icon::{
 menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},TrayIconBuilder, TrayIconEvent,
 };
enum UserEvent {
 TrayIconEvent(tray_icon::TrayIconEvent),
 MenuEvent(tray_icon::menu::MenuEvent),
 }

static helpText: &str = "'help' '24h' 'all' 'setTime (secs)' 'clear'\n(barGraph) █= 30 minutes\nsetTime=";

fn convertTime(secs: u64, level: u8) -> u64 {
 let mut time = secs;
 if level >= 0 {
  time = time / 60; // minutes
  }
 if level >= 1 {
  time = time / 60; // hours
  }
 if level >= 2 {
  time = time / 24; // days
  }
 if level >= 3 {
  time = time / 365; // leap years
  }
 time
 }

type ThreadKey = &'static str;

struct ThreadManager {
    delays: Arc<Mutex<HashMap<ThreadKey, Duration>>>,
}

impl ThreadManager {
    fn new() -> Self {
        Self {
            delays: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn spawn<F>(&self, key: ThreadKey, func: F, delay: Duration)
    where
        F: Fn() + Send + 'static + Clone,
    {
        let mut delays = self.delays.lock().unwrap();

        if delays.contains_key(key) {
            // Update the delay if thread already exists
            *delays.get_mut(key).unwrap() = delay;
            println!("Updated delay for thread '{}'", key);
            return;
        }

        // Insert new delay
        delays.insert(key, delay);
        let delays_clone = Arc::clone(&self.delays);

        thread::spawn(move || loop {
            func();

            // Get current delay
            let current_delay = {
                let map = delays_clone.lock().unwrap();
                *map.get(key).unwrap()
            };

            thread::sleep(current_delay);
        });
    }
}

fn reopen_in_terminal() {
    use std::process::Command;

    let current_exe = std::env::current_exe().expect("Failed to get current exe");

    eprintln!("Reopening in terminal: {}", current_exe.display());

    let terminals = [
        ("xdg-terminal-exec", vec![]),
        ("gnome-terminal", vec!["--"]),
        ("konsole", vec!["-e"]),
        ("xterm", vec!["-e"]),
        ("lxterminal", vec!["-e"]),
    ];

    for (term, arg) in terminals {
        let result = Command::new(term)
            .args(arg)
            .arg(&current_exe)
            .spawn();

        if result.is_ok() {
            eprintln!("Launched with {}", term);
            return;
        }
    }

    eprintln!("No supported terminal found!");
}

fn get_focused_process() -> Option<(u32, String)> {
 let (conn, screen_num) = RustConnection::connect(None).ok()?;
 let screen = &conn.setup().roots[screen_num];

 // Atoms
 let net_active = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW").ok()?.reply().ok()?.atom;
 let net_wm_pid = conn.intern_atom(false, b"_NET_WM_PID").ok()?.reply().ok()?.atom;
 let wm_class = conn.intern_atom(false, b"WM_CLASS").ok()?.reply().ok()?.atom;

 // Get active window
 let reply = conn
  .get_property(false, screen.root, net_active, AtomEnum::WINDOW, 0, 1)
  .ok()?.reply().ok()?;
 let window = reply.value32()?.next()?;

 // Try PID first
 if let Ok(reply) = conn.get_property(false, window, net_wm_pid, AtomEnum::CARDINAL, 0, 1).unwrap().reply() {
  if let Some(pid) = reply.value32().and_then(|mut v| v.next()) {
   let exe = format!("/proc/{}/comm", pid);
   if let Ok(name) = fs::read_to_string(exe) {
    return Some((pid, name.trim().to_string()));
   }
  }
 }

 // Fallback to WM_CLASS
 if let Ok(reply) = conn.get_property(false, window, wm_class, AtomEnum::STRING, 0, 64).unwrap().reply() {
  if !reply.value.is_empty() {
   let classes: Vec<&str> = reply.value.split(|&b| b == 0).filter_map(|s| std::str::from_utf8(s).ok()).collect();
   if let Some(name) = classes.first() {
    return Some((0, name.to_string())); // pid=0 if unknown
   }
  }
 }

 None
}

fn logTime() {
 let start = Instant::now();
 loop {
  let elapsed = start.elapsed().as_secs();
  let mut file = OpenOptions::new()
   .create(true)
   .append(true)
   .open("save.txt")
   .expect("Could not open save.txt");

  if let Some((pid, name)) = get_focused_process() {
   writeln!(file, "{} {} : {}s", pid, name, elapsed).unwrap();
  } else {
   writeln!(file, "unknown : {}s", elapsed).unwrap();
  }

  file.flush().unwrap();
 }
}

fn barGraph() {
 let values = vec![("Apples", 5), ("Oranges", 8), ("Bananas", 3)];
 let colors = ["\x1b[35m", "\x1b[36m", "\x1b[33m"]; // magenta, cyan, yellow
  let secs = SystemTime::now() .duration_since(UNIX_EPOCH) .expect("Time went backwards").as_secs();

 println!("{:?}", convertTime(secs,3));
 for (i, (label, val)) in values.iter().enumerate() {
  let bar = "█".repeat(*val as usize);
  println!("{}{:>8}: {} ({})\x1b[0m", colors[i % colors.len()], label, bar, val);
  }
 println!("{}={}min","key",20);
 }

fn clear() {
 print!("\x1B[2J\x1B[H");
 io::stdout().flush().unwrap();
 }

fn pieGraph(totalTime: u64){
 println!("Total Time: {}",0);
 let radius: i32 = 4;
 let wedges = 7usize;
 let x_scale: f64 = 2.0;
 for y in -radius..=radius {
  for x in -(radius as f64 * x_scale).round() as i32..=(radius as f64 * x_scale).round() as i32 {
   let dx = (x as f64) / x_scale;
   let dy = y as f64;
   if dx * dx + dy * dy <= (radius as f64) * (radius as f64) {
    let mut angle = dy.atan2(dx); // -PI..PI
    if angle < 0.0 { angle += TAU; }
    let idx = ((angle / TAU) * wedges as f64).floor() as usize % wedges;
    let color_code = match idx % 6 {
     0 => 31, // red
     1 => 33, // yellow
     2 => 32, // green
     3 => 36, // cyan
     4 => 34, // blue
     _ => 35, // magenta
     };
    print!("\x1b[{}m█\x1b[0m", color_code);
    } else {
    print!(" ");
    }
   }
  println!();
  }
 }

fn loadIcon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

fn trayIcon(){
     let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // set a tray event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        proxy.send_event(UserEvent::TrayIconEvent(event));
    }));

    // set a menu event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let tray_menu = Menu::new();
    let open_i = MenuItem::new("Open", true, None);
    let quit_i = MenuItem::new("Quit", true, None);
    tray_menu.append_items(&[
        &open_i,
        &PredefinedMenuItem::separator(),
        &quit_i,
    ]);
    let mut tray_icon = None;

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(tao::event::StartCause::Init) => {
                let icon = loadIcon(std::path::Path::new(path));

                // We create the icon once the event loop is actually running
                // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                tray_icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu.clone()))
                        .with_tooltip("tao - awesome windowing lib")
                        .with_icon(icon)
                        .build()
                        .unwrap(),
                );

                // We have to request a redraw here to have the icon actually show up.
                // Tao only exposes a redraw method on the Window so we use core-foundation directly.
                #[cfg(target_os = "macos")]
                unsafe {
                    use objc2_core_foundation::{CFRunLoopGetMain, CFRunLoopWakeUp};

                    let rl = CFRunLoopGetMain().unwrap();
                    CFRunLoopWakeUp(&rl);
                }
            }

            Event::UserEvent(UserEvent::TrayIconEvent(event)) => {
                println!("{event:?}");
            }

  Event::UserEvent(UserEvent::MenuEvent(event)) => {
   if event.id == open_i.id() {
    reopen_in_terminal();
    std::process::exit(0);
    }
   else if event.id == quit_i.id() {
    tray_icon.take();
    *control_flow = ControlFlow::Exit;
    }else if event.id == open_i.id() {
    use std::process::Command;
    let _ = Command::new("xterm")
    .arg("-e")
    .arg("bash -c 'echo Hello from tray!; exec bash'")
    .spawn();
   }
  }

            _ => {}
        }
    })
 }



fn commands2() {
 thread::spawn(move || {
  let mut input = String::new();
  loop {
   print!("> ");
   io::stdout().flush().unwrap();

   input.clear();
   if io::stdin().read_line(&mut input).is_err() {
    break;
   }

   let cmd = input.trim().to_string();
   let parts: Vec<&str> = cmd.split_whitespace().collect();

   if parts.is_empty() {
    continue;
   }

   match parts[0] {
    "help" => {
     println!("{}",helpText);
     }
    "setTime" => {
     if parts.len() > 1 {
      if let Ok(num) = parts[1].parse::<u64>() {
       println!("Set time to {}", num);
       // here: handle.update(num);
       } else {
       println!("Invalid number: {}", parts[1]);
       }
      } else {
      println!("Usage: setTime <number>");
      }
     }
    "all" => {
     pieGraph(0);
     }
    "24h" => {
     barGraph();
     }
    other => {
     println!("wrong: {}", other);
    }
   }
  }
 });
}

type FuncKey = &'static str;
struct ThreadHandle {
 delay: Arc<Mutex<Duration>>,
}
impl ThreadHandle {
 fn update(&self, secs: u64) {
  *self.delay.lock().unwrap() = Duration::from_secs(secs);
 }
}
fn thread_func<F>(key: FuncKey, func: F, secs: u64) -> ThreadHandle
where
 F: Fn() + Send + 'static + Clone,
{
 //variable.update(40);
 let delay = Arc::new(Mutex::new(Duration::from_secs(secs)));
 let delay_clone = Arc::clone(&delay);

 thread::spawn(move || loop {
  func();
  let d = *delay_clone.lock().unwrap();
  thread::sleep(d);
 });
 ThreadHandle { delay }
}

fn main() {
 let mut totalTime=0;

 println!("> help\n{}", helpText);
 println!("24h");
 let logTimer = thread_func("task2", || {
  logTime();
 }, 2);

 commands2();
 trayIcon();
 }