use std::{
    collections::HashMap,
    env::args
};
// static const
const RED: &'static str = "\x1b[1;31m";
const RESET: &'static str = "\x1b[0m";


// ArgParser
// parser struct
pub struct ArgParser {
    pub settings: Settings,
    pub other_args: Vec<String>,
    pub args: HashMap<String, Option<Vec<String>>>,
    _private: Private
}

// parser impl
impl ArgParser {
    // public
    pub fn new() -> ArgParser {
        ArgParser {
            settings: Settings::new(),
            other_args: Vec::new(),
            args: HashMap::new(),
            _private: Private::new()
        }
    }
    pub fn add_arg(&mut self, arg: &str, multiple_allowed: bool, require_next_arg: bool) {
        if multiple_allowed {self._private.multi.push(arg.clone().to_string())}
        if !require_next_arg {self._private.no_next.push(arg.clone().to_string())}
        self._private.arg_keys.push(arg.to_string())
    }
    pub fn parse(&mut self) -> Result<(), String> {
        let argv: Vec<String> = args().skip(1).collect();
        let argc = argv.len();
        // if no args
        // if no args are provided but allowed
        if argc == 0 {
            if !self.settings.allow_no_args {
                return match self.settings.event_functions.no_args_allowed {
                    Some(func) => Ok(func()),
                    None => Err(format!("{}ArgParse Errror {}Provide at least one argument", RED, RESET).to_string())
                }
            // no args provided but allowed
            } else {
                return match self.settings.event_functions.no_arg_err {
                    Some(func) => Ok(func()),
                    None => Err(format!("{}ArgParse Errror {}{}", RED, RESET, self.settings.event_phrases.no_arg_err))
                }
            }
        } else {
            let mut arg_index: usize = 0;
            while arg_index < argc {
                // check if arg is valid
                if ArgParser::contains(&self._private.arg_keys, &argv[arg_index]) != -1 {
                    // if not yet done or multiple allowed
                    if ArgParser::contains(&self._private.multi, &argv[arg_index]) != -1 || ArgParser::contains(&self._private.multi_cmp, &argv[arg_index]) == -1 {
                        self._private.multi_cmp.push(argv[arg_index].clone());
                        // if no next arg is required
                        if ArgParser::contains(&self._private.no_next, &argv[arg_index]) != -1 {
                            self.args.insert(argv[arg_index].clone(), None);
                        // if next arg is required
                        } else {
                            // if there is a next elem
                            if arg_index+1 <= argc-1 {
                                if self.args.contains_key(&argv[arg_index]) {
                                    match self.args[&argv[arg_index]].clone() {
                                        Some(mut vec) => {
                                            vec.push(argv[arg_index+1].clone());
                                            // replace vec
                                            let _ = &&self.args.remove(&argv[arg_index]);
                                            self.args.insert(argv[arg_index].clone(), Some(vec));
                                        },
                                        None => {}
                                    }
                                } else {
                                    let vec: Vec<String> = vec![argv[arg_index+1].clone()];
                                    self.args.insert(argv[arg_index].clone(), Some(vec));
                                }
                                // skip
                                arg_index += 1;
                            // no next arg err
                            } else {
                                match self.settings.event_functions.missing_arg_err {
                                    Some(func) => func(),
                                    None => return Err::<(), String>(format!("{}ArgParse Errror \'{}{}{}\'{} {}", RED, RESET, &argv[arg_index], RED, RESET, self.settings.event_phrases.missing_arg_err))
                                }
                            }
                        }
                    } else {
                        match self.settings.event_functions.multi_arg_err {
                            Some(func) => func(),
                            None => return Err::<(), String>(format!("{}ArgParse Errror \'{}{}{}\'{} {}", RED, RESET, argv[arg_index], RED, RESET, self.settings.event_phrases.multi_arg_err))
                        }
                    }
                // invalid arg
                } else {
                    if self.settings.allow_invalid_args {
                        self.other_args.push(argv[arg_index].clone());
                    } else {
                        match self.settings.event_functions.invalid_arg_err {
                            Some(func) => func(),
                            None => return Err::<(), String>(format!("{}ArgParse Errror \'{}{}{}\'{} {}", RED, RESET, argv[arg_index], RED, RESET, self.settings.event_phrases.invalid_arg_err))
                        }
                    }
                }
                // inc (DO NOT TOUCH!)
                arg_index += 1;
            };
        }
        Ok(())
    }
    // private
    fn contains(target_vec: &Vec<String>, item: &String) -> i32 {
        for (pos, elem) in target_vec.iter().enumerate() {
            if item.eq(elem) {
                let ret: i32 = pos as i32;
                return ret;
            }
        }
        return -1
    }
}

////// Parser Settings
//// event phrases
// event phrase struct
pub struct EventPhrases {
    pub no_arg_err: String,
    pub missing_arg_err: String,
    pub invalid_arg_err: String,
    pub multi_arg_err: String,
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
pub struct EventFunctions {
    pub no_arg_err: Option<fn()>,
    pub missing_arg_err: Option<fn()>,
    pub invalid_arg_err: Option<fn()>,
    pub multi_arg_err: Option<fn()>,
    pub no_args_allowed: Option<fn()>,
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
pub struct Settings {
    pub allow_invalid_args: bool,
    pub allow_no_args: bool,
    pub event_phrases: EventPhrases,
    pub event_functions: EventFunctions
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
// _private vars
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
