use crossbeam_channel::{bounded, select, Receiver, Sender};
use futures::StreamExt;
use std::io::{Error, ErrorKind};
use tracing::error;

use fluvio_cdc::consumer::MysqlManager;
use fluvio_cdc::consumer::OffsetStore;
use fluvio_cdc::consumer::{get_cli_opt, Config};

use fluvio::{FluvioError, Offset, PartitionConsumer};

async fn run() -> Result<(), FluvioError> {
    // read profile
    let params = get_cli_opt();
    let config = Config::load(&params.profile)?;
    let profile = config.profile();

    // init store
    let mut offset_store = OffsetStore::init(profile.last_offset_file()).await?;

    // connect to db
    println!("Connecting to mysql database... ");
    let mut mysql = MysqlManager::connect(profile)?;

    // create channels
    let ctrl_c_events = ctrl_channel()?;
    let (sender, receiver) = bounded::<String>(100);

    // start Fluvio consumer thread
    let consumer = fluvio::consumer(&profile.topic(), 0).await?;
    let offset = Offset::absolute(offset_store.offset()).unwrap();
    async_std::task::spawn(consume(consumer, offset, sender));

    loop {
        select! {
            recv(receiver) -> msg => {
                match msg {
                    Ok(msg) => {
                        mysql.update_database(msg)?;
                        offset_store.increment_offset().await?;
                    }
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

async fn consume(
    consumer: PartitionConsumer,
    offset: Offset,
    sender: Sender<String>,
) -> Result<(), FluvioError> {
    let mut stream = consumer.stream(offset).await?;

    // read read from producer and print to terminal
    while let Some(Ok(record)) = stream.next().await {
        let bytes = record.as_ref();
        let msg = String::from_utf8(bytes.to_vec()).expect("error vec => string");
        sender.send(msg).expect("error sending message");
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

fn main() {
    if let Err(err) = async_std::task::block_on(run()) {
        println!("Error: {}", err.to_string());
    }
}
