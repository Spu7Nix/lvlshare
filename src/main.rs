use std::env;

use std::fs;

use std::path::PathBuf;
use std::time::Instant;

use base64;

use libflate::gzip;

mod levelstring;

use glib::types::StaticType;
use gtk::ComboBoxExt;
use gtk::GtkListStoreExt;
use gtk::HeaderBarExt;

use gtk::Orientation::*;
use gtk::{ButtonExt, Inhibit, LabelExt, ListStore, OrientableExt, TreeModelExt, WidgetExt};

use gio::prelude::*;
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, CellRendererText, Label, Orientation, TreeView, TreeViewColumn,
    WindowPosition,
};

use relm::{init, Component, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum HeaderMsg {}

#[widget]
impl Widget for Header {
    fn model() -> () {}

    fn update(&mut self, event: HeaderMsg) {}

    view! {
        #[name="titlebar"]
        gtk::HeaderBar {
            title: Some("Title"),
            show_close_button: true,
        }
    }
}

pub struct Model {
    header: Component<Header>,
}

#[derive(Msg)]
pub enum Msg {
    Quit,
    Export,
}
fn error_message(message: &str, parent_window: &gtk::Window) {
    use gtk::{ButtonsType, DialogFlags, MessageDialog, MessageType};
    let dialog = MessageDialog::new(
        Some(parent_window),
        DialogFlags::empty(),
        MessageType::Info,
        ButtonsType::Ok,
        message,
    );
    dialog.run();
    dialog.close();
}

#[widget]
impl Widget for Win {
    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),

            Msg::Export => {
                use gtk::{ResponseType, Window};
                match self.level_select.get_active_text() {
                    Some(name) => {
                        use gtk::{FileChooserAction, FileChooserNative};
                        let dialog = FileChooserNative::new::<Window>(
                            Some("Save File"),
                            Some(&self.window),
                            FileChooserAction::Save,
                            Some("_Save"),
                            Some("Cancel"),
                        );

                        let name_str = String::from(name);

                        let filter = gtk::FileFilter::new(); // http://gtk-rs.org/docs/gtk/struct.FileFilter.html
                        filter.add_pattern("*.lvl");

                        dialog.add_filter(&filter);
                        dialog.set_current_name(PathBuf::from(name_str.clone() + ".lvl"));

                        match dialog.run() {
                            gtk::ResponseType::Cancel => {}

                            _ => {
                                let level_string = &levelstring::export_level(&name_str);

                                let mut file =
                                    fs::File::create(PathBuf::from(dialog.get_filename().unwrap()))
                                        .expect("Error when creating file.");
                                use std::io::Write;
                                file.write_all(level_string)
                                    .expect("Error when writing to file");
                            }
                        }
                    }

                    None => error_message("Select a level to export!", &self.window),
                }
                //let level_string = levelstring::export_level(&name);
            }
        }
    }
    fn model() -> Model {
        let header = init::<Header>(()).expect("Header");

        Model { header }
    }

    view! {
        #[name = "window"]
        gtk::Window(gtk::WindowType::Toplevel) {
            titlebar: Some(self.model.header.widget()),
            title: "LVLShare",
            decorated: true,


            gtk::Box {
                homogeneous: true,

                orientation: Horizontal,

                gtk::Box {
                    spacing: 20,
                    orientation: Vertical,
                    #[name = "level_select"]
                    gtk::ComboBoxText {
                        model: Some(&create_level_list())
                    },

                    gtk::Button {
                        label: "Export level as .lvl",
                        clicked => Msg::Export,
                    },
                },

                gtk::Box {
                    orientation: Vertical,
                },





            },


            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}

fn create_level_list() -> ListStore {
    // Creation of a model with two rows.
    let model = ListStore::new(&[String::static_type()]);

    // Filling up the tree view.
    let entries = levelstring::get_level_names();

    for entry in entries.iter() {
        model.insert_with_values(None, &[0], &[&entry]);
    }
    model
}

fn main() {
    Win::run(()).unwrap();
    /*let start_time = Instant::now();

    let mut args = env::args();
    args.next();

    let gd_path = PathBuf::from(std::env::var("localappdata").expect("No local app data"))
        .join("GeometryDash/CCLocalLevels.dat");

    let save_file = fs::File::open(gd_path.clone()).expect("Cannot find savefile!");
    let mut xor = xorstream::Transformer::new(vec![11], save_file);
    let b64 = base64::read::DecoderReader::new(&mut xor, base64::URL_SAFE);
    let save_decryptor = gzip::Decoder::new(b64).unwrap();

    match args.next().expect("Expected command!").as_ref() {
        "export" => {
            let level_name = args.next().unwrap();

            let level = levelstring::get_level_string(save_decryptor, &level_name);

            use std::io::Write;
            let mut file = fs::File::create(format!("{}.lvl", level_name)).unwrap();
            file.write_all(&level).unwrap();
        }

        "import" => {
            levelstring::export_level(PathBuf::from(args.next().unwrap()), gd_path, save_decryptor);
        }

        _ => panic!("Unknown command"),
    }

    println!(
        "Completed in {} milliseconds!",
        start_time.elapsed().as_millis()
    );*/
}
