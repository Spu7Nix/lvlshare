mod levelstring;
use std::env;

use std::fs;

use std::path::PathBuf;
use std::time::Instant;

use base64;
use libflate::gzip;

fn main() {
    let start_time = Instant::now();

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
    );
}
