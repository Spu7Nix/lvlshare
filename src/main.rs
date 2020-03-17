mod levelstring;
use std::env;

use std::fs;

use std::path::PathBuf;

fn main() {
    let mut args = env::args();
    args.next();

    let gd_path = PathBuf::from(std::env::var("localappdata").expect("No local app data"))
        .join("GeometryDash/CCLocalLevels.dat");

    match args.next().expect("Expected command!").as_ref() {
        "export" => {
            let file_content =
                fs::read_to_string(gd_path).expect("Your local geometry dash files were not found");

            let level_name = args.next().unwrap();

            let level = levelstring::get_level_string(file_content, &level_name);

            use std::io::Write;
            let mut file = fs::File::create(format!("{}.lvl", level_name)).unwrap();
            file.write_all(&level).unwrap();
        }

        _ => panic!("Unknown command"),
    }
}
