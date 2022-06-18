use std::env;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

struct Operation {
    label: String,
    function: i32,
    arguments: Vec<String>
}

enum LineParseState {
    Label,
    Function,
    OpenArguments,
    Argument,
    EndState,
    InvalidState
}

fn load_code<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).expect("Failed to open the file!");
    let mut string = String::new();

    let _ = file.read_to_string(&mut string);

    return string;
}

fn split_lines(program: &String) -> (String, Vec<String>) {
    let mut lines = program.lines().collect::<Vec<_>>();
    let states = String::from(*lines.get(0).unwrap());
    lines.remove(0);
    
    return (states, lines.iter().map(|&x| x.into()).collect::<Vec<String>>());
}

/*
Functions: 
0.  Z(x)        -> Zero xth cell
1.  S(x)        -> Increment xth cell
2.  T(x, y)     -> Copy xth cell to yth cell
3.  I(x, y, z)  -> Jump to label z if values in xth and yth cells are the same
    
    label       -> [A-Za-z]+
    argument_n  -> [0-9]+
    argument_s  -> {label}
    arguments   -> ({argument_s|argument_n},)* {argument_s|argument_n}
    function    -> [SZTI]({arguments})
    operation   -> {label}: {function}
*/

fn parse_line(line_number: i32, line: &String) -> Result<Operation, String> {
    let mut parse_state = LineParseState::Label; 
    let mut operation: Operation = Operation{ label: String::from(""), function: -1, arguments: vec![] };

    let mut argument_count = 0;
    let mut current_argument = 0;

    for (index, c) in line.chars().enumerate() {
        match parse_state {
            LineParseState::Label => {
                match c {
                    ':' => {
                        parse_state = LineParseState::Function;
                    },
                    ' ' => {
                        continue;
                    },
                    'A'..='z' | '0'..='9' | '.' => {
                        operation.label.push(c);
                    },
                    _ => {
                        parse_state = LineParseState::InvalidState;  
                    }
                }
            },
            LineParseState::Function => {
                match c {
                    'Z' => {
                        parse_state = LineParseState::OpenArguments;
                        operation.function = 0;
                        argument_count = 1;
                        operation.arguments = vec![String::from("")];
                    },
                    'S' => {
                        parse_state = LineParseState::OpenArguments;
                        operation.function = 1;
                        argument_count = 1;
                        operation.arguments = vec![String::from("")];
                    },
                    'T' => {
                        parse_state = LineParseState::OpenArguments;
                        operation.function = 2;
                        argument_count = 2;
                        operation.arguments = vec![String::from(""), String::from("")];
                    },
                    'I' => {
                        parse_state = LineParseState::OpenArguments;
                        operation.function = 3;
                        argument_count = 3;
                        operation.arguments = vec![String::from(""), String::from(""), String::from("")];
                    },
                    ' ' => {
                        continue;
                    },
                    _ => {
                        parse_state = LineParseState::InvalidState;
                    }
                }
            },
            LineParseState::OpenArguments => {
                match c {
                    '(' => {
                        parse_state = LineParseState::Argument;  
                    },
                    ' ' => {
                        continue;
                    },
                    _ => {
                        parse_state = LineParseState::InvalidState;
                    }
                }
            },
            LineParseState::Argument => {
                if current_argument < 2 {
                    match c {
                        '0'..='9' => {
                            operation.arguments[current_argument].push(c);
                        }
                        ',' => {
                            if current_argument + 1 < argument_count && operation.arguments[current_argument].len() > 0 {
                                current_argument += 1;
                            }
                            else {
                                parse_state = LineParseState::InvalidState;
                            }
                        },
                        ')' => {
                            if current_argument == argument_count - 1 {
                                parse_state = LineParseState::EndState;
                            } 
                            else {
                                parse_state = LineParseState::InvalidState;
                            }
                        },
                        ' ' => {
                            continue;
                        }
                        _ => {
                            parse_state = LineParseState::InvalidState;
                        }
                    }
                }
                else {
                    match c {
                        'A'..='z' | '0'..='9' | '.' => {
                            operation.arguments[current_argument].push(c);
                        },
                        ')' => {
                            if operation.arguments[current_argument].len() > 0 {
                                parse_state = LineParseState::EndState;
                            }
                            else {
                                parse_state = LineParseState::InvalidState;
                            }
                        },
                        ' ' => {
                            continue;
                        },
                        _ => {
                            parse_state = LineParseState::InvalidState;
                        }
                    }
                }
            },
            LineParseState::EndState => {
                break;
            }
            LineParseState::InvalidState => {
                println!("Line {}:{} -> Parser went into invalid state: ", line_number, index);
                println!("{}\n{}^\n{}here", line, " ".repeat(index - 1), " ".repeat(index - 1));
                return Err(String::from("Parser went into invalid state"));
            }
        }
    }

    match parse_state {
        LineParseState::EndState => {}
        _ => {
            println!("Line {}:{} -> Parser went into invalid state: ", line_number, line.len() - 1);
            println!("{}\n{}^\n{}here", line, " ".repeat(line.len() - 1), " ".repeat(line.len() - 1));
            return Err(String::from("Parser went into invalid state"));
        }
    }

    return Ok(operation);
}

fn main() -> std::io::Result<()> {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let program = load_code(args.get(0).expect("Failed to open the file!"));
    let (output, lines) = split_lines(&program);

    args.remove(0);

    let mut operations: Vec<Operation> = vec![];
    let mut label_index: HashMap<String, u32> = HashMap::new();

    for (index, line) in lines.iter().enumerate() {
        let operation = parse_line(index as i32, &line).unwrap();

        if label_index.contains_key(&operation.label) {
            return Err(Error::new(ErrorKind::Other, format!("Line {}: Label redefinition", index + 1)));
        }
        else {
            label_index.insert(operation.label.clone(), index as u32);
        }

        operations.push(operation);
    }

    let mut cells: Vec<u128> = args.iter().map(|x| x.parse::<u128>().unwrap()).collect::<Vec<u128>>();

    for (index, operation) in operations.iter().enumerate() {
        if operation.function == 3 && !label_index.contains_key(operation.arguments.get(2).unwrap()) {
            return Err(Error::new(ErrorKind::Other, format!("Line {}: Label '{}' not defined", index + 1, operation.arguments.get(2).unwrap())));
        }

        match operation.function {
            0 | 1 => {
                let x = operation.arguments.get(0).unwrap().parse::<usize>().unwrap();

                if cells.len() <= x {
                    cells.resize(x + 1, 0);
                }
            },
            2 | 3 => {
                let x = operation.arguments.get(0).unwrap().parse::<usize>().unwrap();
                let y = operation.arguments.get(1).unwrap().parse::<usize>().unwrap();

                if cells.len() <= x {
                    cells.resize(x + 1, 0);
                }

                if cells.len() <= y {
                    cells.resize(y + 1, 0);
                }
            },
            _ => {

            }
        }
    }

    let mut index = 0;
    while index < operations.len()
    {
        let op = operations.get(index).unwrap();

        match op.function {
            0 => {
                let x = op.arguments.get(0).unwrap().parse::<usize>().unwrap();
                
                let cell = cells.get_mut(x).unwrap();
                *cell = 0;

                index += 1;
            },
            1 => {
                let x = op.arguments.get(0).unwrap().parse::<usize>().unwrap();

                let cell = cells.get_mut(x).unwrap();
                *cell += 1;

                index += 1;
            },
            2 => {
                let x = op.arguments.get(0).unwrap().parse::<usize>().unwrap();
                let y = op.arguments.get(1).unwrap().parse::<usize>().unwrap();

                let cell_x_value = *cells.get(x).unwrap();
                let cell_y = cells.get_mut(y).unwrap();

                *cell_y = cell_x_value;

                index += 1;
            },
            3 => {
                let x = op.arguments.get(0).unwrap().parse::<usize>().unwrap();
                let y = op.arguments.get(1).unwrap().parse::<usize>().unwrap();
                let z = op.arguments.get(2).unwrap();

                let cell_x = cells.get(x).unwrap();
                let cell_y = cells.get(y).unwrap();

                if *cell_x == *cell_y {
                    index = (*label_index.get(z).unwrap()) as usize;
                }
                else {
                    index += 1;
                }
            }
            _ => {

            }
        }
    }

    if output == "_"
    {
        println!("{:?}", cells);
    }
    else
    {
        let index = output.parse::<usize>().unwrap();
        if cells.len() <= index {
            cells.resize(index + 1, 0);
        }
        println!("{}", *cells.get(index).unwrap());
    }

    return Ok(());
}