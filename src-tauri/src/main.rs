use serde::Serialize;
use tauri::api::shell;
use tauri::{
  App, CustomMenuItem, Manager, Menu, MenuEntry, MenuItem, Submenu, Window, WindowMenuEvent,
  WindowUrl,
};
static mut WINDOW: Option<Window> = None;

pub fn emit<S: Serialize + Clone>(event: &str, payload: S) {
  match unsafe { &WINDOW } {
    Some(window) => window.emit(event, payload).unwrap(),
    None => {}
  }
}

fn info(s: String) {
  emit("log", format!(r#"INFO : {s}"#))
}
fn error(s: String) {
  emit("log", format!(r#"ERROR: {s}"#))
}
mod cfg;

fn main() {
  tauri::Builder::new()
    .invoke_handler(tauri::generate_handler![
      cfg::set_baud_rate,
      cfg::set_serial_port,
      cfg::set_connected,
      cfg::set_keep_connecting,
      cfg::refresh_state,
      cfg::refresh_ports,
    ])
    .menu(menu())
    .on_menu_event(on_menu_event_handler)
    .setup(setup)
    .run(tauri::generate_context!())
    .expect("error while running application");
}

fn menu() -> Menu {
  Menu::with_items([
    MenuEntry::Submenu(Submenu::new(
      "File",
      Menu::with_items([MenuItem::CloseWindow.into()]),
    )),
    MenuEntry::Submenu(Submenu::new(
      "Help",
      Menu::with_items([CustomMenuItem::new("Learn More", "Learn More").into()]),
    )),
  ])
}

fn on_menu_event_handler(event: WindowMenuEvent) {
  let event_name = event.menu_item_id();
  match event_name {
    "Learn More" => {
      let url = "https://web.ooorja.com";
      shell::open(&event.window().shell_scope(), url, None).unwrap();
    }
    _ => {}
  }
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
  let window = Window::builder(app, "main", WindowUrl::default())
    .title("Resonant Electronics")
    .inner_size(800.0, 550.0)
    .position(2560.0, 0.0)
    .min_inner_size(400.0, 200.0)
    .maximized(true)
    .build()
    .expect("Failed to create window");
  #[cfg(debug_assertions)] // only include this code on debug builds
  window.open_devtools();
  unsafe {
    WINDOW = Some(window);
  }
  std::thread::spawn(|| worker_thread());
  Ok(())
}

fn worker_thread() {
  let rt = tokio::runtime::Runtime::new().unwrap();
  let mut i = 0;
  unsafe {
    for r in REG.as_mut() {
      *r = i;
      i += 1;
    }
  }
  rt.block_on(async {
    loop {
      modbus_loop().await;
      tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
  });
}

async fn modbus_loop() {
  if !cfg::keep_connecting() {
    return;
  }
  let serial_port = cfg::serial_port();
  let baud_rate = cfg::baud_rate();
  #[cfg(unix)]
  let port = format!("/dev/ttyUSB{serial_port}");
  #[cfg(windows)]
  let port = format!("COM{serial_port}");
  let builder = tokio_serial::new(&port, baud_rate);
  let server_serial = match tokio_serial::SerialStream::open(&builder) {
    Ok(serial) => serial,
    Err(err) => return error(format!("error opening {port} {err}")),
  };
  let server = server::rtu::Server::new(server_serial);
  info(format!("Simulating client. at {port}"));
  cfg::set_connected(true);
  let signal = async move {
    while cfg::keep_connecting() {
      tokio::time::sleep(std::time::Duration::from_millis(1000)).await
    }
    info(format!("Disconnected {port}"));
    cfg::set_connected(false);
  };
  server.serve_until(|| Ok(MbServer), Box::pin(signal)).await;
}

use tokio_modbus::prelude::{Request, Response};
use tokio_modbus::server::{self, Service};

struct MbServer;

const REG_SIZE: usize = 10240;

static mut REG: [u16; REG_SIZE] = [0; REG_SIZE];

fn read_regs(a: u16, c: u16) -> Vec<u16> {
  unsafe { REG[a as usize..(a + c) as usize].into() }
}
fn write_regs(a: u16, vs: Vec<u16>) -> u16 {
  vs.iter().map(|v| write_reg(a, *v)).len() as u16
}
fn write_reg(a: u16, v: u16) -> u16 {
  unsafe { REG[a as usize] = v };
  v
}
fn read_coils(a: u16, c: u16) -> Vec<bool> {
  unsafe {
    REG[a as usize..(a + c) as usize]
      .iter()
      .map(|v| *v % 2 == 0)
      .collect()
  }
}
fn write_coil(a: u16, v: bool) -> bool {
  match v {
    false => unsafe { REG[a as usize] = 0 },
    true => unsafe { REG[a as usize] = 1 },
  };
  v
}
fn write_coils(a: u16, vs: Vec<bool>) -> u16 {
  vs.iter().map(|v| write_coil(a, *v)).len() as u16
}
use Response::*;
impl<'a> Service for MbServer {
  type Request = Request;
  type Response = Response;
  type Error = std::io::Error;
  type Future = core::future::Ready<Result<Self::Response, Self::Error>>;
  fn call(&self, req: Self::Request) -> Self::Future {
    let rq = format!("{req:3?}");
    let res = match req {
      Request::ReadCoils(a, c) => ReadCoils(read_coils(a, c)),
      Request::ReadDiscreteInputs(a, c) => ReadDiscreteInputs(read_coils(a, c)),
      Request::WriteSingleCoil(a, v) => WriteSingleCoil(a, write_coil(a, v)),
      Request::WriteMultipleCoils(a, vs) => WriteMultipleCoils(a, write_coils(a, vs)),
      Request::ReadInputRegisters(a, c) => ReadInputRegisters(read_regs(a, c)),
      Request::ReadHoldingRegisters(a, c) => ReadHoldingRegisters(read_regs(a, c)),
      Request::WriteSingleRegister(a, v) => WriteSingleRegister(a, write_reg(a, v)),
      Request::WriteMultipleRegisters(a, vs) => WriteMultipleRegisters(a, write_regs(a, vs)),
      // Request::ReadWriteMultipleRegisters(a, c, a, words) => ReadWriteMultipleRegisters(),
      _ => Response::ReadInputRegisters([].into()),
    };
    info(format!("{rq} => {res:?}"));
    core::future::ready(Ok(res))
  }
}
