/*  _ _ _
 * | (_) |__         __ _ _ __ __ _ _ __   __ _ _ __ ___  ___   _ __ ___
 * | | | '_ \ _____ / _` | '__/ _` | '_ \ / _` | '__/ __|/ _ \ | '__/ __|
 * | | | |_) |_____| (_| | | | (_| | |_) | (_| | |  \__ \  __/_| |  \__ \
 * |_|_|_.__/       \__,_|_|  \__, | .__/ \__,_|_|  |___/\___(_)_|  |___/
 *                            |___/|_|
 * A minimal blazingly fast Argument Parsing library for modern Rust
 * https://github.com/NewDawn0/argparse.rs
 * 
 * Author: NewDawn0
 * Contibutors: -
 * License: MIT
 * Language: Rust
 * Version: 1.0.1
 *
 *
 * LICENSE:
 * Copyright (c) 2022 NewDawn0
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 * 
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
*/

//////////// Setup ////////////
/* === Crate level macro rules === */
#![allow(private_in_public)]
/* === Libraries === */
use std::{
    fmt::{self, Display},
    collections::{HashMap, HashSet},
    any::{Any, TypeId},
    env::args
};
/* === Constants === */
/* Argument options bitfield of u8 */
pub const ARG_DEFAULT: u8 = 0x00;       // 0b00000001
pub const ARG_REQUIRED: u8 = 0x01;      // 0b00000001
pub const ARG_MULTIPLE: u8 = 0x02;      // 0b00000010
pub const ARG_REQUIRES_NEXT: u8 = 0x04; // 0b00000100

const RESET: &'static str = "\x1b[0m";
const BOLD: &'static str = "\x1b[1m";
const CYAN: &'static str = "\x1b[0;36m";
const BLUE: &'static str = "\x1b[0;34m";
const PURPLE: &'static str = "\x1b[0;35m";
const YELLOW: &'static str = "\x1b[1;33m";
const RED: &'static str = "\x1b[1;31m";

pub struct ArgParser<'a> {
    args: HashMap<&'a str, Argument>,
    print_arg_vecs: Vec<Vec<&'a str>>,
    modfiable_args: Vec<&'a str>,
    required_args: HashSet<&'a str>,
    p_info: PInfo<'a>,
    pub other_args: Vec<String>,
    pub help: String,
    pub settings: Settings<'a>,
}

impl <'a>ArgParser<'a> {
    pub fn new(program_name: &'a str, program_version: &'a str) -> ArgParser<'a> {
        ArgParser {
            args: HashMap::<&'a str, Argument>::new(),
            print_arg_vecs: Vec::<Vec<&'a str>>::new(),
            modfiable_args: Vec::<&'a str>::new(),
            required_args: HashSet::<&'a str>::new(),
            p_info: PInfo::new(program_name, program_version),
            other_args: Vec::<String>::new(),
            help: String::new(),
            settings: Settings::new(),
        }
    }
    pub fn add_arg(&mut self, flags: &[&'static str], options: u8) -> &mut Self {
        self.print_arg_vecs.push(flags.to_vec());
        self.modfiable_args.clear();
        for flag in flags {
            if self.args.contains_key(flag) { 
                panic!("Argument was already added")
            }
            self.args.insert(flag, Argument::new(options));
            self.modfiable_args.push(flag);
            if (options & ARG_REQUIRED) != 0 { self.required_args.insert(flag); println!("\nadded {} to required", flag);}
        }
        self
    }
    pub fn help(&mut self, description: String) -> &mut Self {
        if self.modfiable_args.is_empty() { panic!("No arguments to add a description to"); }
        for flag in &self.modfiable_args {
            if let Some(arg) = self.args.get_mut(flag) {
                arg.description = description.clone();
            }
        }
        self
    }
    pub fn default_value<T: 'static + Clone>(&mut self, value: T) -> &mut Self {
        if self.modfiable_args.is_empty() { panic!("No arguments to set a default value") }
        let type_id = TypeId::of::<T>();
        for flag in &self.modfiable_args {
            if let Some(arg) = self.args.get_mut(flag) {
                arg.default_value = Some(Box::new(value.clone()));
                match arg.expected_type {
                    Some(exp_type) => {
                        if exp_type != type_id {
                            panic!("Expected type does not match");
                        }
                    },
                    None => arg.expected_type = Some(type_id)
                }
            }
        }
        self
    }
    pub fn expected_type<T: 'static>(&mut self) -> &mut Self {
        if self.modfiable_args.is_empty() { panic!("No arguments to set a set an expected type") }
        let type_id = TypeId::of::<T>();
        for flag in &self.modfiable_args {
            if let Some(arg) = self.args.get_mut(flag) {
                match arg.expected_type {
                    Some(t) => if t != type_id { panic!("Cannot change type") },
                    None => arg.expected_type = Some(type_id)
                }
            }
        }
        self
    }
    pub fn parse(&mut self) -> Result<(), ParserError> {
        // Setup
        let mut err: Option<ParserError> = None;
        let argv: Vec<String> = args().skip(1).collect();
        if self.settings.generate_default_flags { // generate default opts [-v ,--version, -h, --help]
            self.gen_default_args(&["-v", "--version"], ARG_MULTIPLE);
            self.gen_default_args(&["-h", "--help"], ARG_DEFAULT);
        }
        if self.settings.generate_help_menu { self.gen_help() } // generate the default help menu
        // parsing
        if argv.len() == 0 {
            return match self.settings.allow_no_args {
                true => Ok(()),
                false => Err(ParserError::NoArgs)
            }
        } else {
            let mut skip: bool = false;
            let mut checked_args = HashSet::<&str>::new();
            for (k, val) in argv.iter().enumerate() {
                println!("{}", val);
                if skip { skip = false; continue; }
                match self.args.get_mut(val.as_str()) {
                    Some(arg) => {
                        if checked_args.contains(val.as_str()) {
                            if arg.options & ARG_MULTIPLE != 0 {
                                if arg.options & ARG_REQUIRES_NEXT != 0 {
                                    if self.required_args.contains(val.as_str()) {
                                        self.required_args.remove(val.as_str());
                                    }
                                    match k+2 <= argv.len() {
                                        false => return Err(ParserError::IndexOutOfBound),
                                        true => {
                                            arg.values.push(Box::new(argv[k+1].clone()));
                                            skip = true
                                        }
                                    }
                                } else {
                                    arg.values.push(Box::new(true));
                                    if self.required_args.contains(val.as_str()) {
                                        self.required_args.remove(val.as_str());
                                    }
                                }
                            } else {
                                return Err(ParserError::DuplicateArg)
                            }
                        } else {
                            checked_args.insert(val.as_str());
                            if arg.options & ARG_REQUIRES_NEXT != 0 {
                                if self.required_args.contains(val.as_str()) {
                                    self.required_args.remove(val.as_str());
                                }
                                match k+2 <= argv.len() {
                                    false => return Err(ParserError::IndexOutOfBound),
                                    true => {
                                        arg.values.push(Box::new(argv[k+1].clone()));
                                        skip = true
                                    }
                                }
                            } else {
                                arg.values.push(Box::new(true));
                                if self.required_args.contains(val.as_str()) {
                                    self.required_args.remove(val.as_str());
                                }
                            }
                        }
                    },
                    None => { // Invalid arguments
                        match self.settings.allow_invalid_args {
                            true => self.other_args.push(val.to_string()),
                            false => {
                                err = Some(ParserError::InvalidArg);
                                self.other_args.push(val.to_string())
                            }
                        }
                    },
                }
            }
        }
        if !self.required_args.is_empty() { return Err(ParserError::RequirementNotMet) }
        match err {
            Some(e) => Err(e),
            None => Ok(())
        }
    }
    fn gen_default_args(&mut self, default_args: &[&'a str], options: u8) {
        let mut insert_index: Option<usize> = None;
        let mut to_insert: Vec<&str> = Vec::new();
        for elem in default_args {
            match self.args.get(elem) {
                Some(_) => {
                    if let Some(index) = find_2d(self.print_arg_vecs.clone(), elem) {
                        insert_index = Some(index)
                    }
                },
                None => {
                    self.args.insert(elem, Argument::new(options));
                    to_insert.push(elem)
                },
            }
        }
        match insert_index {
            Some(ii) => {
                if let Some(inner_vec) = self.print_arg_vecs.get_mut(ii) {
                    inner_vec.append(&mut to_insert)
                }
            }
            None => self.print_arg_vecs.push(to_insert)
        } 
    }
    pub fn gen_help(&mut self) {
        let mut strlong: usize = 0;
        for (k, _arg) in &self.args {
            if k.len() > strlong { strlong = k.len() }
        }
        let mut dash = String::new();
        for _ in 0..=&self.p_info.program_name.len() + &self.p_info.program_version.len() + " on version".len() {
            dash.push_str("=")
        }
        let h = &mut self.help;
        let proglen = &self.p_info.program_name.len();
        h.push_str(&format!("{}{}{}{}{} on version {}{}\n{}\nUsage{}\n", PURPLE, BOLD, &self.p_info.program_name, RESET, PURPLE, BOLD, &self.p_info.program_version, dash, RESET));
        h.push_str(&format!("    {}{}{} <{}flags{}>\n", BLUE, &self.p_info.program_name, RESET, CYAN, RESET));
        for (index, arg_pair) in self.print_arg_vecs.iter().enumerate() {
            for key in arg_pair.iter() {
                if let Some(arg) = &self.args.get(key) {
                    h.push_str(&format!("{:width$}{}{:s$}{}{:s$}\n", "", CYAN, key, RESET, arg.description, width=proglen+5, s=strlong+4))
                }
            }
            if index != self.print_arg_vecs.len()-1 {
                h.push_str("\n");
            }
        }
    }
    pub fn get<T: 'static + Clone>(&self, flag: &str) -> Result<Vec<T>, GetError> {
        match self.args.get(flag) {
            Some(arg) => {
                if let Some(r#type) = arg.expected_type {
                    if r#type != TypeId::of::<T>() { Err(GetError::InvalidType) } else {
                        match &arg.values.is_empty() {
                            true => {
                                match &arg.default_value {
                                    Some(default) => match default.downcast_ref::<T>() {
                                        Some(val) => Ok(vec![val.clone()]),
                                        None => Err(GetError::InvalidType)
                                    },
                                    None => Err(GetError::NoValue)
                                }
                            },
                            false => {
                                let mut ret = Vec::<T>::new();
                                for elem in &arg.values {
                                    match elem.downcast_ref::<T>() {
                                        Some(a) => ret.push(a.clone()),
                                        None => return Err(GetError::InvalidType) // shouldn't be possible
                                    }
                                }
                                Ok(ret)
                            },
                        }
                    }
                } else {
                    match &arg.default_value {
                        Some(d) =>  match d.downcast_ref::<T>() {
                            Some(val) => Ok(vec![val.clone()]),
                            None => Err(GetError::InvalidType)
                        },
                        None => Err(GetError::NoValue) 
                    }
                }

            },
            None => Err(GetError::InvalidFlag) // flag doesnt exist
        }
    }
}

struct Argument {
    options: u8,
    description: String,
    values: Vec<Box<dyn Any>>,
    default_value: Option<Box<dyn Any>>,
    expected_type: Option<TypeId>
}

impl Argument {
    fn new(options: u8) -> Argument {
        Argument {
            options,
            description: String::from("-"),
            values: Vec::<Box<dyn Any>>::new(),
            default_value: None,
            expected_type: None
        }
    }
}
struct PInfo<'a> {
    program_name: &'a str,
    program_version: &'a str,
}
impl <'a>PInfo<'a> {
    fn new(program_name: &'a str, program_version: &'a str) -> PInfo<'a> {
        PInfo {
            program_name,
            program_version,
        }
    }
}
pub struct EventPhrases<'a> {
    pub no_arg_err: &'a str,
    pub missing_arg_err: &'a str,
    pub invalid_arg_err: &'a str,
    pub multi_arg_err: &'a str,
}
// event phrase builder
impl <'a>EventPhrases<'a> {
    fn new() -> EventPhrases<'a> {
        EventPhrases {
            no_arg_err: "Provide at least one argument",
            missing_arg_err: "Provide an argument for this option",
            invalid_arg_err: "Invalid argument",
            multi_arg_err: "This argument can only be provided once",
        }
    }
}
pub struct Settings<'a> {
    pub allow_invalid_args: bool,
    pub generate_default_flags: bool,
    pub generate_help_menu: bool,
    pub allow_no_args: bool,
    pub event_phrases: EventPhrases<'a>,
}
// parser settings builder
impl <'a>Settings<'a> {
    fn new() -> Settings<'a> {
        Settings {
            allow_invalid_args: false,
            generate_default_flags: true,
            generate_help_menu: true,
            allow_no_args: false,
            event_phrases: EventPhrases::new(),
        }
    }
}

pub enum ParserError {
    IndexOutOfBound,
    RequirementNotMet,
    DuplicateArg,
    InvalidArg,
    NoArgs,
}
pub enum GetError {
    InvalidFlag,
    InvalidType,
    NoValue,
}

impl Display for GetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetError::NoValue => write!(f, "{}Error:{} NoValue\n{}   -->{} Value has of flag not been set", RED, RESET, YELLOW, RESET),
            GetError::InvalidFlag => write!(f, "{}Error:{} InvalidFlag\n{}   -->{} Flag does not exist", RED, RESET, YELLOW, RESET),
            GetError::InvalidType => write!(f, "{}Error:{} InvalidType\n{}   -->{} Types of flag and the get function don't match", RED, RESET, YELLOW, RESET)
        }
    }
}
impl Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::NoArgs => write!(f, "{}Error:{} NoArgs\n{}   -->{} Provide an argument", RED, RESET, YELLOW, RESET),
            ParserError::InvalidArg => write!(f, "{}Error:{} InvalidArg\n{}   -->{} Invalid argument", RED, RESET, YELLOW, RESET),
            ParserError::DuplicateArg => write!(f, "{}Error:{} DuplicateArg\n{}   -->{}Argument not allowed be called repeatedly", RED, RESET, YELLOW, RESET),
            ParserError::IndexOutOfBound => write!(f, "{}Error:{} IndexOutOfBound\n{}   -->{} Flag requires an aditional value which was not found", RED, RESET, YELLOW, RESET),
            ParserError::RequirementNotMet => write!(f, "{}Error:{} RequirementNotMet\n{}   -->{} Not all required arguments were used", RED, RESET, YELLOW, RESET)
        }
    }
}

fn find_2d<T: Ord>(n_vec: Vec<Vec<T>>, value: T) -> Option<usize> {
    n_vec.iter().enumerate()
        .find(|(_, inner_vec)| inner_vec.contains(&value))
        .map(|(index, _)| index)
}
