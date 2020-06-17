use crossbeam_channel::Sender;
use futures::stream::StreamExt;
use std::io::{Error, ErrorKind};
use std::thread;

use flv_client::profile::ScConfig;
use flv_client::ClientError;
use flv_client::FetchLogOption;
use flv_client::FetchOffset;
use flv_client::ReplicaLeader;
use flv_client::ScClient;
use flv_client::SpuController;
use flv_client::SpuReplicaLeader;
use flv_future_aio::task::run_block_on;

//=======================
//   Public Interface
//=======================

pub fn start_consumer(topic: &String, offset: i64, sender: Sender<String>) -> Result<(), Error> {
    let mut client = connect()?;
    let replica = get_replica(&mut client, topic.clone(), 0)?;

    thread::spawn(move || {
        star_consumer_loop(replica, offset, sender.clone());
    });

    Ok(())
}

//=======================
// Wrappers for (Async)
//=======================

fn connect() -> Result<ScClient, Error> {
    let client = run_block_on(_connect())
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("{}", e)))?;
    Ok(client)
}

fn get_replica(
    client: &mut ScClient,
    topic: String,
    partition: i32,
) -> Result<SpuReplicaLeader, Error> {
    let replica = run_block_on(_get_replica(client, topic, partition))
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("{}", e)))?;
    Ok(replica)
}

fn star_consumer_loop(replica: SpuReplicaLeader, offset: i64, sender: Sender<String>) {
    run_block_on(_star_consumer_loop(replica, offset, sender));
}

//=======================
// Async APIs
//=======================

async fn _connect() -> Result<ScClient, ClientError> {
    let config = ScConfig::new(None, None)?;
    let client = config.connect().await?;

    Ok(client)
}

async fn _get_replica(
    client: &mut ScClient,
    topic: String,
    partition: i32,
) -> Result<SpuReplicaLeader, ClientError> {
    let replica = client
        .find_replica_for_topic_partition(topic.as_str(), partition)
        .await?;

    Ok(replica)
}

async fn _star_consumer_loop(mut replica: SpuReplicaLeader, offset: i64, sender: Sender<String>) {
    let mut log_stream = replica.fetch_logs(FetchOffset::Offset(offset), FetchLogOption::default());

    // read read from producer and print to terminal
    while let Some(response) = log_stream.next().await {
        let records = response.records;
        for batch in records.batches {
            for record in batch.records {
                if let Some(bytes) = record.value().inner_value() {
                    let msg = String::from_utf8(bytes).expect("error vec => string");
                    sender.send(msg).expect("error sending message");
                }
            }
        }
    }
}
