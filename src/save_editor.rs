use anyhow::Result as AnyResult;
use base64;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use std::path::PathBuf;

use crate::cache::STSCache;

const ENCODING_KEY: &[u8] = b"key";

#[derive(Serialize, Deserialize, Debug)]
struct JsonCard {
    id: String,
    misc: u32,
    upgrades: u32,
}

fn encode(data: &[u8], key: &[u8]) -> Vec<u8> {
    let mut result = Vec::from(data);
    let key_len = key.len();
    result
        .iter_mut()
        .enumerate()
        .for_each(|(i, v)| *v ^= key[i % key_len]);
    result
}

pub fn get_save_file_path(folder: &PathBuf) -> Option<PathBuf> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() <= 1 {
        let mut save_file_names = Vec::with_capacity(5);
        for entry_result in std::fs::read_dir(&folder).ok()? {
            if let Ok(entry) = entry_result {
                if let Ok(file_data) = entry.metadata() {
                    if file_data.is_file() {
                        let filename = entry.path();
                        if let Some(extension) = filename.extension() {
                            if extension == "autosave" {
                                save_file_names.push(filename);
                            }
                        }
                    }
                }
            };
        }
        match save_file_names.len() {
            0 => None,
            1 => Some(folder.join(save_file_names.pop().unwrap())),
            _ => {
                println!("Found several save files, select one:");
                let mut buffer = String::with_capacity(5);
                let choice = loop {
                    buffer.clear();
                    for (index, file) in save_file_names.iter().enumerate() {
                        println!("{}: {:?}", index + 1, file);
                    }
                    std::io::stdin().read_line(&mut buffer).ok()?;
                    if let Ok(value) = usize::from_str_radix(buffer.trim(), 10) {
                        if value > 0 && value <= save_file_names.len() {
                            break value - 1;
                        }
                    }
                };
                Some(folder.join(save_file_names[choice].clone()))
            }
        }
    } else {
        let arg = PathBuf::from(args.last().unwrap());
        if arg.exists() {
            Some(arg)
        } else {
            let arg_filename = folder.join(arg);
            if arg_filename.exists() {
                Some(arg_filename)
            } else {
                None
            }
        }
    }
}

fn unpack_file(savefile_path: &PathBuf) -> AnyResult<JsonValue> {
    let contents = std::fs::read(&savefile_path)?;
    let config = base64::Config::new(base64::CharacterSet::Standard, true);
    let unbased_encoded = base64::decode_config(&contents, config)?;
    let unbased_decoded = encode(&unbased_encoded, ENCODING_KEY);
    let json_dict: JsonValue = serde_json::from_slice(&unbased_decoded)?;
    Ok(json_dict)
}

fn pack_file(value: JsonValue, filepath: &PathBuf) -> AnyResult<()> {
    let config = base64::Config::new(base64::CharacterSet::Standard, true);
    let json_string = value.to_string();
    let unbased_encoded = encode(&json_string.as_bytes(), ENCODING_KEY);
    let based_encoded = base64::encode_config(&unbased_encoded, config);
    std::fs::write(&filepath, based_encoded).map_err(anyhow::Error::msg)
}

fn print_status(json: &JsonValue) {
    println!("Gold: {} ({})", json["gold"], json["gold_gained"]);

    let cards = serde_json::from_value::<Vec<JsonCard>>(json["cards"].clone()).unwrap();
    let cards_ids: Vec<String> = cards.into_iter().map(|x| x.id).collect();
    println!("Cards: {:?}", cards_ids);

    let relics = serde_json::from_value::<Vec<String>>(json["relics"].clone()).unwrap();
    println!("Relics: {:?}", relics);

    println!("Select action ('q' to quit):");
    println!("g - Give 100 gold");
    println!("z - Remove all cards");
    println!("x - Give 10 random cards");
}

pub fn process_file(save_file: &PathBuf, cache: &STSCache) -> AnyResult<()> {
    let mut json_dict = unpack_file(save_file)?;
    let mut buffer = String::with_capacity(5);
    let mut rng = rand::thread_rng();
    loop {
        print_status(&json_dict);
        buffer.clear();
        std::io::stdin().read_line(&mut buffer)?;
        match buffer.trim() {
            "g" => {
                let g1 = serde_json::from_value::<u32>(json_dict["gold"].clone()).unwrap() + 100;
                json_dict["gold"] = JsonValue::from(g1);
                let g2 =
                    serde_json::from_value::<u32>(json_dict["gold_gained"].clone()).unwrap() + 100;
                json_dict["gold_gained"] = JsonValue::from(g2);
            }
            "z" => {
                json_dict["cards"] = serde_json::to_value(Vec::<JsonCard>::new()).unwrap();
            }
            "x" => {
                let mut all_cards =
                    serde_json::from_value::<Vec<JsonCard>>(json_dict["cards"].clone()).unwrap();
                for _ in 0..10 {
                    let random_card_id = rng.gen_range(0, cache.cards.len());
                    let random_card = JsonCard {
                        id: cache.cards[random_card_id].id.clone(),
                        misc: 0,
                        upgrades: 0,
                    };
                    all_cards.push(random_card);
                }
                json_dict["cards"] = serde_json::to_value(all_cards).unwrap();
            }
            "q" => break,
            _ => continue,
        }
    }
    pack_file(json_dict, save_file)
}
