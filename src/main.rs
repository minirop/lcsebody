extern crate byteorder;
use byteorder::{ReadBytesExt, LittleEndian};

extern crate encoding_rs;
use encoding_rs::SHIFT_JIS;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Result;
use std::io::Read;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut filename = String::from("scripts/ANI");

    if args.len() == 2 {
        filename = args[1].clone();
    }

    let mut f = File::open(filename)?;
    let filesize = f.metadata()?.len();

    let count = f.read_i32::<LittleEndian>()?;
    //println!("count: {:?}", count);
    let _ = f.read_i32::<LittleEndian>()?;
    //println!("unknown: {:?}", unknown);

    let start = f.stream_position()?;

    f.seek(SeekFrom::Current((count*12) as i64))?;

    // read strings
    let mut strings = HashMap::new();

    let strings_start = f.stream_position()?;
    while f.stream_position()? < filesize {
        let string_start = f.stream_position()?;
        let len = f.read_u32::<LittleEndian>()?;

        let mut buffer = vec![0; len as usize];
        f.read(&mut buffer)?;

        let (res, _enc, _errors) = SHIFT_JIS.decode(&buffer);
        let string = res.to_string().trim_matches(char::from(0)).to_string();

        strings.insert(string_start - strings_start, string);
    }

    f.seek(SeekFrom::Start(start))?;

    let mut set = HashSet::new();

    for _ in 0..count {
        let opcode = f.read_u32::<LittleEndian>()?;
        let arg1 = f.read_u32::<LittleEndian>()?;
        let arg2 = f.read_u32::<LittleEndian>()?;

        if opcode == 0x11 {
            print!("push");
            match arg1 {
                0x00 => println!("_int {}", arg2),
                0x01 => println!("_bool {}", if arg2 == 1 { "true" } else { "false" }),
                0x02 => println!("_string {:?}", strings[&(arg2 as u64)]),
                _ => panic!("unknown push: {:?}", arg1),
            };
            set.insert(arg2);
        } else {
            println!("{:#04x} {:#04x} {:?}", opcode, arg1, arg2);
        }
    }

    Ok(())
}
