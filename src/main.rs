use clap::{crate_version, App, Arg};
use rand::Rng;
use terminal_size::{terminal_size, Height, Width};

use eca1d::{Ca, TermColor, TermImage};

fn is_binary_or_u8(val: String) -> Result<(), String> {
    let err = String::from("has to be binary string (ex 0b01010101) or number between 0-255");
    if val.starts_with("0b") {
        match u8::from_str_radix(&val[2..], 2) {
            Ok(_) => Ok(()),
            Err(_) => Err(err),
        }
    } else {
        match val.parse::<u8>() {
            Ok(_) => Ok(()),
            Err(_) => Err(err),
        }
    }
}

fn is_usize(val: String) -> Result<(), String> {
    match val.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("has to be a number")),
    }
}

fn is_float_between_0_1(val: String) -> Result<(), String> {
    let num = match val.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return Err(String::from("has to be number")),
    };

    if num > 1. || num < 0. {
        return Err(String::from("has to be between 0 and 1"));
    }
    Ok(())
}

fn main() {
    let matches = App::new("Elementary 1D Cellular Automata Explorer")
                            .version(crate_version!())
                            .about("Quickly explore different rules for elementary 1D cellular automata.")
                            .arg(Arg::with_name("rule")
                                .takes_value(true)
                                .required(true)
                                .validator(is_binary_or_u8)
                                .index(1)
                                .help("The rule to use (0-255)."))
                            .arg(Arg::with_name("width")
                                .short("w")
                                .long("width")
                                .takes_value(true)
                                .validator(is_usize)
                                .help("The width of the automata (defaults to width of the terminal window)."))
                            .arg(Arg::with_name("iterations")
                                .short("i")
                                .long("iter")
                                .takes_value(true)
                                .validator(is_usize)
                                .help("The number of simulation steps (defaults to the height of the terminal window - 1)."))
                            .arg(Arg::with_name("random")
                                .short("r")
                                .long("random")
                                .takes_value(true)
                                .validator(is_float_between_0_1)
                                .help("Randomly generated seed with density <random>."))
                            .arg(Arg::with_name("braille")
                                .short("b")
                                .long("braille")
                                .help("Draw the image using unicode braille symbols.")
                                .conflicts_with("unicode"))
                            .arg(Arg::with_name("unicode")
                                .short("u")
                                .long("unicode")
                                .help("Draw the image using unicode HALF BLOCK symbols"))
                            .arg(Arg::with_name("print_rules")
                                .short("p")
                                .long("print-rules")
                                .help("Print the rules"))
                            .get_matches();

    // Safe to unwrap since arg is required and validated.
    let rule = matches.value_of("rule").unwrap();
    let rule = if rule.starts_with("0b") {
        u8::from_str_radix(&rule[2..], 2).unwrap()
    } else {
        rule.parse().unwrap()
    };

    let (term_width, term_height) = if let Some((Width(w), Height(h))) = terminal_size() {
        (w, h)
    } else {
        // If we can't find the terminal size let's just use something small.
        (80, 40)
    };

    // To fill the terminal when no width or height is specified we need to
    // compensate for the extra data in braille symbols (4x2) and HALF BLOCKS
    // (2x1).
    let width: usize = if let Some(w) = matches.value_of("width") {
        // Value is validated by clap as usize.
        w.parse().unwrap()
    } else {
        if matches.is_present("braille") {
            (term_width * 2) as usize
        } else {
            term_width as usize
        }
    };
    let height: usize = if let Some(h) = matches.value_of("iterations") {
        // Value is validated by clap as usize
        h.parse().unwrap()
    } else {
        // Printing the rules require two columns at the top.
        let offset = if matches.is_present("print_rules") {
            3
        } else {
            1
        };

        let mult = if matches.is_present("braille") {
            4
        } else if matches.is_present("unicode") {
            2
        } else {
            1
        };
        ((term_height - offset) * mult) as usize
    };

    if matches.is_present("print_rules") {
        print_rules(rule);
    }

    let seed = if let Some(r) = matches.value_of("random") {
        let mut rng = rand::thread_rng();
        let density: f64 = r.parse().unwrap();

        let mut res = Vec::with_capacity(width);
        for _ in 0..width {
            res.push(if rng.gen::<f64>() < density { 1 } else { 0 });
        }
        res
    } else {
        let mut res = vec![0; width];
        let len = res.len();
        res[len / 2] = 1;
        res
    };

    let mut ca = Ca::new(seed, rule);

    let image = TermImage::new(ca.run(height));
    if matches.is_present("braille") {
        print!("{}", image.draw_braille(TermColor::White, TermColor::Black));
    } else if matches.is_present("unicode") {
        print!("{}", image.draw_unicode(TermColor::White, TermColor::Black));
    } else {
        print!("{}", image.draw_ascii());
    }
}

fn print_rules(rule: u8) {
    let top: String = (0..8).map(|i| format!(" {:03b} |", i)).collect();
    let bottom: String = format!("{:08b}", rule)
        .bytes()
        .map(|i| format!("  {}  |", i as char))
        .collect();
    println!("|{}", top);
    println!("|{}", bottom);
}
