mod actionscript;

use crate::{
    actionscript::Actionscript,
};

const VERSION: u32 = 1; // Big change / New feature
const REV: u32 = 0; // Little non breaking change in the code
const HOTFIX: u32 = 0; // Hotfix

// TODO: create rules (in a easy script language ?) so translate json -> new AS files directly without the need of a older src version already cleaned

fn main() {
    println!("Dofuscator v{}.{}.{}", VERSION, REV, HOTFIX);

    let start_time = std::time::Instant::now();

    /*let actionscript = Actionscript::from_file("Mount.as").unwrap();

    actionscript.to_json("Mount.json").unwrap();

    let actionscript = Actionscript::from_file("AudioManager_obfu.as").unwrap();

    actionscript.to_json("AudioManager_obfu.json").unwrap();*/

    let actionscript = Actionscript::from_file("AudioManager.as").unwrap();

    actionscript.to_json("AudioManager.json").unwrap();

    println!("Done in {}ms", start_time.elapsed().as_millis());
    println!("Press any key to close...");
    
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
}