use nvgpu::GpFifoEntry;

use std::env;
use std::fs::File;
use std::io::Read;

use std::collections::BTreeMap;

#[derive(Debug)]
struct GpFifoDecoder {
    raw_entry: GpFifoEntry,
    arguments: BTreeMap<u32, Option<u32>>,
    next_index: u32
}

impl GpFifoDecoder {
    pub fn new(entry: u32) -> Self {
        let mut res = GpFifoDecoder {
            raw_entry: GpFifoEntry(entry),
            arguments: BTreeMap::new(),
            next_index: 0
        };

        let args_range = 0..Self::arguments_count(&res.raw_entry);

        for i in args_range.into_iter() {
            res.arguments.insert(i as u32, None);
        }

        if res.raw_entry.submission_mode() == 4 {
            res.arguments.insert(0, Some(res.raw_entry.inline_arguments()));
            res.next_index += 1;
        }

        res
    }

    pub fn push_argument(&mut self, argument: Option<u32>) {
        assert!(!self.is_complete());

        self.arguments.insert(self.next_index, argument);

        self.next_index += 1;
    }

    pub fn is_complete(&self) -> bool {
        self.raw_entry.submission_mode() == 4 || self.next_index == self.raw_entry.argument_count()
    }

    pub fn arguments_count(entry: &GpFifoEntry) -> usize {
        if entry.submission_mode() == 4 {
            1
        } else {
            entry.argument_count() as usize
        }
    }

    pub fn to_method(raw_value: u32) -> String {
        let mut res = Vec::new();
        let entry = GpFifoEntry(raw_value);
        let args_range = 0..Self::arguments_count(&entry);

        let arguments_list: Vec<String> = args_range.map(|value| format!("uint32_t arg{}", value)).collect();

        let arguments_string = arguments_list.join(", ");

        let submission_mode_str = match entry.submission_mode() {
            0 => "IncreasingOld",
            1 => "Increasing",
            2 => "NonIncreasingOld",
            3 => "NonIncreasing",
            4 => "Inline",
            5 => "IncreasingOnce",
            _ => unimplemented!()
        };

        res.push(format!("// Submission Mode: {}, Sub Channel Id: {}, envytools offset: 0x{:04x}\n", submission_mode_str, entry.sub_channel(), entry.method() * 4));
        res.push(format!("void method_{:x}(", entry.method()));
        res.push(arguments_string);
        res.push(String::from(")\n"));
        res.push(String::from("{\n"));

        let mut argument_offset = entry.method();

        for i in (0..Self::arguments_count(&entry)).into_iter() {
            res.push(format!("    REGISTERS[0x{:x}] = arg{};\n", argument_offset, i));

            if entry.submission_mode() == 0 || entry.submission_mode() == 1 || (entry.submission_mode() == 5 && i == 0) {
                argument_offset += 1;
            }
        }

        res.push(String::from("}\n"));

        res.iter().flat_map(|s| s.chars()).collect()
    }

    pub fn to_method_call(&self) -> String {
        let mut res = Vec::new();

        res.push(format!("method_{:x}(", self.raw_entry.method()));

        let arguments_list: Vec<String> = self.arguments.iter().map(|(_, value)| {
            if let Some(value) = value {
                format!("0x{:x}", value)
            } else {
                String::from("???")
            }
        }).collect();

        let arguments_string = arguments_list.join(", ");
        res.push(arguments_string);
        res.push(String::from(");\n"));
        
        res.iter().flat_map(|s| s.chars()).collect()
    }
}

fn main() {

    if env::args().len() < 2 {
        let app_name = env::args().nth(0).unwrap();
        println!("usage: {} cmds.txt", app_name);
        std::process::exit(1);
    }

    let path = env::args().nth(1).unwrap();
    let mut file = File::open(path).expect("File not found");

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let mut known_methods = Vec::new();
    let mut method_calls = Vec::new();
    let mut current_entry = None;

    for line in content.lines() {
        let value = u32::from_str_radix(line.trim_start_matches("0x"), 16).ok();

        if current_entry.is_none() {
            if value.is_none() {
               continue; 
            }

            let value = value.unwrap();

            if !known_methods.contains(&value) {
                known_methods.push(value);
            }

            let entry = GpFifoDecoder::new(value);

            if !entry.is_complete() {
                current_entry = Some(entry);
            } else {
                method_calls.push(entry);
            }

        } else {
            let mut entry = current_entry.take().unwrap();

            entry.push_argument(value);

            if !entry.is_complete() {
                current_entry = Some(entry);
            } else {
                method_calls.push(entry);
            }
        }
    }

    // Add incomplete method if data is missing
    if let Some(entry) = current_entry {
        method_calls.push(entry);
    }

    for method in known_methods {
        println!("{}", GpFifoDecoder::to_method(method));
    }

    println!("// Start method calls");

    for method_call in method_calls {
        println!("{}", method_call.to_method_call());
    }

}