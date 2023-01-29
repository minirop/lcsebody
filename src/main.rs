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
    //println!("space used by strings: {:?}", strings_space);

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

        print!("{:#04x} {:#04x} {:#04x}: ", opcode, arg1, arg2);
        match opcode {
            0x00 => println!("destroy window"),
            0x01 => println!("add"),
            0x02 => println!("sub"),
            0x03 => println!("mul"),
            0x04 => println!("div"),
            0x05 => println!("mod"),
            0x06 => println!("neg"),
            0x07 => {
                match arg1 {
                    0x00 => println!("EQ"),
                    0x01 => println!("NE"),
                    0x02 => println!("LT"),
                    0x03 => println!("LE"),
                    0x04 => println!("GT"),
                    0x05 => println!("GE"),
                    _ => panic!("Unknown CMP arg: {}", arg1),
                };
            },
            0x08 => println!("jump to {:?}", arg2),
            0x09 => println!("jump if true to {:?}", arg2),
            0x0A => println!("jump if false to {:?}", arg2),
            0x0B => {
                match arg1 {
                    0x03 => print!("(this + 0x296c)[$STACK_TOP-1] = $STACK_TOP;"),
                    0x04 => print!("(this + 0x2970)[$STACK_TOP-1] = ($STACK_TOP == 1);"),
                    0x09 => print!("(this + 0x2984)[$STACK_TOP-1] = $STACK_TOP;"),
                    _ => print!("<complicated things>;"),
                };
                println!(" push_int(1);");
            },
            0x0C => println!("pop"),
            0x0D => {
                match arg1 {
                    0x01 => println!("destroy window"),
                    0x02 => println!("return from script"),
                    0x0A => println!("load script"),
                    0x0B => println!("call script"),
                    0x0C => println!("set window title"),
                    0x12 => println!("LoadLayer"),
                    0x13 => println!("LoadMaskLayer"),
                    0x14 => println!("SetLayerVisible"),
                    0x15 => println!("SetLayerPosition"),
                    0x16 => println!("CropLayer"),
                    0x1A => println!("StartupGraphicsEffect"),
                    0x2C => println!("print dialogue"),
                    0x4F => println!("show choices"),
                    _ => println!("unknown"),
                };
            },
            0x0E => println!("return from script"),
            0x0F => {
                match arg2 {
                    0x00 => println!("push_int((this + 0x296c)[{}]);", arg1),
                    0x06 => println!("push_int((this + 0x2978)[{}]);", arg1),
                    0x0C => println!("push_int((this + 0x2984)[{}]);", arg1),
                    _ => println!("unknown"),
                };
            },
            0x10 => {
                match arg2 {
                    0x00 | 0x02 | 0x04 | 0x06 | 0x08 | 0x0a | 0x0c => println!("push_var({}, {});", (arg2 / 2) + 3, arg1),
                    0x01 | 0x03 | 0x05 | 0x07 | 0x09 | 0x0b | 0x0d => println!("stack[stack_top].type = {}; stack[stack_top].value += {};", (arg2 / 2) + 3, arg1),
                    _ => println!("unknown"),
                };
            },
            0x11 => {
                print!("push");
                match arg1 {
                    0x00 => println!("_int {}", arg2),
                    0x01 => println!("_bool {}", if arg2 == 1 { "true" } else { "false" }),
                    0x02 => println!("_string {:?}", strings[&(arg2 as u64)]),
                    _ => panic!("unknown push: {:?}", arg1),
                };
                set.insert(arg2);
            },
            0x14 | 0x15 => println!("noop"),
            _ => println!("unknown"),
        };
    }

    Ok(())
}
