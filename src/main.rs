use vifsimlib::container::container::ParseStatus;
use pollster;
use std::fs::File;
use std::io::Read;

#[pollster::main]
async fn main() {
    use std::time::Instant;
    let now = Instant::now();

    let mut provider = File::open("provider.json").unwrap();
    let mut provider_data = String::new();
    provider.read_to_string(&mut provider_data).unwrap();

    let mut program = File::open("program.json").unwrap();
    let mut program_data = String::new();
    program.read_to_string(&mut program_data).unwrap();

    let mut server = vifsimlib::container::container::boot_container(None);
    server.load_server_params(&"{ \"stopAfter\": 5000, \"stopOn\": 0 }");
    server.load_server_params(&"{ \"stopAfter\": 5000, \"stopOn\": 1 }");
    match server.load_provider(&provider_data) {
        ParseStatus::Empty => panic!("Parse went wrong"),
        ParseStatus::Loaded => match server.load_program(&program_data) {
                ParseStatus::Empty => panic!("Parse went wrong"),
                ParseStatus::Loaded => server.start("Main").await,
        }
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}