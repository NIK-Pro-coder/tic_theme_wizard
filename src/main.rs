use std::{
    fs::{self, File},
    io::Read,
};

use dirs::home_dir;

use macroquad::prelude::*;

const DEFAULT_PALETTE: [u8; 48] = [
    24, 26, 44, 93, 39, 93, 177, 62, 83, 239, 125, 87, 255, 205, 117, 167, 240, 112, 56, 183, 100,
    37, 113, 121, 41, 54, 111, 59, 93, 201, 65, 166, 246, 115, 239, 247, 244, 244, 244, 148, 176,
    194, 86, 108, 134, 51, 60, 87,
];
const PIX_SIZE: f32 = 4.0;

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

fn split_every(mut what: Vec<u8>, every: u8) -> Vec<Vec<u8>> {
    let mut tmp: Vec<u8> = vec![];
    let mut new: Vec<Vec<u8>> = vec![];

    for _i in what.clone() {
        match what.pop() {
            Some(elem) => tmp.push(elem),
            None => {}
        }
        if tmp.clone().len() >= every.into() {
            new.push(tmp.clone());
            tmp.clear();
        }
    }

    new.reverse();

    new
}

fn split_bytes(what: Vec<u8>) -> Vec<u8> {
    let mut new: Vec<u8> = vec![];

    for i in what.clone() {
        new.push(i & 0b00001111);
        new.push((i & 0b11110000) >> 4);
    }

    new
}

#[macroquad::main("Tic80 Theme Wizard")]
async fn main() {
    let mut path_to_conf = home_dir().unwrap();
    path_to_conf.push(".local/share/com.nesbox.tic/TIC-80/.local/b09c50c/config.tic");

    let mut chunks: Vec<Chunk> = deconstruct_tic(path_to_conf.to_str().unwrap().into());

    if find(chunks.clone(), "Default".into()) {
        chunks = replace(
            chunks.clone(),
            Chunk {
                bank: 0,
                data: DEFAULT_PALETTE.to_vec().clone(),
                name: "Palette".to_string(),
            },
        );
    }

    let glyphs: Chunk = extract(chunks.clone(), "Tiles".into());
    let split_glyphs: Vec<u8> = split_bytes(glyphs.data.clone());

    let sprites: Chunk = extract(chunks.clone(), "Sprites".into());
    let palette: Chunk = extract(chunks.clone(), "Palette".into());

    let mut colors: Vec<Color> = vec![];

    for i in split_every(palette.data.clone(), 3) {
        colors.push(color_u8!(i[2], i[1], i[0], 255))
    }

    let draw_spr = |spr: Vec<u8>, x: f32, y: f32, trans: u8| {
        for id in 0..spr.len() {
            let px = 7.0 - (id as f32 % 8.0);
            let py = 7.0 - (id as f32 / 8.0).floor();

            if spr[id] != trans {
                draw_rectangle(
                    px * PIX_SIZE + x,
                    py * PIX_SIZE + y,
                    PIX_SIZE,
                    PIX_SIZE,
                    colors[spr[id] as usize],
                );
            }
        }
    };

    let split_glyphs: Vec<Vec<u8>> = split_every(split_glyphs, 64);

    let draw_by_id = |id: usize, x: f32, y: f32, trans: u8| {
        draw_spr(split_glyphs[id].clone(), x, y, trans);
    };

    let draw_wrapping = |what: Vec<usize>, wrap: i32, x: f32, y: f32, trans: u8| {
        for id in 0..what.len() {
            let ox = (id as f32 % wrap as f32) * 8.0;
            let oy = (id as f32 / wrap as f32).floor() * 8.0;

            draw_by_id(what[id], (x + ox) * PIX_SIZE, (y + oy) * PIX_SIZE, trans);
        }
    };

    let create_draw = |what: Vec<usize>, wrap: i32, trans: u8| {
        return move |x: f32, y: f32| {
            draw_wrapping(what.clone(), wrap, x, y, trans);
        };
    };

    let mut left: Vec<usize> = (0..split_glyphs.len()).collect();

    macro_rules! draw_macro {
        ($vec_name:ident : $elem:expr => $func_name:ident, $wrap:expr) => {
            let $vec_name: Vec<usize> = $elem;
            let $func_name = create_draw($elem, $wrap, 0);

            for i in $elem.clone() {
                match left.iter().position(|x| *x == i) {
                    Some(n) => {
                        left.remove(n);
                    }
                    None => {}
                }
            }
        };
        ($vec_name:ident : $elem:expr => $func_name:ident, $wrap:expr, $trans:expr) => {
            let $vec_name: Vec<usize> = $elem;
            let $func_name = create_draw($elem, $wrap, $trans);

            for i in $elem.clone() {
                match left.iter().position(|x| *x == i) {
                    Some(n) => {
                        left.remove(n);
                    }
                    None => {}
                }
            }
        };
    }

    draw_macro!(tic_faces : vec![
        0, 1, 2, 3, 4, 5, 6, 7, 32, 33, 16, 17, 18, 19, 20, 21, 22, 23, 48, 49
        ] => tics, 10
    );

    draw_macro!(editor_vec : (88..=92).collect::<Vec<usize>>() => editor_tabs, 137);
    draw_macro!(control_vec : (80..=84).collect::<Vec<usize>>() => control_panel, 137);
    draw_macro!(bank_vec : (85..=86).collect::<Vec<usize>>() => bank_controls, 137);
    draw_macro!(code_vec : (96..=102).collect::<Vec<usize>>() => code_controls, 137);
    draw_macro!(map_vec : (103..=109).collect::<Vec<usize>>() => map_controls, 137);
    draw_macro!(music_vec : vec![114, 115, 116, 117, 118, 119, 120, 137] => music_controls, 137);
    draw_macro!(sprite_vec : (121..=136).collect::<Vec<usize>>() => sprite_controls, 137);

    draw_macro!(arrow_vec : vec![8, 9, 10, 11, 24, 25, 26, 27] => arrows, 4);
    draw_macro!(button_vec : vec![12, 13, 14, 15, 28, 29, 30, 31] => buttons, 4);

    draw_macro!(piano_keys_vec : vec![39, 40, 41, 42] => piano_keys, 4, 3);
    draw_macro!(channel_on_vec : vec![36, 52] => channel_on, 4);

    draw_macro!(big_button_vec : (144..=159).collect::<Vec<usize>>() => big_buttons, 2);
    draw_macro!(big_arrow_vec : vec![
            194, 195, 196, 197, 198, 199, 200, 201,
            210, 211, 212, 213, 214, 215, 216, 217,
            226, 227, 228, 229, 230, 231, 232, 233,
            242, 243, 244, 245, 246, 247, 248, 249,
        ] => big_arrows, 8, 16
    );
    draw_macro!(keyboard_vec : vec![160, 161, 162, 163, 176, 177, 178, 179] => keyboard, 4, 16);
    draw_macro!(enter_vec : vec![
            192, 206, 193,
            208, 220, 222,
            34,  221, 222,
            34,  208, 209,
            224, 238, 225,
            240, 252, 222,
            34,  221, 222,
            34,  240, 241
        ] => enter_key, 3
    );
    draw_macro!(shift_vec : vec![202, 203, 218, 219, 234, 235, 250, 251] => shift_key, 2, 16);

    draw_macro!(misc_vec : left.clone() => misc, 16);

    loop {
        clear_background(BLACK);

        misc(0.0, 0.0);

        next_frame().await;
    }
}
