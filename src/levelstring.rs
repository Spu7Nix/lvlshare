// useful things for dealing with gd level data

use libflate::{gzip, zlib};
use std::fs;
use std::io::Read;

use std::path::PathBuf;

use quick_xml::events::Event;
use quick_xml::Reader;
use quick_xml::Writer;

use std::collections::HashMap;

use std::io::BufReader;
use std::io::Cursor;

pub fn get_local_levels_path() -> Result<PathBuf, String> {
    Ok(PathBuf::from(match std::env::var("localappdata") {
        Ok(path) => path,
        Err(e) => return Err(e.to_string()),
    })
    .join("GeometryDash/CCLocalLevels.dat"))
}

pub fn get_level_names() -> Result<Vec<String>, String> {
    let gd_path = match get_local_levels_path() {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    let save_file = match fs::File::open(gd_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Cannot find savefile: {}", e)),
    };
    let mut xor = xorstream::Transformer::new(vec![11], save_file);
    let b64 = base64::read::DecoderReader::new(&mut xor, base64::URL_SAFE);
    let save_decryptor = gzip::Decoder::new(b64).unwrap();

    let mut reader = Reader::from_reader(BufReader::new(save_decryptor));
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut k2_detected = false;
    let mut names = Vec::<String>::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Text(e)) => {
                let text = e.unescape_and_decode(&reader).unwrap();
                if k2_detected {
                    names.push(text);
                    k2_detected = false;
                } else if text == "k2" {
                    k2_detected = true;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => (), // There are several other `Event`s we do not consider here
        }
    }
    Ok(names)
}

pub fn export_level(level_name: &str) -> Result<Vec<u8>, String> {
    let gd_path = match get_local_levels_path() {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    let save_file = match fs::File::open(gd_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Cannot find GD savefile: {}", e)),
    };

    let mut xor = xorstream::Transformer::new(vec![11], save_file);
    let b64 = base64::read::DecoderReader::new(&mut xor, base64::URL_SAFE);
    let save_decryptor = gzip::Decoder::new(b64).unwrap();

    let mut reader = Reader::from_reader(BufReader::new(save_decryptor));
    reader.trim_text(true);
    let mut buf = Vec::new();

    for _ in 0..7 {
        reader.read_event(&mut buf).unwrap();
    }

    //thing that gets the level
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                if name == b"d" {
                    //println!("reading from level in position \"{}\"", current_outer_key);
                    let mut d_value_layers = 0; //incase there are any other d values in there, make sure to include them and not close when they close
                    let mut k2_detected = false;
                    let mut level_found = false;
                    let mut not_the_level = false;
                    //writes the level
                    let mut writer = Writer::new(Cursor::new(Vec::new()));

                    loop {
                        match reader.read_event(&mut buf) {
                            Ok(Event::Text(e)) => {
                                if !not_the_level {
                                    if !level_found {
                                        let text = e.unescape_and_decode(&reader).unwrap();
                                        if k2_detected {
                                            if text == level_name {
                                                //we found the level!
                                                level_found = true;
                                                k2_detected = false;
                                            } else {
                                                not_the_level = true;
                                            }
                                        }
                                        if text == "k2" {
                                            k2_detected = true
                                        }
                                    }
                                    assert!(writer.write_event(Event::Text(e)).is_ok())
                                }
                            }

                            Ok(Event::Start(e)) => {
                                if e.name() == b"d" {
                                    d_value_layers += 1
                                }
                                if !not_the_level {
                                    assert!(writer.write_event(Event::Start(e)).is_ok())
                                }
                            }
                            Ok(Event::End(e)) => {
                                if e.name() == b"d" {
                                    if d_value_layers == 0 {
                                        buf.clear();
                                        if not_the_level {
                                            break;
                                        } else {
                                            let mut encoder =
                                                gzip::Encoder::new(Vec::new()).unwrap();
                                            std::io::copy(
                                                &mut &writer.into_inner().into_inner()[..],
                                                &mut encoder,
                                            )
                                            .unwrap();
                                            return Ok(encoder.finish().into_result().unwrap());
                                        }
                                    } else {
                                        d_value_layers -= 1
                                    }
                                }
                                if !not_the_level {
                                    assert!(writer.write_event(Event::End(e)).is_ok())
                                }
                            }
                            Ok(e) => {
                                if !not_the_level {
                                    assert!(writer.write_event(e).is_ok())
                                }
                            }

                            Err(e) => {
                                if !not_the_level {
                                    return Err(format!(
                                        "Error at position {}: {:?}",
                                        reader.buffer_position(),
                                        e
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => return Err(format!("Level \"{}\" not found!", level_name)),
            Err(e) => {
                return Err(format!(
                    "Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => (), // There are several other `Event`s we do not consider here
        }
    }
}

pub fn import_level(level_file: PathBuf) -> Option<String> {
    let gd_path = match get_local_levels_path() {
        Ok(p) => p,
        Err(e) => return Some(e),
    };

    let save_file = match fs::File::open(gd_path.clone()) {
        Ok(file) => file,
        Err(e) => return Some(format!("Cannot find savefile: {}", e)),
    };
    let mut xor = xorstream::Transformer::new(vec![11], save_file);
    let b64 = base64::read::DecoderReader::new(&mut xor, base64::URL_SAFE);
    let save_decryptor = gzip::Decoder::new(b64).unwrap();

    let mut reader = Reader::from_reader(BufReader::new(save_decryptor));
    reader.trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    //decompress the level file

    let level_compressed = match fs::File::open(level_file) {
        Ok(file) => file,
        Err(e) => return Some(e.to_string()),
    };
    let mut level_decompressor = gzip::Decoder::new(level_compressed).unwrap();

    let mut level = Vec::new();
    level_decompressor.read_to_end(&mut level).unwrap();

    //NEED TO FIND: <k>GS_value</k>

    //write it in
    let mut buf = Vec::new();
    for _ in 0..11 {
        match reader.read_event(&mut buf) {
            Ok(e) => writer.write_event(e).unwrap(),
            Err(e) => {
                return Some(format!(
                    "Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
        };
    }

    writer.write(b"<k>k_0</k><d>").unwrap();

    writer.write(&level).unwrap();

    writer.write(b"</d>").unwrap();
    let mut d_layer = 0;
    let mut level_num = 1;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                if name == b"d" {
                    d_layer += 1;
                    writer.write_event(Event::Start(e)).unwrap();
                } else if name == b"k" && d_layer == 0 {
                    writer.write(b"<k>k_").unwrap();
                    writer.write(level_num.to_string().as_bytes()).unwrap();
                    writer.write(b"</k>").unwrap();

                    reader.read_event(&mut buf).unwrap();
                    reader.read_event(&mut buf).unwrap();
                    level_num += 1;
                } else {
                    writer.write_event(Event::Start(e)).unwrap();
                }
            }

            Ok(Event::End(e)) => {
                if e.name() == b"d" {
                    d_layer -= 1;
                }
                writer.write_event(Event::End(e)).unwrap();
            }
            Ok(Event::Eof) => break,

            Ok(e) => {
                writer.write_event(e).unwrap();
            }

            Err(e) => {
                return Some(format!(
                    "Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
        };

        buf.clear()
    }

    let bytes = writer.into_inner().into_inner();
    //encrypt level save
    use std::io::Write;

    let mut encoder = zlib::Encoder::new(Vec::new()).unwrap();
    encoder.write_all(&bytes).unwrap();
    let compressed = encoder.finish().into_result().unwrap();

    use crc32fast::Hasher;

    let mut hasher = Hasher::new();
    hasher.update(&bytes);
    let checksum = hasher.finalize();

    let data_size = bytes.len() as u32;

    let mut with_signature = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\x0b".to_vec();
    with_signature.extend(&compressed[2..compressed.len() - 4]);
    with_signature.extend(checksum.to_le_bytes().to_vec());
    with_signature.extend(data_size.to_le_bytes().to_vec());

    let mut encoded = Vec::new();
    {
        let mut encoder = base64::write::EncoderWriter::new(&mut encoded, base64::URL_SAFE);
        encoder.write_all(&with_signature).unwrap();
        encoder.finish().unwrap();
    }

    for b in encoded.iter_mut() {
        *b ^= 11
    }

    fs::write(gd_path, encoded).unwrap();
    None
}

pub fn get_user_stats() -> Result<HashMap<String, String>, String> {
    let gd_path = PathBuf::from(match std::env::var("localappdata") {
        Ok(path) => path,
        Err(e) => return Err(e.to_string()),
    })
    .join("GeometryDash/CCGameManager.dat");

    let save_file = match fs::File::open(gd_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Cannot find savefile: {}", e)),
    };
    let mut xor = xorstream::Transformer::new(vec![11], save_file);
    let b64 = base64::read::DecoderReader::new(&mut xor, base64::URL_SAFE);
    let save_decryptor = gzip::Decoder::new(b64).unwrap();

    let mut reader = Reader::from_reader(BufReader::new(save_decryptor));
    reader.trim_text(true);
    let mut buf = Vec::new();

    let mut stats = HashMap::new();
    let mut reading_stats = false;

    let mut read_key = true;
    let mut current_stat = String::new();

    let mut name_detected = false;
    let mut user_id_detected = false;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Text(e)) => {
                let text = e.unescape_and_decode(&reader).unwrap();

                if reading_stats {
                    if read_key {
                        let mut skipped = false;
                        let decrypted_key = match text.as_ref() {
                            "1" => "Jumps",
                            "2" => "Total attempts",
                            "4" => "Completed online levels",
                            "5" => "Demons beaten",
                            "6" => "Stars",
                            "13" => "Diamonds",
                            "14" => "Orbs",
                            "8" => "Coins",
                            "12" => "User coins",
                            "9" => "Killed players",

                            "GS_completed" => {
                                skipped = true;
                                reading_stats = false;
                                ""
                            }

                            _ => {
                                //skip
                                skipped = true;
                                for _ in 0..5 {
                                    if let Err(e) = reader.read_event(&mut buf) {
                                        return Err(format!(
                                            "Error at position {}: {:?} (while skipping stat)",
                                            reader.buffer_position(),
                                            e
                                        ));
                                    };
                                }
                                ""
                            }
                        };
                        if !skipped {
                            current_stat += decrypted_key;

                            read_key = false;
                        }
                    } else {
                        stats.insert(current_stat, text);
                        current_stat = String::new();

                        read_key = true;

                        if stats.len() >= 10 {
                            reading_stats = false;
                        }
                    }
                } else if text == "GS_value" {
                    reading_stats = true
                } else if name_detected {
                    stats.insert("name".to_string(), text);
                    name_detected = false;
                } else if user_id_detected {
                    stats.insert("user_id".to_string(), text);
                    user_id_detected = false;
                } else if text == "playerName" {
                    name_detected = true;
                } else if text == "playerUserID" {
                    user_id_detected = true;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => (), // There are several other `Event`s we do not consider here
        }
    }

    Ok(stats)
}
