use super::emit;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicBool, AtomicU32};

static BAUD_RATE: AtomicU32 = AtomicU32::new(9600);
static SERIAL_PORT: AtomicU32 = AtomicU32::new(0);
static CONNECTED: AtomicBool = AtomicBool::new(false);
static KEEP_CONNECTING: AtomicBool = AtomicBool::new(false);

pub fn baud_rate() -> u32 {
  BAUD_RATE.load(SeqCst)
}
pub fn serial_port() -> u32 {
  SERIAL_PORT.load(SeqCst)
}
pub fn connected() -> bool {
  CONNECTED.load(SeqCst)
}
pub fn keep_connecting() -> bool {
  KEEP_CONNECTING.load(SeqCst)
}
#[tauri::command]
pub fn set_baud_rate(value: u32) -> u32 {
  BAUD_RATE.store(value, SeqCst);
  emit("baud_rate", value);
  value
}
#[tauri::command]
pub fn set_serial_port(value: u32) -> u32 {
  SERIAL_PORT.store(value, SeqCst);
  emit("serial_port", value);
  value
}
#[tauri::command]
pub fn set_connected(value: bool) -> bool {
  CONNECTED.store(value, SeqCst);
  emit("connected", value);
  value
}
#[tauri::command]
pub fn set_keep_connecting(value: bool) -> bool {
  KEEP_CONNECTING.store(value, SeqCst);
  emit("keep_connecting", value);
  value
}

#[tauri::command]
pub fn refresh_state() {
  emit("baud_rate", baud_rate());
  emit("serial_port", serial_port());
  emit("connected", connected());
  emit("keep_connecting", keep_connecting());
}

#[tauri::command]
pub fn refresh_ports() {
  let ports = tokio_serial::available_ports().unwrap_or_else(|_| vec![]);
  println!("ports {ports:?}");
  let ports = ports
    .into_iter()
    .filter(|p| {
      
    })
    .map(|p| p.port_name)
    .collect::<Vec<String>>();
  emit("available_ports", ports);
}
