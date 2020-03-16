mod levelstring;
use std::env;

use std::fs;

use std::path::PathBuf;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("please provide a command (export, import) and a level name")
    }
    let gd_path = PathBuf::from(std::env::var("localappdata").expect("No local app data"))
        .join("GeometryDash/CCLocalLevels.dat");
        
    match args[1].as_ref() {
        "export" => {
            let file_content = fs::read_to_string(gd_path).expect("Your local geometry dash files were not found");

            let level = levelstring::get_level_string(file_content, &args[2]);

            use std::io::Write;
            let mut file = fs::File::create(format!("{}.gdlvl", args[2])).unwrap();
            file.write_all(level.as_bytes());
        },

        _ => panic!("Unknown command")
    }
}
