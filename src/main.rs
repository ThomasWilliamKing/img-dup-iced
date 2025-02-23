// #![feature(unsafe_destructor, phase)]

use crate::config::parse_args;
use crate::config::ProgramSettings;
use output::test_outfile;
use std::env::args;

macro_rules! json_insert(
    ($map:expr, $key:expr, $val:expr) => (
        $map.insert(::std::borrow::ToOwned::to_owned($key), $val.to_json())
    );
);

mod config;
mod img;
mod output;
mod processing;
mod par_queue;

#[cfg(feature = "gui")]
mod ui;

fn main() {
    run();

    exit();
}

// Exit immediately, don't leave any threads alive
pub fn exit() {    
    unsafe { libc::exit(0); }   
}

#[cfg(feature = "gui")]
fn show_gui(settings: ProgramSettings) {   
	ui::show_gui(settings);
}

#[cfg(not(feature = "gui"))]
fn show_gui(_: ProgramSettings) {
    println!("img_dup was not compiled with GUI support!");    
}

fn run() {
    let args = args();

    let settings = parse_args(args.as_slice());

	if settings.gui {
        show_gui(settings);
		return;
	}

    // Silence standard messages if we're outputting JSON
    let mut out = get_output(&settings);    

    match settings.outfile {
        Some(ref outfile) => {
            (writeln!(out, "Testing output file ({})...",
                outfile.display())).unwrap();
            test_outfile(outfile).unwrap();
        },
        None => (),        
    };
    
    out.write_line("Searching for images...").unwrap();

    let mut image_paths = processing::find_images(&settings);

    let image_count = image_paths.len();

    (writeln!(out, "Images found: {}", image_count)).unwrap();

    if settings.limit > 0 {
        (writeln!(out, "Limiting to: {}", settings.limit)).unwrap();
        image_paths.truncate(settings.limit);
    }

    (writeln!(out, "Processing images in {} threads. Please wait...\n", 
             settings.threads)).unwrap();

    let results = processing::process(&settings, image_paths);

    out.write_line("").unwrap();

    output::output_results(&settings, &results).unwrap()   
}

fn get_output<Writer>(settings: &ProgramSettings) -> Box<Writer> {
    // if settings.silent_stdout() {
    //     Box::new(NullWriter) as Box<Writer> 
    // } else {
        Box::new(std::io::stdout())  as Box<Writer>
    // }    
}

