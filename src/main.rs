use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs as fileRead;
use std::io::Error;

static JUMP_TABLE: Lazy<HashMap<&str, u16>> = Lazy::new(|| {
    HashMap::from([
        ("JGT", 0b001),
        ("JEQ", 0b010),
        ("JGE", 0b011),
        ("JLT", 0b100),
        ("JNE", 0b101),
        ("JLE", 0b110),
    ])
});

static SRC_TABLE: Lazy<HashMap<&str, u16>> = Lazy::new(|| {
    HashMap::from([
        ("0", 0b0101010),
        ("1", 0b0111111),
        ("-1", 0b0111010),
        ("D", 0b0001100),
        ("A", 0b0110000),
        ("(A)", 0b1110000),
    ])
});

static TARGET_TABLE: Lazy<HashMap<&str, u16>> =
    Lazy::new(|| HashMap::from([("D", 0b010), ("A", 0b100)]));

fn read_file_in(filename: &str) -> Result<Vec<String>, Error> {
    let contents: String = fileRead::read_to_string(filename)?;
    let lines: Vec<String> = contents
        .lines()
        .map(|line: &str| line.trim().to_uppercase().to_string())
        .filter(|line: &String| !line.is_empty())
        .collect();
    Ok(lines)
}

fn compare_length(rest_length: usize, expected_length: usize) {
    if rest_length != expected_length {
        panic!(
            "Error: Expected {} parts, but got {} parts.",
            expected_length, rest_length
        );
    }
}

fn compare_equal_strings(a: &str, b: &str) {
    if a != b {
        panic!("Error: Expected '{}' but got '{}'.", b, a);
    }
}

fn process_add_or_sub_instruction(rest: &[String], instruction_code: u16) -> u16 {
    compare_length(rest.len(), 5);

    let target: u16 = match TARGET_TABLE.get(&rest[0].as_str()) {
        Some(&val) => val,
        None => panic!("Error: Unknown target '{}'", rest[0]),
    };

    // Check that there is a comma in the instruction
    compare_equal_strings(&rest[1], ",");
    compare_equal_strings(rest[2].as_str(), "D");
    compare_equal_strings(&rest[3], ",");

    let access: u16 = match rest[4].as_str() {
        "A" => 0b0,
        "(A)" => 0b1,
        _ => panic!("Error: Expected 'A' or '(A)' but got '{}'.", rest[4]),
    };

    let binary_instruction: u16 =
        0b1110000000000000 | (access << 12) | (instruction_code << 7) | (target << 3);

    println!("Instruction binary: {:016b}", binary_instruction);
    binary_instruction // Return the constructed binary instruction
}

fn process_ldr_instructions(rest: &[String]) -> u16 {
    compare_length(rest.len(), 3);

    if rest[0] == "A" && rest[1] == "," && rest[2].starts_with('$') {
        let number_str: &str = &rest[2][1..]; // Skip the '$'
        println!("Immediate value: {}", number_str);
        if let Ok(number) = number_str.parse::<u16>() {
            if number <= 32767 {
                let binary_instruction = 0b0000000000000000 | (number & 0x7FFF);
                return binary_instruction;
            }
        }
    }

    let target: u16 = match TARGET_TABLE.get(&rest[0].as_str()) {
        Some(&val) => val,
        None => panic!("Error: Unknown target '{}'", rest[0]),
    };

    compare_equal_strings(&rest[1], ",");

    let src: u16 = match SRC_TABLE.get(&rest[2].as_str()) {
        Some(&val) => val,
        None => panic!("Error: Unknown source '{}'", rest[2]),
    };

    let binary_instruction: u16 = 0b1110000000000000 | (src << 6) | (target << 3);
    println!("LDR instruction binary: {:016b}", binary_instruction);
    binary_instruction // Return the constructed binary instruction
}

fn process_str_instruction(rest: &[String]) -> u16 {
    compare_length(rest.len(), 3);
    compare_equal_strings(&rest[0], "(A)");
    compare_equal_strings(&rest[1], ",");

    let src: u16 = match SRC_TABLE.get(&rest[2].as_str()) {
        Some(&val) => val,
        None => panic!("Error: Unknown source '{}'", rest[2]),
    };

    let binary_instruction: u16 = 0b1110000000000000 | (src << 6) | 0b001000;
    println!("STR instruction binary: {:016b}", binary_instruction);
    binary_instruction // Return the constructed binary instruction
}

fn process_jump_instruction(instruction: &str, rest: &[String]) -> u16 {
    let jump: u16 = match JUMP_TABLE.get(&instruction) {
        Some(&val) => val,
        None => panic!("Error: Unknown jump instruction '{}'", instruction),
    };

    compare_length(rest.len(), 1);

    let source: u16 = match SRC_TABLE.get(&rest[0].as_str()) {
        Some(&val) => val,
        None => panic!("Error: Unknown source '{}'", rest[0]),
    };

    let binary_instruction: u16 = 0b1110000000000000 | (source << 6) | (jump << 0);
    println!("Jump instruction binary: {:016b}", binary_instruction);
    binary_instruction // Return the constructed binary instruction
}

fn go_through_lines(lines: Vec<String>) {
    let mut list_of_instructions: Vec<u16> = Vec::new();

    for line in lines {
        let split_line: Vec<String> = line
            .replace(",", " , ")
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        match split_line.get(0).map(|s| s.as_str()) {
            Some("ADD") => {
                list_of_instructions
                    .push(process_add_or_sub_instruction(&split_line[1..], 0b00010));
                println!("Processing ADD: {:?}", split_line);
            }
            Some("SUB") => {
                list_of_instructions
                    .push(process_add_or_sub_instruction(&split_line[1..], 0b01011));
                println!("Processing SUB: {:?}", split_line);
            }
            Some("JMP") => {
                // Process JMP instruction
                list_of_instructions.push(0b1110101010000111); // Example binary for JMP
                println!("Binary for JMP: {:016b}", 0b1110101010000111);
            }
            Some("STR") => {
                // Process STR instruction
                list_of_instructions.push(process_str_instruction(&split_line[1..]));
                println!("Processing STR: {:?}", split_line);
            }
            Some("LDR") => {
                let instruction: u16 = process_ldr_instructions(&split_line[1..]);
                println!("Binary for LDR: {:016b}", instruction);
                list_of_instructions.push(instruction);
                println!("Processing LDR: {:?}", split_line);
            }
            Some("JGT") | Some("JEQ") | Some("JGE") | Some("JLT") | Some("JNE") | Some("JLE") => {
                process_jump_instruction(&split_line[0].as_str(), &split_line[1..]);
            }
            _ => {
                // Handle unknown or unsupported instructions
                println!("Unknown instruction: {:?}", split_line);
            }
        }
    }
}

fn main() {
    let filename: &str = "./testcases/test5.nha";

    // Attempt to read and process the file
    let lines: Vec<String> = match read_file_in(filename) {
        Ok(lines) => lines, // If successful, use the processed lines
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e); // Print error if reading fails
            return; // Exit the function early
        }
    };

    go_through_lines(lines);
}
