// use data::configdata::*;
use data::config::*;
// extern crate clap;
// use clap::{Arg, App, SubCommand};

fn main() {
    // let matches = App::new("Maratona Config Checker")
    //                       .version("1.0")
    //                       .author("Emilio Wuerges. <wuerges@gmail.com>")
    //                       .about("Checks the maratona configs.")
    //                       .arg(Arg::with_name("config")
    //                            .short("c")
    //                            .long("config")
    //                            .value_name("FILE")
    //                            .help("Sets a custom config file")
    //                            .takes_value(true))
    //                       .get_matches();

    // println!("matches: {:?}", matches);

    let contest = contest();

    println!("titulo = \"Maratona de Programação da SBC - 1ª fase\"");
    for s in contest.sedes {
        if s.codes.len() == 1 {
            println!(
                "\n\n[[sede]]\nnome=\"{}\"\ncodigo=\"{}\"\npremiacao={}\nvagas={}\n",
                s.name, s.codes[0], s.premiacao, s.vagas
            );
        }
        else {
            println!(
                "\n\n[[supersede]]\nnome=\"{}\"\npremiacao={}\nvagas={}",
                s.name, s.premiacao, s.vagas
            );
            println!("codigo=[");
            for c in s.codes {
                println!("  \"{}\",", c);
            }
            println!("]");
        }
    }
}
