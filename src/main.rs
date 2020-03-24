#![windows_subsystem = "windows"]

use std::fs;

use std::path::{Path, PathBuf};

mod levelstring;

use glib::types::StaticType;
use gtk::ComboBoxExt;

use gtk::Orientation::*;
use gtk::{ButtonExt, Inhibit, LabelExt, ListStore, OrientableExt, WidgetExt};

use gtk::prelude::*;

use relm::Widget;
use relm_derive::{widget, Msg};

pub struct Model {}

#[derive(Msg)]
pub enum Msg {
    Quit,
    Export,
    Import,
}
fn info_message(message: &str, parent_window: Option<&gtk::Window>) {
    use gtk::{ButtonsType, DialogFlags, MessageDialog, MessageType};
    let dialog = MessageDialog::new(
        parent_window,
        DialogFlags::empty(),
        MessageType::Info,
        ButtonsType::Ok,
        message,
    );
    dialog.run();
    dialog.close();
}

fn get_title_atributes() -> pango::AttrList {
    let attrs = pango::AttrList::new();
    attrs.insert(pango::Attribute::new_scale(2.0).unwrap());
    attrs
}

fn create_filter() -> gtk::FileFilter {
    let filter = gtk::FileFilter::new();
    filter.add_pattern("*.lvl");
    filter
}

#[widget]
impl Widget for Win {
    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),

            Msg::Export => {
                use gtk::Window;
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

                        let filter = create_filter();

                        dialog.add_filter(&filter);
                        dialog.set_current_name(PathBuf::from(name_str.clone() + ".lvl"));

                        match dialog.run() {
                            gtk::ResponseType::Cancel => {}

                            _ => {
                                let level_string = &match levelstring::export_level(&name_str) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        info_message(&e, Some(&self.window));
                                        return;
                                    }
                                };

                                let mut file = match fs::File::create(PathBuf::from(
                                    dialog.get_filename().unwrap(),
                                )) {
                                    Ok(file) => file,
                                    Err(e) => {
                                        info_message(
                                            &(String::from("Error when creating file: ")
                                                + &format!("{}", e)),
                                            Some(&self.window),
                                        );
                                        return;
                                    }
                                };

                                use std::io::Write;
                                file.write_all(level_string)
                                    .expect("Error when writing to file");
                            }
                        }
                    }

                    None => info_message("Select a level to export!", Some(&self.window)),
                }
                //let level_string = levelstring::export_level(&name);
            }

            Msg::Import => {
                let mut gd_found = false;
                process_list::for_each_process(|_, name: &Path| {
                    if name.to_str().unwrap().replace("\0", "") == "GeometryDash.exe" {
                        info_message("Close Geometry Dash before importing!", Some(&self.window));
                        gd_found = true;
                    }
                })
                .unwrap();
                if gd_found {
                    return;
                }

                match self.file_select.get_filename() {
                    Some(file) => {
                        match levelstring::import_level(file) {
                            Some(err) => info_message(&err, Some(&self.window)),
                            None => info_message(
                                "Level has been imported to Geometry dash",
                                Some(&self.window),
                            ),
                        };
                    }
                    None => info_message("Select a file to import!", Some(&self.window)),
                }
            }
        }
    }
    fn model() -> Model {
        Model {}
    }

    view! {
        #[name = "window"]
        gtk::Window(gtk::WindowType::Toplevel) {

            title: "LVLShare",
            decorated: true,


            gtk::Box {
                homogeneous: true,
                spacing: 10,

                orientation: Horizontal,

                gtk::Box {
                    //homogeneous: true,
                    spacing: 10,
                    orientation: Vertical,
                    gtk::Label {
                        text: "Export Level",
                        attributes: Some(&get_title_atributes()),
                    },
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
                    //homogeneous: true,
                    spacing: 10,
                    orientation: Vertical,
                    gtk::Label {
                        text: "Import File",
                        attributes: Some(&get_title_atributes()),
                    },
                    #[name = "file_select"]
                    gtk::FileChooserButton {
                        filter: &create_filter(),
                    },
                    gtk::Button {
                        label: "Import file as a Geometry Dash level",
                        clicked => Msg::Import,
                    },
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
    let entries = match levelstring::get_level_names() {
        Ok(list) => list,
        Err(e) => {
            info_message(&e, None);
            std::process::exit(0)
        }
    };

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
