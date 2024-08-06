use std::{
    fs::{self, File},
    io::Read,
};

use dirs::home_dir;

#[derive(Clone, Debug)]
struct Chunk {
    bank: u8,
    data: Vec<u8>,
    name: String,
}

fn build_chunk(c_bank: u8, c_data: &Vec<u8>, c_name: String) -> Chunk {
    // .clone() just to be sure

    Chunk {
        bank: c_bank.clone(),
        data: c_data.clone(),
        name: c_name.clone(),
    }
}

fn deconstruct_tic(path: String) -> Vec<Chunk> {
    // reading the .tic file
    let mut f = File::open(String::from(path.clone())).expect("No file found!");

    // get file size
    let size: u64 = fs::metadata(path.clone()).expect("No file found").len();

    // creating a vector to store the bytes
    let mut buf = vec![0; size as usize];

    // put the bytes into the vector
    let _ = f.read_exact(&mut buf);

    let mut chunks: Vec<Chunk> = vec![];
    let mut check = 0;

    // static types are good

    let mut chunk_size: u16 = 0;
    let mut chunk_bank: u8 = 0;
    let mut chunk_type: &str = "";
    let mut chunk_data: Vec<u8> = vec![];

    for i in buf {
        // chunks follow the scheme of
        // type(5 bits) + bank(3 bits)

        chunk_type = match check {
            0 => match i & 0b00011111 {
                1 => "Tiles",
                2 => "Sprites",
                4 => "Map",
                5 => "Code",
                6 => "Flags",
                9 => "Samples",
                10 => "Waveform",
                12 => "Palette",
                14 => "Music",
                15 => "Patterns",
                17 => "Default",
                18 => "Screen",
                19 => "Binary",
                _ => "(Reserved)",
            },
            _ => chunk_type,
        };
        chunk_bank = match check {
            0 => i & 0b11100000,
            _ => chunk_bank,
        };

        // size(16 bits)

        chunk_size = match check {
            1 => i as u16,
            2 => chunk_size + ((i as u16) << 8),
            _ => chunk_size,
        };

        // reserved(8 bits)

        // actual data(size bits)

        if check == 4 {
            if chunk_size > 0 {
                chunk_size -= 1;
                chunk_data.push(i);
            } else {
                check = 0;
            }
        }

        // handle data insertion

        if check < 3 {
            // cycle state

            check += 1;
        } else if chunk_size == 0 {
            // reset state

            check = 0;

            // add chunk

            chunks.push(build_chunk(chunk_bank, &chunk_data, chunk_type.into()));
            chunk_data.clear();
        } else {
            // set state

            check = 4;
        }
    }

    chunks
}

fn extract(from: Vec<Chunk>, name: String) -> Chunk {
    for i in from {
        if i.name == name {
            return i;
        }
    }

    return Chunk {
        bank: 0,
        data: vec![],
        name: name,
    };
}

fn replace(from: Vec<Chunk>, what: Chunk) -> Vec<Chunk> {
    let mut new: Vec<Chunk> = vec![];

    let mut added: bool = false;

    for i in from {
        if i.name == what.name {
            new.push(what.clone());
            added = true;
        } else {
            new.push(i);
        }
    }

    if !added {
        new.push(what.clone());
    }

    new
}

fn find(from: Vec<Chunk>, name: String) -> bool {
    for i in from {
        if i.name == name {
            return true;
        }
    }

    false
}

fn main() {
    let mut path_to_conf = home_dir().unwrap();

    path_to_conf.push(".local/share/com.nesbox.tic/TIC-80/.local/b09c50c/config.tic");

    // reading the .tic file
    let chunks: Vec<Chunk> = deconstruct_tic(path_to_conf.to_str().unwrap().into());

    let glyphs: Chunk = extract(chunks, "Tiles".into());
    let sprites: Chunk = extract(chunks, "Sprites".into());

    println!("{:?}", glyphs);
}
