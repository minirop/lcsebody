extern crate byteorder;
use byteorder::{ReadBytesExt, LittleEndian};

extern crate encoding_rs;
use encoding_rs::SHIFT_JIS;

use std::collections::HashMap;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Result;
use std::io::Read;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Not enough args");
    }

    let filename = args[1].clone();

    let mut decomp = false;
    if args.len() == 3 {
        if args[2] == "-d" {
            decomp = true;
        } else {
            panic!("unknown argument: {}", args[2]);
        }
    }

    let mut f = File::open(filename)?;
    let filesize = f.metadata()?.len();

    let count = f.read_u32::<LittleEndian>()?;
    //println!("count: {:?}", count);
    let _strings_space = f.read_u32::<LittleEndian>()?;
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

    if decomp {
        decompile(&mut f, count, &strings)?;
    } else {
        disassemble(&mut f, count, &strings)?;
    }

    Ok(())
}

fn disassemble(f: &mut File, count: u32, strings: &HashMap<u64, String>) -> Result<()> {
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
                    0x05 => print!("<complicated things>;"),
                    0x09 => print!("(this + 0x2984)[$STACK_TOP-1] = $STACK_TOP;"),
                    _ => print!("unknown"),
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
                    0x0E => println!("blocking wait"),
                    0x0F => println!("skippable wait"),
                    0x12 => println!("LoadLayer"),
                    0x13 => println!("LoadMaskLayer"),
                    0x14 => println!("SetLayerVisible"),
                    0x15 => println!("SetLayerPosition"),
                    0x16 => println!("CropLayer"),
                    0x17 => println!("SetLayerId"),
                    0x1A => println!("StartupGraphicsEffect"),
                    0x1E => println!("set font + size"),
                    0x26 => println!("set textbox size?"),
                    0x2C => println!("print dialogue"),
                    0x30 => println!("play audio"),
                    0x3A => println!("create dialogue box button"),
                    0x44 => println!("create menu"),
                    0x45 => println!("wait button click"),
                    0x4D => println!("show dialogue box (0: ok or 1: yes/no)"),
                    0x4F => println!("show choices"),
                    0x5A => println!("create blinking cursor"),
                    0x6A => println!("remove menu"),
                    _ => println!("unknown"),
                };
            },
            0x0E => println!("return from script"),
            0x0F => {
                match arg2 {
                    0x00 => println!("push_int((this + 0x296c)[{}]);", arg1),
                    0x02 => println!("push_bool((this + 0x2970)[{}]);", arg1),
                    0x06 => println!("push_int((this + 0x2978)[{}]);", arg1),
                    0x0C => println!("push_int((this + 0x2984)[{}]);", arg1),
                    0x0D => println!("push_int((this + 0x2984)[pop() + {}]);", arg1),
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
                print!("push_");
                match arg1 {
                    0x00 => println!("int {}", arg2),
                    0x01 => println!("bool {}", if arg2 == 1 { "true" } else { "false" }),
                    0x02 => println!("string {:?}", strings[&(arg2 as u64)]),
                    _ => panic!("unknown push: {:?}", arg1),
                };
            },
            0x14 | 0x15 => println!("noop"),
            _ => println!("unknown"),
        };
    }

    Ok(())
}

fn decompile(f: &mut File, count: u32, strings: &HashMap<u64, String>) -> Result<()> {
    let mut stack = vec![];
    let mut gotoes: Vec<u32> = vec![];

    for line in 0u32..count {
        let opcode = f.read_u32::<LittleEndian>()?;
        let arg1 = f.read_u32::<LittleEndian>()?;
        let arg2 = f.read_u32::<LittleEndian>()?;

        if gotoes.contains(&line) {
            println!("L{:#04x}:", line);
        }

        match opcode {
            0x00 => println!("destroy_window();"),
            0x01 => print_binop(&mut stack, "+"),
            0x02 => print_binop(&mut stack, "-"),
            0x03 => print_binop(&mut stack, "*"),
            0x04 => print_binop(&mut stack, "/"),
            0x05 => print_binop(&mut stack, "%"),
            0x06 => {
                let val = get_arg(&mut stack);
                stack.push(format!("-({})", val));
            },
            0x07 => {
                match arg1 {
                    0x00 => print_binop(&mut stack, "=="),
                    0x01 => print_binop(&mut stack, "!="),
                    0x02 => print_binop(&mut stack, "<"),
                    0x03 => print_binop(&mut stack, "<="),
                    0x04 => print_binop(&mut stack, ">"),
                    0x05 => print_binop(&mut stack, ">="),
                    _ => panic!("Unknown CMP arg: {}", arg1),
                };
            },
            0x08 => {
                gotoes.push(arg2);
                println!("goto L{:#04x};", arg2);
            },
            0x09 => {
                gotoes.push(arg2);
                println!("if true {{ goto L{:#04x}; }}", arg2);
            },
            0x0A => {
                gotoes.push(arg2);
                println!("if false {{ goto L{:#04x}; }}", arg2);
            },
            0x0B => {
                let value = get_arg(&mut stack);
                let index = get_arg(&mut stack);
                match arg1 {
                    0x03 => println!("DAT_0x296c[{:?}] = {:?};", index, value),
                    0x04 => println!("DAT_0x2970[{:?}] = ({:?} == 1);", index, value),
                    0x05 => println!("<complicated things>;"),
                    0x09 => println!("DAT_0x2984[{:?}] = ({:?} == 1);", index, value),
                    _ => println!("unknown_0x0B_{:#04x}({:?}, {:?});", arg1, index, value),
                };
            },
            0x0C => { stack.pop(); },
            0x0D => {
                match arg1 {
                    0x01 => print!("destroy_window("),
                    0x02 => print!("return("),
                    0x0A => print!("load("),
                    0x0B => print!("call("),
                    0x0C => print!("set_window_title("),
                    0x0E => print!("blocking_wait("),
                    0x0F => print!("wait("),
                    0x12 => print!("LoadLayer("),
                    0x13 => print!("LoadMaskLayer("),
                    0x14 => print!("SetLayerVisible("),
                    0x15 => print!("SetLayerPosition("),
                    0x16 => print!("CropLayer("),
                    0x17 => print!("SetLayerId("),
                    0x1A => print!("StartupGraphicsEffect("),
                    0x1E => print!("set_font_size?("),
                    0x26 => print!("set_textbox_size?("),
                    0x2C => print!("print_dialogue("),
                    0x30 => print!("play_audio("),
                    0x3A => print!("create_dialogue_box_button("),
                    0x44 => print!("create_menu("),
                    0x45 => print!("wait_button_click("),
                    0x4D => print!("show_dialogue_box("),
                    0x4F => print!("show_choices("),
                    0x5A => print!("create_blinking_cursor("),
                    0x6A => print!("close_menu("),
                    _ => print!("unknown_{:#04x}(", arg1),
                };

                print_args(&mut stack);

                println!(");");

                stack.push("0".to_string());
            },
            0x0E => println!("return();"),
            0x0F => {
                match arg2 {
                    0x00 => stack.push(format!("DAT_0x296c[{}]", arg1)),
                    0x02 => stack.push(format!("DAT_0x2970[{}]", arg1)),
                    0x06 => stack.push(format!("DAT_0x2978[{}]", arg1)),
                    0x0C => stack.push(format!("DAT_0x2984[{}]", arg1)),
                    0x0D => {
                        let val = get_arg(&mut stack);
                        stack.push(format!("DAT_0x2984[{} + {}]", val, arg1));
                    },
                    _ => stack.push(format!("UNKNOWN_0x0F_{}[{}]", arg2, arg1)),
                };
            },
            0x10 => {
                match arg2 {
                    0x00 | 0x02 | 0x04 | 0x06 | 0x08 | 0x0a | 0x0c => stack.push(format!("DAT_0x29{}[{}]", 0x6c + 4 * (arg2 / 2), arg1)),
                    0x01 | 0x03 | 0x05 | 0x07 | 0x09 | 0x0b | 0x0d => println!("stack[stack_top].type = {}; stack[stack_top].value += {};", (arg2 / 2) + 3, arg1),
                    _ => panic!("unknown 0x10"),
                };
            },
            0x11 => {
                match arg1 {
                    0x00 => stack.push(format!("{}", arg2)),
                    0x01 => stack.push(format!("{}", arg2 == 1)),
                    0x02 => stack.push(format!("{:?}", strings[&(arg2 as u64)])),
                    _ => panic!("unknown push: {:?}", arg1),
                };
            },
            0x14 | 0x15 => {},
            _ => println!("unknown_{:#04x}_{:#04x}_{:#04x}();", opcode, arg1, arg2),
        };
    }

    Ok(())
}

fn print_binop(stack: &mut Vec<String>, op: &str) {
    let left = get_arg(stack);
    let right = get_arg(stack);
    let ret = format!("{} {} {}", left, op, right);
    stack.push(ret);
}

fn get_arg(stack: &mut Vec<String>) -> String {
    stack.pop().unwrap()
}

fn print_args(stack: &mut Vec<String>) {
    let mut first = true;
    for val in stack.into_iter() {
        if !first {
            print!(", ");
        }
        print!("{}", val);

        first = false;
    }

    stack.clear();
}
