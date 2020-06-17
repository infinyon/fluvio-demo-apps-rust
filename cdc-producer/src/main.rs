use crossbeam_channel::{bounded, select, Receiver};
use std::io::{Error, ErrorKind};

mod binlog_manager;
mod cli;
mod fluvio_manager;
mod mysql;
mod store;

use binlog_manager::BinLogManager;
use binlog_manager::Resume;
use cli::get_cli_opt;
use cli::Config;
use fluvio_manager::FluvioManager;

fn start_loop() -> Result<(), Error> {
    // read profile
    let params = get_cli_opt();
    let config = Config::load(&params.profile)?;
    let profile = config.profile();

    // create channels
    let ctrl_c_events = ctrl_channel()?;
    let (sender, receiver) = bounded::<String>(100);

    // create fluvio manager
    let mut flv_manager = FluvioManager::new(profile.topic(), profile.replicas(), None)?;
    let bn_file = flv_manager.get_last_file_offset()?;

    // create binlog manager
    let bn_manager = BinLogManager::new(&profile, sender)?;

    // create resume
    let resume = Resume::new(bn_file);
    if let Some(resume) = &resume {
        println!("{}", resume);
    }

    let ts_frequency = None;
    bn_manager.run(resume, ts_frequency);

    loop {
        select! {
            recv(receiver) -> msg => {
                match msg {
                    Ok(msg) => {
                        if let Err(err) = flv_manager.process_msg(msg) {
                            println!("{}", err.to_string());
                        }
                    },
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
