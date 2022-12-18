# ⚡argparse.rs
Argparse is a fast cli argument parsing library with a minimal codebase and 0 dependencies
Similar to the [Argparse](https://github.com/NewDawn0/argparser) from the other languages as it's is just a port to rust, which means virtually no learning curve if you used another version of the parse before. Even if not the the library is quite simple to use.

## Why should you use argparse
- **🚀 Fast compile times:** Library uses 0 external dependencies
- **🚀 Fast execution times:** Average execution times of 72ms tested on a 2016 MacBook-pro
- **⚡ Customizability:** Easily interfere and override many of the built in events
- **🧠 Easy to use:** The learning curve is minimal
- **🤏 Small codebase:** Only 169 lines of code


## Installation
Adding this library is quite simple. In your Cargo.toml simply add the following lines
```toml
[dependencies]
argparse = { git = "https://github.com/NewDawn0/argparser.rs.git" }

# ... your other dependencies
```
or run `$ cargo add --git https://github.com/NewDawn0/argparser.rs.git`. With that the library is installed and ready to be used.

## Usage
```rust
use argparse::ArgParser;

fn main() {
    // initialize a new parser
    let mut parser = ArgParser::new();

    // Configure the parser using the settings fields of the parser
    parser.settings.allow_invalid_args = true;
    parser.settings.event_phrases.no_args_err = "Provide an argument".to_string();
    parser.settings.event_functions.invalid_args_err = Some(a_function);

    // add arguments
    parser.add_args("-j", true, false);
    parser.add_args("-h", true, true);
    parser.add_args("--help", true, true);

    // parse
    match parser.parse() {
        Ok(()) => {}, // Do something...
        Err(e) => eprinln!("{}", e);
    }
}

fn a_function() {
    println!("Other function is called");
}
```

## Parser Configuration
The parser configuration is optional but if you want to override the default values and behaviours it is necessary. This type of configuration is also what make this library so flexible.
The configuration is a substruct inside the ArgParser struct called settings. The settings contain some more nested structs for different options. Global options like allowing the case that no arguments are provided or allowing invalid arguments to be collected in the other_argsVector are able to be ajusted right in the settings substruct. Event phrases are error messages which are formatted and returned in the event that an error occurs. The event phrases are found in as a substruct in the settings struct and each of the fields in event phrases struct is a different error
```rust
Settings struct event structure fields seperated by dot notation
ArgParser.settings
          ├── allow_no_args: bool
          ├── allow_invalid_args: bool
          ├── event_phrases
          │     ├── no_arg_err: String
          │     ├── missing_arg_er: String
          │     ├── invalid_arg_err: String
          │     └── multi_arg_err: String
          └── event_functions
                ├── no_arg_err: Option<fn()>
                ├── missing_arg_err: Option<fn()>
                ├── invalid_arg_err: Option<fn()>
                ├── multi_arg_err: Option<fn()>
                └── no_args_allowed: Option<fn()>
```
**General Options**:
Modify some more general options
- **allow_no_args**: allow the program to be run without arguments
- **allow_invalid_args**: Allow the program to be run with invalid arguments which are stored in the other_args string vector

**Event Phrases**
Event phrases change the error message which is returned in the result upon calling the parse function
- **no_arg_err**: if there are no arguments provided and the programme isn't allowed to be run without arguments
- **missing_arg_err**: if the flag is missing arguments
- **invalid_arg_err**: if the argument provided is invalid (will not be run if invalid argument are enabled)
- **multi_arg_err**: if the provided argument was provided multiple times which is disallowed as default per argument

**Event Functions**
Event functions are functions that are able to override the default behaviour in error events
- **no_arg_err**: override the function that would get executed if no arguments are provided if it is dissalowed
- **missing_arg_err**: override the function that would get executed if the argumtns are provided
- **invalid_arg_err**: override the function that would return the invalid argument error
- **multi_arg_err**:  override the function returning the error message if the same arg is provided multiple times and the argument can't be used numerous times
- **no_args_allowed**: change the function if it's allowed to have no arguments

## Add Arguments
Adding arguments is quite easy the parser has a function called add_args which takes 3 arguments. First the function takes a string as the argument name. Argument 2 is a boolean which determines if the flag is allowed to be used multiple times. The last variable determines wether the flag does not require an argument.
```rust
use argparse::ArgParser;

fn main () {
    // Some argument examples

    // create new parser
    let parser = ArgParser::new();

    // Here the --help is the name of the flag and as it is an help flag we want it to be allowed to be used multiple times, and because it is a help flag it doesn't require an argument
    parser.add_args("--help", true, true);

    // Because the argument is an output file, we want the flag to only be used once as we only want to create one outfile. This file usualla needs a name which is why we require the next argument
    parser.add_args("--outfile", false, false);
}
```

## Parsing
Using the parse method the provided arguments get parsed
The Arguments are stored as a HashMap of a string which is the argumtent's name as the key and an option of a vector of strings of infos. If invalid arguments are allowed, they are stored in a vector of strings called other_args. If the flag does not require an argument the hash map value is None 
```rust
use argparse::ArgParser;

fn main() {
    // create parser and configure it
    let mut parser = ArgParser::new();
    parser.settings.allow_invalid_args = true;

    // add flags and parse
    parser.add_args("-i", true, false);
    parser.parse();

    // print all the flags
    println!("Other arguments {:?}", parser.other_args);
    println!("input files {:?}", parser.args["-i"]);
}
```
