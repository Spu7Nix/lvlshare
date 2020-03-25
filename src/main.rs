//#![windows_subsystem = "windows"]

mod levelstring;

extern crate sciter;
use sciter::{
    dispatch_script_call, make_args,
    types::{RECT, SCITER_CREATE_WINDOW_FLAGS::*},
    Element, Value, Window,
};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
struct Handler {
    host: Weak<sciter::Host>,
}

impl Handler {
    fn export_level(&self, level_name: String, mut location: String) {
        use std::io::Write;
        location.replace_range(..7, "");
        location = location.replace("%20", " ");
        println!("{}", location);
        let path = PathBuf::from(location);

        let mut file = File::create(path).expect("Error creating file.");
        match levelstring::export_level(&level_name) {
            Ok(level) => {
                file.write_all(&level).expect("Error writing to file.");
            }
            Err(err) => message_box(format!("Error when exporting level: {}", err), &self.host),
        };
    }

    fn import_file(&self, mut level_file: String) {
        level_file.replace_range(..7, "");
        level_file = level_file.replace("%20", " ");
        println!("{}", level_file);
        let path = PathBuf::from(level_file);
        let root = &self.host;
        match levelstring::import_level(path) {
            Some(err) => message_box(format!("Error when importing level: {}", err), root),
            None => message_box(format!("Level imported to Geometry Dash!"), root),
        };
    }

    fn gd_found(&self) -> bool {
        let mut gd_found = false;
        process_list::for_each_process(|_, name: &Path| {
            if name.to_str().unwrap().replace("\0", "") == "GeometryDash.exe" {
                gd_found = true;
            }
        })
        .unwrap();
        return gd_found;
    }

    fn get_level_names(&self) -> Value {
        match levelstring::get_level_names() {
            Ok(list) => {
                let mut array = Value::array(0);
                for name in list {
                    array.push(name);
                }
                array
            }
            Err(err) => {
                message_box(err, &self.host);
                Value::array(0)
            }
        }
    }
}

use sciter::HELEMENT;

impl sciter::EventHandler for Handler {
    dispatch_script_call! {
      fn export_level(String, String);
      fn import_file(String);
      fn gd_found();
      fn get_level_names();

    }
}

fn message_box(msg: String, host: &Weak<sciter::Host>) {
    if let Some(host) = host.upgrade() {
        match host.eval_script(&format!("view.msgbox(\"{}\");", msg)) {
            Ok(_) => {}
            Err(_) => {}
        };
    }
}

fn main() {
    // Step 1: Include the 'minimal.html' file as a byte array.
    // Hint: Take a look into 'minimal.html' which contains some tiscript code.
    let html = include_bytes!("gui.htm");

    // Step 2: Enable the features we need in our tiscript code.
    sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(
        sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8		// Enables `Sciter.machineName()`
		| sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8, // Enables opening file dialog (`view.selectFile()`)
    ))
    .unwrap();

    // Enable debug mode for all windows, so that we can inspect them via Inspector.
    sciter::set_options(sciter::RuntimeOptions::DebugMode(true)).unwrap();

    let mut frame = Window::create(
        RECT {
            left: 0,
            top: 0,
            right: 400,
            bottom: 180,
        },
        SW_MAIN | SW_CONTROLS,
        None,
    );

    let handler = Handler {
        host: Rc::downgrade(&frame.get_host()),
    };
    frame.event_handler(handler);
    frame.load_html(html, None);

    frame.run_app();
}
