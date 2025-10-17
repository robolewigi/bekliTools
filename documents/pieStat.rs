/*
[package]
name = "pieStatistic"
version = "2.0.0"
edition = "2024"

[dependencies]
lazy_static = "1.4"
image = "0.25.8"
tao = "0.34.3"
tray-icon = "0.21.1"
x11rb = "0.13.2"
sysinfo = "0.30" 
termion = "4.0.5"
libc = "0.2.177"
*/
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::collections::{HashMap};
use std::collections::hash_map::DefaultHasher;
use std::f64::consts::TAU;
use std::fs::{self, OpenOptions, read_to_string};
use std::io::{self, Write, BufWriter};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;
use std::process::{Command};
use std::sync::Mutex;
use std::hash::{Hash, Hasher};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};
use x11rb::rust_connection::RustConnection;
use sysinfo::{System, Pid, get_current_pid};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use tray_icon::TrayIconBuilder;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use lazy_static::lazy_static;
enum UserEvent {
 MenuEvent(tray_icon::menu::MenuEvent),
}

static timer: AtomicU64 = AtomicU64::new(55);
static showPerformance: AtomicBool = AtomicBool::new(false);
static helpText: &str = "'help' '24h' 'all' 'setTime (secs)' 'clear' 'performance' 'background' 'rename (name);(name)' 'remove (name)'\n(barGraph) █= 30 minutes\nsetTimer=";
static isHeadless: AtomicBool = AtomicBool::new(false);

lazy_static! {
 static ref renames: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
 static ref removes: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

fn performance() {
 if !showPerformance.load(Ordering::Relaxed){
  showPerformance.store(true,Ordering::Relaxed);
  thread::spawn(move || {
   let mut sys = System::new_all();
   let pid: Pid = get_current_pid().unwrap();
   while showPerformance.load(Ordering::Relaxed) {
    sys.refresh_process(pid);
    if let Some(process) = sys.process(pid) {
     let mem_bytes = process.memory();
     let mem_mb = mem_bytes as f64 / 1024.0 / 1024.0;
     println!(
      "This program -> CPU: {:.2}% | Memory: {:.2} MB",
      process.cpu_usage(),mem_mb);
     }
     let t = timer.load(Ordering::Relaxed);
     let sleep_time = if isHeadless.load(Ordering::Relaxed) {
     (t / 10).max(15)
    } else {
    (t / 15).max(1)
   };
    thread::sleep(Duration::from_secs(sleep_time));
    }
   });
  } else {
  showPerformance.store(false, Ordering::Relaxed);
  }
 }

fn reopenTerminal() {
 let currentExe = std::env::current_exe().expect("failed current exe");
 let cmd = format!(
  "env DISPLAY=$DISPLAY DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS \
  bash -c '({{ command -v xdg-terminal-exec && xdg-terminal-exec \"{path}\"; }} || \
  {{ command -v gnome-terminal && gnome-terminal -- \"{path}\"; }} || \
  {{ command -v xterm && xterm -e \"{path}\"; }}) &'",
  path = currentExe.display()
 );
 let result = Command::new("bash"). arg("-c"). arg(&cmd). spawn();
 match result {
  Ok(_) => eprintln!(""),
  Err(e) => eprintln!("cantFindTerminal ({})", e),
 }
}

fn getFocused() -> Option<(u32, String, String)> {
 let (conn, screen_num) = RustConnection::connect(None).ok()?;
 let screen = &conn.setup().roots[screen_num];
 let netActive = conn.intern_atom(
  false,b"_NET_ACTIVE_WINDOW") 
  .ok()?.reply().ok()?.atom;

 let netPid = conn.intern_atom(false, b"_NET_WM_PID").ok()?.reply().ok()?.atom;
 let wmClass = conn.intern_atom(false, b"WM_CLASS").ok()?.reply().ok()?.atom;
 let netName = conn.intern_atom(false, b"_NET_WM_NAME").ok()?.reply().ok()?.atom;
 let utf8String = conn.intern_atom(false, b"UTF8_STRING").ok()?.reply().ok()?.atom;

 let reply= conn.get_property(false, screen.root,  
  netActive, AtomEnum::WINDOW, 0, 1).ok()?.reply().ok()?;
 let window = reply.value32()?.next()?;

 let mut pid: u32 = 0;
 if let Ok(reply) = conn.get_property(false, window, netPid, AtomEnum::CARDINAL, 0, 1).unwrap().reply() {
  if let Some(p) = reply.value32().and_then(|mut v| v.next()) {
   pid = p;
   }
  }
 let processName = if pid > 0 {
  let exe = format!("/proc/{}/comm", pid);
  fs::read_to_string(exe).unwrap_or_else(|_| "unknown".to_string()).trim().to_string()
  } else {
  if let Ok(reply) = conn.get_property(false, window, wmClass, AtomEnum::STRING, 0, 64).unwrap().reply() {
   if !reply.value.is_empty() {
    let classes: Vec<&str> = reply.value.split
     (|&b| b == 0).filter_map
     (|s| std::str::from_utf8(s).ok()).collect();
    classes.first().unwrap_or(&"unknown").to_string()
    } else {
    "unknown".to_string()
    }
   } else {
   "unknown".to_string()
   }
  };
 let windowTitle = if let Ok(reply) =
  conn.get_property(false, window, netName, utf8String, 0, u32::MAX).unwrap().reply(){
  if !reply.value.is_empty() {
   String::from_utf8(reply.value).unwrap_or_else(|_| "unknown".to_string())
   } else {
   "unknown".to_string()
   }
  } else {
  "unknown".to_string()
  };
 Some((pid, processName, windowTitle))
 }

fn logTime() {
 thread::spawn(move || {
  loop {
   let mut allTotals: HashMap<String, u64> = HashMap::new();
   let mut history: Vec<String> = Vec::new();

   if let Ok(content) = read_to_string("log.txt") {
    let mut section = "";
    for line in content.lines() {
     let trimmed = line.trim();
     if trimmed.is_empty() { continue; }
     if trimmed.starts_with('[') {
      section = trimmed;
      continue;
     }

     match section {
      "[all]" => {
       let mut parts = trimmed.split_whitespace();
       if let (Some(name), Some(secs_str)) = (parts.next(), parts.next()) {
        if let Ok(secs) = secs_str.parse::<u64>() {
         allTotals.entry(name.to_string()).or_insert(secs);
        }
       }
      }
      "[24h]" => history.push(trimmed.to_string()),
      _ => {}
     }
    }
   }

   if let Some((_pid, processName, window_title)) = getFocused() {
    let addSeconds = timer.load(Ordering::Relaxed);
    *allTotals.entry(processName.clone()).or_insert(0) += addSeconds;

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    let mins = (secs / 60) % 60;
    let hours = (secs / 3600) % 24;
    let days = secs / 86400;
    history.push(format!(
     "day{} {:02}:{:02} {} [{}] {}",days, hours, mins, processName, window_title, addSeconds));
   }

   let nowSecs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
   let cutoff = nowSecs.saturating_sub(86_400);
   let mut parsedEntries: Vec<(u64, String)> = Vec::new();
   for line in &history {
    if let Some(day_str) = line.split_whitespace().next().and_then(|x| x.strip_prefix("day")) {
     if let Ok(dayNum) = day_str.parse::<u64>() {
      let hh = line[4..6].parse::<u64>().unwrap_or(0);
      let mm = line[7..9].parse::<u64>().unwrap_or(0);
      let entryTime = dayNum * 86400 + hh * 3600 + mm * 60;
      parsedEntries.push((entryTime, line.clone()));
     }
    }
   }
   parsedEntries.sort_by_key(|(t, _)| *t);
   let mut filtered = parsedEntries.clone();

   let mut removed = 0;
   for i in 0..2.min(filtered.len()) {
    let (t, _line) = filtered[i - removed].clone();
    if t < cutoff {
     filtered.remove(i - removed);
     removed += 1;
    }
   }
   history = filtered.into_iter().map(|(_, l)| l).collect();
   let file = OpenOptions::new(). write(true). create(true). truncate(true). open("log.txt"). unwrap();

   let mut writer = BufWriter::new(file);
   writeln!(writer, "[all]").unwrap();
   for (name, secs) in &allTotals {
    writeln!(writer, "{} {}", name, secs).unwrap();
   }
   writeln!(writer, "\n[24h]").unwrap();
   for line in &history {
    writeln!(writer, "{}", line).unwrap();
   }
   writer.flush().unwrap();
   let t = timer.load(Ordering::Relaxed);
   let sleep_time = if isHeadless.load(Ordering::Relaxed) {
    t.max(15)
   } else {
    t
   };
   thread::sleep(Duration::from_secs(sleep_time));
  }
 });
}

fn save() {
 let renameLock = renames.lock().unwrap();
 let removeLock = removes.lock().unwrap();
 let mut data = String::new();
 let value = timer.load(Ordering::Relaxed);
 data.push_str(&format!("setTimer: {}\n", value));

 for (old, new) in renameLock.iter() {
  data.push_str(&format!("rename:{}={}\n", old, new));
 }
 for name in removeLock.iter() {
  data.push_str(&format!("remove:{}\n", name));
 }
 if let Err(e) = std::fs::write("save.txt", data) {
  println!("failed save {}", e);
 }
}

fn load() {
 let mut renameLock = renames.lock().unwrap();
 let mut removeLock = removes.lock().unwrap();
 renameLock.clear();
 removeLock.clear();

 match fs::read_to_string("save.txt") {
  Ok(contents) => {
   for line in contents.lines() {
    let line = line.trim();
    if let Some(numStr) = line.strip_prefix("setTimer:") { 
     if let Ok(num) = numStr.trim().parse::<u64>() {
      timer.store(num, Ordering::Relaxed);
     }
    }else if let Some(rest) = line.strip_prefix("rename:") {
     if let Some((old, new)) = rest.trim().split_once('=') {
      let old = old.trim().to_string();
      let new = new.trim().to_string();
      if let Some(existing) = renameLock.iter_mut().find(|(o, _)| *o == old) {
       existing.1 = new;
      } else {
       renameLock.push((old, new));
      }
     }
    }else if let Some(name) = line.strip_prefix("remove:") {
     removeLock.push(name.trim().to_string());
    }
   }
  } Err(_) => { println!("");
  }
 }
}

fn barGraph() {
 let slotSecs: u64 = 24 * 3600 / 60;
 let mut slots: Vec<String> = vec!["none".to_string(); 60];
 let mut totals: HashMap<String, u64> = HashMap::new();
 let mut allLines: Vec<String> = Vec::new();
 let mut keepLines: Vec<String> = Vec::new();
 let nowSecs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
 let cutoff = nowSecs.saturating_sub(24 * 3600); 

 if let Ok(content) = fs::read_to_string("log.txt") {
  let mut section = "";
  for line in content.lines() {
   let trimmed = line.trim();
   if trimmed.is_empty() { continue; }
   if trimmed.starts_with('[') {
    section = trimmed;
   }
   if section != "[24h]" {
    allLines.push(line.to_string());
   }

   if section == "[24h]" && !line.is_empty() {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 5 && parts[0].starts_with("day") {
     let dayValue = parts[0][3..].parse::<u64>().unwrap_or(0);
     let hh = parts[1][..2].parse::<u64>().unwrap_or(0);
     let mm = parts[1][3..5].parse::<u64>().unwrap_or(0);
     let entryTime = dayValue * 86400 + hh * 3600 + mm * 60;

     if entryTime <= nowSecs && entryTime >= cutoff {
      keepLines.push(line.to_string());
      let program = parts[2].to_string();
      totals.entry(program.clone()).or_insert(0);
      let startSec = (entryTime % 86400) as u64;
      let endSec = startSec + parts.last().unwrap_or(&"0").parse::<u64>().unwrap_or(0);
      let startSlot = (startSec / slotSecs).min(59) as usize;
      let endSlot = (endSec / slotSecs).min(59) as usize;
      for i in startSlot..=endSlot {
       slots[i] = program.clone();
      }
      *totals.get_mut(&program).unwrap() += parts.last().unwrap_or(&"0").parse::<u64>().unwrap_or(0);
     }
    }
   }
  }
 }

 let mut before24h = Vec::new();
 for line in &allLines {
  if line.trim() == "[24h]" { break; }
  before24h.push(line.clone());
 }
 let file = fs::OpenOptions::new(). write(true). create(true). truncate(true). open("log.txt"). unwrap();
 let mut writer = BufWriter::new(file);
 for line in &before24h {
  writeln!(writer, "{}", line).unwrap();
 }
 writeln!(writer, "\n[24h]").unwrap();
 for line in &keepLines {
  writeln!(writer, "{}", line).unwrap();
 }
 writer.flush().unwrap();

 let colors = [
 "\x1b[31;1m", "\x1b[32;1m", "\x1b[33;1m", "\x1b[34;1m",
 "\x1b[35;1m", "\x1b[36;1m", "\x1b[91m", "\x1b[92m",
 "\x1b[93m", "\x1b[94m", "\x1b[95m", "\x1b[96m",
 "\x1b[97m", "\x1b[90m",];
 let mut colorMap: HashMap<String, &str> = HashMap::new();
 for program in totals.keys() {
  let mut hasher = DefaultHasher::new();
  program.hash(&mut hasher);
  let colorIdx = (hasher.finish() as usize) % colors.len();
  colorMap.insert(program.clone(), colors[colorIdx]);
 }
 colorMap.insert("none".to_string(), "\x1b[37m");

 for slot in &slots {
  print!("{}█\x1b[0m", colorMap.get(slot).unwrap_or(&"\x1b[37m"));
 }
 println!();
 println!("\nmostRecent:");
 let mut lastSeen: HashMap<String, u64> = HashMap::new();
 for line in &keepLines {
  let parts: Vec<&str> = line.split_whitespace().collect();
  if parts.len() >= 5 && parts[0].starts_with("day") {
   let dayValue = parts[0][3..].parse::<u64>().unwrap_or(0);
   let hh = parts[1][..2].parse::<u64>().unwrap_or(0);
   let mm = parts[1][3..5].parse::<u64>().unwrap_or(0);
   let entryTime = dayValue * 86400 + hh * 3600 + mm * 60;
   let program = parts[2].to_string();
   lastSeen.insert(program, entryTime);
  }
 }

 let mut items: Vec<_> = totals.iter().collect();
 items.sort_by(|(a_name, _), (b_name, _)| {
  let aTime = lastSeen.get(*a_name).copied().unwrap_or(0);
  let bTime = lastSeen.get(*b_name).copied().unwrap_or(0);
  bTime.cmp(&aTime).then_with(|| b_name.cmp(a_name))
 });

 for program in items {
  println!("{}█\x1b[0m {:<15}", colorMap[program.0], program.0);
 }
 println!("{}█\x1b[0m none", colorMap["none"]);
}

fn pieGraph() {
 let renamesLock = renames.lock().unwrap();
 let removesLock = removes.lock().unwrap();
 let mut totals: HashMap<String, u64> = HashMap::new();

 if let Ok(content) = read_to_string("log.txt") {
  let mut section = "";
  for line in content.lines() {
   if line.starts_with('[') {
    section = line.trim();
   } else if !line.is_empty() && section == "[all]" {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() == 2 {
     if let Ok(secs) = parts[1].parse::<u64>() {
      let mut name = parts[0].to_string();

      for (old, new) in renamesLock.iter() {
       if name == *old {
        name = new.clone();
        break;
       }
      }
      if removesLock.iter().any(|r| *r == name) {
       continue;
      }
      *totals.entry(name).or_insert(0) += secs;
     }
    }
   }
  }
 }

 if totals.is_empty() {
  println!("noneIn [all]");
  return;
 }
 let mut sorted: Vec<(String, u64)> = totals.into_iter().collect();
 sorted.sort_by(|a, b| b.1.cmp(&a.1));
 let top: Vec<(String, u64)> = sorted.into_iter().take(5).collect();
 let totalTime: u64 = top.iter().map(|(_, t)| *t).sum();
 if totalTime == 0 {
  println!("no data");
  return;
 }

 let mut wedges = Vec::new();
 let mut acc = 0.0;
 for (_, t) in &top {
  let frac = (*t as f64) / (totalTime as f64);
  let size = frac * TAU;
  wedges.push((acc, acc + size));
  acc += size;
 }

 let radius: i32 = 8;
 let xScale: f64 = 2.0;
 let colors = [31, 33, 32, 36, 34, 35];
 for y in -radius..=radius {
  for x in -(radius as f64 * xScale).round() as i32..=(radius as f64 * xScale).round() as i32 {
   let dx = (x as f64) / xScale;
   let dy = y as f64;
   if dx * dx + dy * dy <= (radius as f64) * (radius as f64) {
    let mut angle = dy.atan2(dx);
    if angle < 0.0 {
     angle += TAU;
    }
    let mut idx = 0;
    for (i, (start, end)) in wedges.iter().enumerate() {
     if angle >= *start && angle < *end {
      idx = i;
      break;
     }
    }
    let colorCode = colors[idx % colors.len()];
    print!("\x1b[{}m█\x1b[0m", colorCode);
   } else {
    print!(" ");
   }
  }
  println!();
 }

 println!("\ntop5:");
 for (i, (name, secs)) in top.iter().enumerate() {
  let colorCode = colors[i % colors.len()];
  let hrs = secs / 3600;
  let mins = (secs % 3600) / 60;
  let s = secs % 60;
  println!("\x1b[{}m█\x1b[0m {:<12} - {:02}:{:02}:{:02}", colorCode, name, hrs, mins, s);
 }
}

fn loadIcon(path: &std::path::Path) -> tray_icon::Icon {
 let (iconRgba, iconWidth, iconHeight) = {
  let image = image::open(path). expect("failedIconPath"). into_rgba8();
  let (width, height) = image.dimensions();
  let rgba = image.into_raw();
  (rgba, width, height)
  };
 tray_icon::Icon::from_rgba(iconRgba, iconWidth, iconHeight).expect("failed icon")
 }

fn trayIcon(){
 let eventLoop = EventLoopBuilder::<UserEvent>::with_user_event().build();
 tray_icon::TrayIconEvent:: set_event_handler(Some(move |_event| {}));
 let proxy = eventLoop.create_proxy();
 MenuEvent::set_event_handler(Some(move |event| {
 let _ = proxy.send_event(UserEvent::MenuEvent(event));
 }));

 let trayMenu = Menu::new();
 let openI = MenuItem::new("Open", true, None);
 let quitI = MenuItem::new("Quit", true, None);
 let _ = trayMenu.append_items( &[&openI,&PredefinedMenuItem::separator(), &quitI,]);
 let mut trayIcon = None;
 eventLoop.run(move |event, _, control_flow| {
  *control_flow = ControlFlow::Wait;

  match event {
   Event::NewEvents(tao::event::StartCause::Init) => {
    let icon = loadIcon(std::path::Path::new("icon.png"));
    trayIcon = Some(TrayIconBuilder::new(). with_menu(Box::new(trayMenu.clone())). with_tooltip("tao - awesome windowing lib"). with_icon(icon.clone()). build(). unwrap());
    }

   Event::UserEvent(UserEvent::MenuEvent(event)) => {
    if event.id == openI.id() {
     reopenTerminal();
     let _ = Command::new("xdg-open"). arg("terminal:"). spawn();
    }else if event.id == quitI.id() {
     trayIcon.take();
     *control_flow = ControlFlow::Exit;
    }
   }
  _ => {}
  }
 })
}

fn headless() {
 let exe = std::env::current_exe().expect("failed current exe");
 let result = std::process::Command::new("nohup"). arg(&exe). arg("--tray"). stdout(std::process::Stdio::null()). stderr(std::process::Stdio::null()). spawn();
 match result {
  Ok(_) => {
   std::process::exit(0);
  }
  Err(e) => eprintln!("failedBackground {e}"),
 }
}

fn addRename(input: &str) {
 let parts: Vec<&str> = input.split(';').collect();
 if parts.len() != 2 {
  println!("try 'rename (old);(new)'");
  return;
 }
 let old = parts[0].trim().to_string();
 let new = parts[1].trim().to_string();
 if old.is_empty() || new.is_empty() {
  return;
 }
 let mut renameLock = renames.lock().unwrap();
 if let Some(pair) = renameLock.iter_mut().find(|(o, _)| o == &old) {
  pair.1 = new.clone();
 } else {
  renameLock.push((old.clone(), new.clone()));
 }
 drop(renameLock);
}

fn commands() {
 thread::spawn(move || {
  let mut input = String::new();
  loop {
   print!("> ");
   io::stdout().flush().unwrap();
   input.clear();
   if io::stdin().read_line(&mut input).is_err() {
    break;
   }
   let cmd = input.trim();
   if cmd.is_empty() {
    continue;
   }
   let parts: Vec<&str> = cmd.split_whitespace().collect();
   if parts.is_empty() {
    continue;
   }
   match parts[0] {
    "help" => {
     println!("{}{}", helpText, timer.load(Ordering::Relaxed));
    }
    "setTimer" => {
     if parts.len() > 1 {
      if let Ok(num) = parts[1].parse::<u64>() {
       println!("timer= {}\nupdateIncoming {}", num, timer.load(Ordering::Relaxed)
       );
       timer.store(num, Ordering::Relaxed);
       save();
      } else {
       println!("notAnInteger {}", parts[1]);
      }
     } else {
      println!("try 'setTimer (number)'");
     }
    }
    "all" => {
     let mut times: HashMap<String, u64> = HashMap::new();
     if let Ok(content) = std::fs::read_to_string("log.txt") {
      let mut section = "";
      for line in content.lines() {
       let trimmed = line.trim();
       if trimmed.is_empty() { continue; }
       if trimmed.starts_with('[') {
        section = trimmed;
        continue;
       }
       if section == "[all]" {
        let mut parts = trimmed.split_whitespace();
        if let (Some(name), Some(secs_str)) = (parts.next(), parts.next()) {
         if let Ok(secs) = secs_str.parse::<u64>() {
          times.insert(name.to_string(), secs);
         }
        }
       }
      }
     }
     if times.is_empty() {
      println!("noData");
     } else {
      pieGraph();
     }
    }
    "24h" => {
     barGraph();
    }
    "clear" => {
     print!("\x1B[2J\x1B[H");
     io::stdout().flush().unwrap();
    }
    "performance" => {
     performance();
    }
    "headless" | "background" => {
     let exe = std::env::current_exe().expect("failed current exe");
     match Command::new(&exe).arg("--headless").spawn() {
      Ok(_) => println!("canQuit"),
      Err(e) => println!("failedBackground {e}"),
     }
    }
    "rename" => {
     if parts.len() > 1 {
      addRename(parts[1]);
      save();
     } else {
      println!("try 'rename (old);(new)'");
     }
    }
    "remove" => {
     let name = cmd. trim_start_matches("remove"). trim_start_matches('='). trim();
     if name.is_empty() {
      println!("try 'remove (name)'");
     } else {
      let mut removeLock = removes.lock().unwrap();
      if !removeLock.contains(&name.to_string()) {
       removeLock.push(name.to_string());
       println!("removed {}", name);
      }
      drop(removeLock);
      save();
     }
    }
    other => {
     println!("unknownCommand {}", other);
    }
   }
   io::stdout().flush().unwrap();
  }
 });
}

fn main() {
 load();
 let args: Vec<String> = std::env::args().collect();
 if args.contains(&"--tray".to_string()) {
  isHeadless.store(true, Ordering::Relaxed);
  logTime();
  trayIcon();
  return;
 }
 if args.contains(&"--headless".to_string()) {
  isHeadless.store(true, Ordering::Relaxed);
  headless();
  return;
 }
 logTime();
 println!("{}{}", helpText, timer.load(Ordering::Relaxed));
 commands();
 trayIcon();
}