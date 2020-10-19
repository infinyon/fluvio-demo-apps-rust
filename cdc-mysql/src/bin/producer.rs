use crossbeam_channel::{bounded, select, Receiver};
use std::io::{Error, ErrorKind};
use tracing::error;
use tracing_subscriber::prelude::*;

use fluvio_cdc::error::CdcError;
use fluvio_cdc::messages::BinLogMessage;
use fluvio_cdc::producer::{get_cli_opt, Config};
use fluvio_cdc::producer::{BinLogManager, FluvioManager, Resume};

async fn run() -> Result<(), CdcError> {
    // read profile
    let params = get_cli_opt();
    let config =
        Config::load(&params.profile).map_err(|source| CdcError::ConfigError { source })?;
    let profile = config.profile();
    let skip_fluvio = params.skip_fluvio;

    // create channels
    let ctrl_c_events = ctrl_channel()?;
    let (sender, receiver) = bounded::<String>(100);

    // create fluvio manager
    let mut flv_manager = FluvioManager::new(profile.topic(), profile.replicas(), None).await?;

    // create binlog manager
    let bn_manager = BinLogManager::new(&profile, sender)
        .map_err(|source| CdcError::BinlogFileError { source })?;

    // create resume offset or none
    let mut resume = Resume::load(profile.resume_offset_file())
        .await
        .map_err(|source| CdcError::ResumeError { source })?;
    if let Some(binfile) = resume.binfile.as_ref() {
        println!("Resuming from {:?}", binfile);
    } else {
        println!("Resuming from start");
    }
    println!("{:?}", resume);

    let ts_frequency = None;
    bn_manager.run(resume.clone(), ts_frequency);

    loop {
        select! {
            recv(receiver) -> msg => {
                match msg {
                    Ok(msg) => {
                        let bn_message: BinLogMessage = serde_json::from_str(&msg)?;
                        let bn_file = bn_message.bn_file.clone();
                        if !skip_fluvio {
                            if let Err(err) = flv_manager.process_msg(bn_message).await {
                                println!("{}", err.to_string());
                                error!("{}", err.to_string());
                                std::process::exit(1);
                            }
                        }
                        resume.update_binfile(bn_file).await?;
                    },
                    Err(err) => {
                        println!("{}", err.to_string());
                        error!("{}", err.to_string());
                        std::process::exit(1);
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

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    async_std::task::block_on(run())?;
    Ok(())
}
