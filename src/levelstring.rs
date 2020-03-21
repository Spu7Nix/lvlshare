// useful things for dealing with gd level data

use base64;
use libflate::{gzip, zlib};
use std::fs;
use std::io::Read;

use std::path::PathBuf;

use std::time::Instant;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;

use std::io::BufReader;
use std::io::Cursor;

pub fn get_level_string(save_decryptor: impl Read, level_name: &str) -> Vec<u8> {
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
                                            return encoder.finish().into_result().unwrap();
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
                                    panic!(
                                        "Error at position {}: {:?}",
                                        reader.buffer_position(),
                                        e
                                    )
                                }
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => panic!("Level \"{}\" not found!", level_name),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }
    }
}

pub fn export_level(level_file: PathBuf, save_file: PathBuf, save_decryptor: impl Read) {
    let mut start_time = Instant::now();

    let mut reader = Reader::from_reader(BufReader::new(save_decryptor));
    reader.trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    //decompress the level file

    let level_compressed = fs::File::open(level_file).expect("Cannot find specified file!");
    let mut level_decompressor = gzip::Decoder::new(level_compressed).unwrap();

    let mut level = Vec::new();
    level_decompressor.read_to_end(&mut level).unwrap();

    //write it in
    let mut buf = Vec::new();
    for _ in 0..11 {
        match reader.read_event(&mut buf) {
            Ok(e) => writer.write_event(e).unwrap(),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
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

            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
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

    fs::write(save_file, encoded).unwrap();
}
