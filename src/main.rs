mod reader;
use reader::Savegame;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <savegame>", args[1]);
        return;
    }
    let mut savegame = Savegame::new(args[1].clone());
    println!("Read savegame: {}", args[1]);
    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        "output_savegame.sav".to_string()
    };
    savegame.save(output_path);
    println!("{}, {}, {}, {:?}", savegame.path, savegame.data.len(), savegame.version, savegame.compression);
}
