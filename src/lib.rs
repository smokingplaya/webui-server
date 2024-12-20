use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use gmod::{gmod13_close, gmod13_open, lua::State, lua_string};

mod server;

static ALREADY_BINDED: AtomicBool = AtomicBool::new(false);

fn log(msg: &str) {
  println!("[WebUI-GMod] {msg}");
}

unsafe extern "C-unwind" fn run(lua: State) -> i32 {
  let port = lua.check_integer(1);

  if ALREADY_BINDED.load(Relaxed) {
    log("The webserver is already up and running!");

    return 0;
  }

  std::thread::spawn(move || {
    actix_web::rt::System::new()
      .block_on(server::run(port))
  });

  ALREADY_BINDED.store(true, Relaxed);

  log(&format!("The webserver was running on port {port}."));

  0
}

#[gmod13_open]
unsafe fn gmod13_open(lua: State) -> i32 {
  let name = lua_string!("webui_server");

  lua.get_global(name);

  if lua.is_nil(-1) {
    lua.pop();
    lua.new_table();
  }

  lua.push_function(run);
  lua.set_field(-2, lua_string!("run"));

  lua.set_global(name);

  0
}

#[gmod13_close]
fn gmod13_close(_l: State) -> i32 {
  0
}