use crossbeam_channel::{bounded, select, Receiver};
use std::io::{Error, ErrorKind};

mod cli;
mod fluvio;
mod mysql_manager;
mod store;

use cli::get_cli_opt;
use cli::Config;
use fluvio::start_consumer;
use mysql_manager::MysqlManager;
use store::OffsetStore;

fn start_loop() -> Result<(), Error> {
    // read profile
    let params = get_cli_opt();
    let config = Config::load(&params.profile)?;
    let profile = config.profile();

    // init store
    let mut offset_store = OffsetStore::init(profile.last_offset_file())?;

    // connect to db
    println!("Connecting to mysql database... ");
    let mut mysql = MysqlManager::connect(profile)?;

    // create channels
    let ctrl_c_events = ctrl_channel()?;
    let (sender, receiver) = bounded::<String>(100);

    // start Fluvio consumer thread
    start_consumer(&profile.topic(), offset_store.offset(), sender)?;

    loop {
        select! {
            recv(receiver) -> msg => {
                match msg {
                    Ok(msg) => {
                        mysql.update_database(msg)?;
                        offset_store.increment_offset()?;
                    }
                    Err(err) => {
                        println!("{}", err.to_string());
                        std::process::exit(0);
                    }
                }
            }
            recv(ctrl_c_events) -> _ => {
                println!();
                println!("Exited by user");
                break;
            }
        }
    }

    Ok(())
}

fn ctrl_channel() -> Result<Receiver<()>, Error> {
    let (sender, receiver) = bounded(100);
    if let Err(err) = ctrlc::set_handler(move || {
        let _ = sender.send(());
    }) {
        return Err(Error::new(ErrorKind::InvalidInput, err));
    }

    Ok(receiver)
}

fn main() -> Result<(), ()> {
    if let Err(err) = start_loop() {
        println!("Error: {}", err.to_string());
        std::process::exit(0);
    }

    Ok(())
}
