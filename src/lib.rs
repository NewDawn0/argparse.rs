use std::{
    collections::HashMap,
    process::exit,
    env::args,
};
// static const
const RED: &'static str = "\x1b[1;31m";
const RESET: &'static str = "\x1b[0m";


// ArgParser
// parser struct
pub struct ArgParser {
    settings: Settings,
    other_args: Vec<String>,
    args: HashMap<String, Option<Vec<String>>>,
    private: Private
}

// parser impl
impl ArgParser {
    // public
    pub fn new() -> ArgParser {
        ArgParser {
            settings: Settings::new(),
            other_args: Vec::new(),
            args: HashMap::new(),
            private: Private::new()
        }
    }
    pub fn add_arg(&mut self, arg: &str, multiple_allowed: bool, require_next_arg: bool) {
        if multiple_allowed {self.private.multi.push(arg.clone().to_string())}
        if !require_next_arg {self.private.no_next.push(arg.clone().to_string())}
        self.private.arg_keys.push(arg.to_string())
    }
    pub fn parse(&mut self) {
        let argv: Vec<String> = args().skip(1).collect();
        let argc = argv.len();
        // if no args
        if argc == 0 {
            // if no args are provided but allowed
            if self.settings.allow_no_args {
                match self.settings.event_functions.no_args_allowed {
                    Some(func) => func(),
                    None => {}
                }
            // no args provided but allowed
            } else {
                match self.settings.event_functions.no_arg_err {
                    Some(func) => func(),
                    None => {
                        eprintln!("{}ArgParse Errror {}{}", RED, RESET, self.settings.event_phrases.no_arg_err);
                        exit(1);
                    }
                }
            }
        // if some args
        } else {
            let mut arg_index: usize = 0;
            while arg_index < argc {
                // check if arg is valid
                if ArgParser::contains(self.private.arg_keys.clone(), argv[arg_index].clone()) != -1 {
                    // if not yet done or multiple allowed
                    if ArgParser::contains(self.private.multi.clone(), argv[arg_index].clone()) != -1 || ArgParser::contains(self.private.multi_cmp.clone(), argv[arg_index].clone()) == -1 {
                        self.private.multi_cmp.push(argv[arg_index].clone());
                        // if no next arg is required
                        if ArgParser::contains(self.private.no_next.clone(), argv[arg_index].clone()) != -1 {
                            self.args.insert(argv[arg_index].clone(), None);
                        // if next arg is required
                        } else {
                            // if there is a next elem
                            if arg_index+1 <= argc-1 {
                                println!("Valid next => ( argc:{}, arg_index:{} )", argc, arg_index);
                            // no next arg err
                            } else {
                                match self.settings.event_functions.missing_arg_err {
                                    Some(func) => func(),
                                    None => {
                                        eprintln!("{}ArgParse Errror \'{}{}{}\'{} {}", RED, RESET, argv[arg_index], RED, RESET, self.settings.event_phrases.missing_arg_err);
                                        exit(1);
                                    }
                                }
                            }
                        }
                    }
                // invalid argument
                } else {
                    if self.settings.allow_invalid_args {
                        self.other_args.push(argv[arg_index].clone());
                    } else {
                        match self.settings.event_functions.invalid_arg_err {
                            Some(func) => func(),
                            None => {
                                eprintln!("{}ArgParse Errror \'{}{}{}\'{} {}", RED, RESET, argv[arg_index], RED, RESET, self.settings.event_phrases.invalid_arg_err);
                                exit(1);
                            }
                        }
                    }
                }
                // End: inc (Do not touch!)
                arg_index += 1;
            }
        }
    }
    // @test
    fn testing(self) {
        println!("\n\x1b[1;31m<---------@testing---------->\x1b[0m");
        println!("\x1b[1;33m///// pub /////\x1b[0m");
        // *@other_args
        print!("other_args => [ ");
        for item in self.other_args.iter() {
            print!("{} ", item);
        }
        println!("];");
        // *@args
        println!("args => [ ");
        for (arg, key) in &self.args {
            print!("  {} => [ ", arg);
            match key {
                Some(item) => {
                    for elem in item.iter() {
                        print!("{} ", elem);
                    }
                    println!("];");
                },
                None => println!("None ];")
            }
        }
        println!("];");
        println!("\x1b[1;33m///// privd /////\x1b[0m");
        // *@arg_keys
        print!("arg_keys => [ ");
        for item in self.private.arg_keys.iter() {
            print!("{} ", item);
        }
        println!("];");
        // *@multi
        print!("multi => [ ");
        for item in self.private.multi.iter() {
            print!("{} ", item);
        }
        println!("];");
        // *@multi_cmp
        print!("multi_cmp => [ ");
        for item in self.private.multi_cmp.iter() {
            print!("{} ", item);
        }
        println!("];");
        println!("\x1b[1;31m<-------end @testing-------->\x1b[0m");
    }
    // private
    fn contains(target_vec: Vec<String>, item: String) -> i32 {
        for (pos, elem) in target_vec.iter().enumerate() {
            if item.eq(elem) {
                let ret: i32 = pos as i32;
                return ret;
            }
        }
        return -1
    }
}

fn test() {
    println!("Function ptr is called!!");
}
//////////////// Main /////////////////
fn main() {
    let mut parser = ArgParser::new();
    parser.settings.allow_no_args = true;
    parser.settings.allow_invalid_args = false;
    parser.settings.event_functions.no_args_allowed = Some(test);
    parser.add_arg("-h", true, true);
    parser.parse();
    parser.testing();
}

//////////////// Parser Settings /////////////////
//// event phrases
// event phrase struct
struct EventPhrases {
    no_arg_err: String,
    missing_arg_err: String,
    invalid_arg_err: String,
    multi_arg_err: String,
}
// event phrase builder
impl EventPhrases {
    fn new() -> EventPhrases {
        EventPhrases {
            no_arg_err: "Provide at least one argument".to_string(),
            missing_arg_err: "Provide an argument for this option".to_string(),
            invalid_arg_err: "Invalid argument".to_string(),
            multi_arg_err: "This argument can only be provided once".to_string(),
        }
    }
}
//// event functions
// event function struct
struct EventFunctions {
    no_arg_err: Option<fn()>,
    missing_arg_err: Option<fn()>,
    invalid_arg_err: Option<fn()>,
    multi_arg_err: Option<fn()>,
    no_args_allowed: Option<fn()>,
}
// event function builder
impl EventFunctions {
    fn new() -> EventFunctions {
        EventFunctions {
            no_arg_err: None,
            missing_arg_err: None,
            invalid_arg_err: None,
            multi_arg_err: None,
            no_args_allowed: None,
        }
    }
}
//// parser settings
// parser settings struct
struct Settings {
    allow_invalid_args: bool,
    allow_no_args: bool,
    event_phrases: EventPhrases,
    event_functions: EventFunctions
}
// parser settings builder
impl Settings {
    fn new() -> Settings {
        Settings {
            allow_invalid_args: false,
            allow_no_args: false,
            event_phrases: EventPhrases::new(),
            event_functions: EventFunctions::new()
        }
    }
}
// private vars
struct Private{
    arg_keys: Vec<String>,
    multi: Vec<String>,
    multi_cmp: Vec<String>,
    no_next: Vec<String>
}
impl Private {
    fn new() -> Private {
        Private {
            arg_keys: Vec::new(),
            multi: Vec::new(),
            multi_cmp: Vec::new(),
            no_next: Vec::new()
        }
    }
}
