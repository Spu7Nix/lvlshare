// useful things for dealing with gd level data

use base64;
use libflate::gzip;
use std::io::Read;

fn xor(data: Vec<u8>, key: u8) -> Vec<u8> {
    let mut new_data = Vec::new();

    for b in data {
        //let new_byte = u64::from(b).pow(key);
        new_data.push(b ^ key)
    }
    new_data
}
/*fn base_64_decrypt(encoded: Vec<u8>) -> Vec<u8> {
    let mut new_data = encoded;
    while new_data.len() % 4 != 0 {
        new_data.push(b'=')
    }
    base64::decode(String::from_utf8(new_data).unwrap().as_str()).unwrap()
}*/

use quick_xml::events::Event;
use quick_xml::Reader;
use quick_xml::Writer;
//use std::fs;
use std::io::Cursor;
//use std::path::PathBuf;

pub fn get_level_string(save: String, level_name: &str) -> Vec<u8> {
    //decrypting the savefile
    let xor = xor(save.as_bytes().to_vec(), 11);
    let replaced = String::from_utf8(xor)
        .unwrap()
        .replace("-", "+")
        .replace("_", "/")
        .replace("\0", "");
    let b64 = base64::decode(replaced.as_str()).unwrap();
    let mut decoder = gzip::Decoder::new(&b64[..]).unwrap();
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf).unwrap();

    //println!("{}", String::from_utf8(buf[..1000].to_vec()).unwrap());

    //getting level string

    let mut reader = Reader::from_str(std::str::from_utf8(&buf).unwrap());
    reader.trim_text(true);

    //let buf = Vec::<u8>::new();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    //let mut level_string = String::new();

    let mut buf = Vec::new();
    /*
    take away the first few things:

    <?xml version="1.0"?>
    <plist version="1.0" gjver="2.0">
        <dict>
            <k>LLM_01</k>
            <d>
                <k>_isArr</k>
                <t />
    */
    for _ in 0..7 {
        println!("{:?}", reader.read_event(&mut buf).unwrap());
    }
    let mut current_outer_key = String::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                if name == b"k" {
                    current_outer_key = reader.read_text("k", &mut Vec::new()).unwrap();
                } else if name == b"d" {
                    println!("reading from level in position \"{}\"", current_outer_key);
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
                                                println!("expected {}, found {}", level_name, text);
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
                                    println!("start: {:?}", e);
                                    d_value_layers += 1
                                }
                                if !not_the_level {
                                    assert!(writer.write_event(Event::Start(e)).is_ok())
                                }
                            }
                            Ok(Event::End(e)) => {
                                if e.name() == b"d" {
                                    println!("end:   {:?}", e);
                                    if d_value_layers == 0 {
                                        buf.clear();
                                        if not_the_level {
                                            break;
                                        } else {
                                            return writer.into_inner().into_inner();
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

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
    }

    //NO NEED TO DECRYPT LEVEL STRING
    /*//decrypting level string
    let ls_b64 = base_64_decrypt(
        level_string
            .replace("-", "+")
            .replace("_", "/")
            .replace("\0", "")
            .as_bytes()
            .to_vec(),
    );

    //println!("{}", String::from_utf8(ls_b64.clone()).unwrap());

    let mut ls_decoder = gzip::Decoder::new(&ls_b64[..]).unwrap();
    let mut ls_buf = Vec::new();
    ls_decoder.read_to_end(&mut ls_buf).unwrap();

    String::from_utf8(ls_buf).unwrap()*/
}

/*

pub fn encrypt_level_string(ls: String, old_ls: String, path: PathBuf) {
    let file_content = fs::read_to_string(path.clone()).unwrap();

    //decrypting the savefile
    let xor_encrypted = xor(file_content.as_bytes().to_vec(), 11);
    let replaced = String::from_utf8(xor_encrypted)
        .unwrap()
        .replace("-", "+")
        .replace("_", "/")
        .replace("\0", "");
    let b64 = base64::decode(replaced.as_str()).unwrap();
    let mut decoder = gzip::Decoder::new(&b64[..]).unwrap();
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf).unwrap();

    //encrypt the ls
    //encrypting level string
    /*def encrypt(dls):
    fin = gzip.compress(dls)
    fin = base64.b64encode(fin)
    fin = fin.decode("utf-8").replace('+', '-').replace('/', '_')
    fin = 'H4sIAAAAAAAAC' + fin[13:]
    return(fin)*/

    //setting level string

    let mut reader = Reader::from_str(std::str::from_utf8(&buf).unwrap());
    reader.trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut buf = Vec::new();

    let mut k4_detected = false;
    let mut done = false;
    let mut k2_detected = false;

    //println!("{}", old_ls);

    let full_ls = old_ls + &ls;

    loop {
        match reader.read_event(&mut buf) {
            // unescape and decode the text event using the reader encoding
            Ok(Event::Text(e)) => {
                let text = e.unescape_and_decode(&reader).unwrap();
                if k4_detected {
                    let encrypted_ls: String = {
                        let mut ls_encoder = gzip::Encoder::new(Vec::new()).unwrap();
                        ls_encoder.write_all(&full_ls.as_bytes()).unwrap();
                        let b64_encrypted =
                            base64::encode(&ls_encoder.finish().into_result().unwrap());
                        let fin = b64_encrypted.replace("+", "-").replace("/", "_");
                        "H4sIAAAAAAAAC".to_string() + &fin[13..]
                    };

                    assert!(writer
                        .write_event(Event::Text(BytesText::from_plain_str(&encrypted_ls)))
                        .is_ok());
                    done = true;
                    k4_detected = false;
                } else {
                    assert!(writer.write_event(Event::Text(e)).is_ok())
                }

                if k2_detected {
                    println!("Writing to level: {}", text);
                    k2_detected = false;
                }

                if !done && text == "k4" {
                    k4_detected = true
                }

                if !done && text == "k2" {
                    k2_detected = true
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(e) => assert!(writer.write_event(e).is_ok()),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
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

    let encoded = base64::encode(&with_signature)
        .replace("+", "-")
        .replace("/", "_")
        .as_bytes()
        .to_vec();

    let fin = xor(encoded, 11);
    assert!(fs::write(path, fin).is_ok());
}*/
