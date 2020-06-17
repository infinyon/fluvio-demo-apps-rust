use async_std::task;
use std::io::{Error, ErrorKind};
use std::time::Duration;

use flv_client::profile::ScConfig;
use flv_client::query_params::ReplicaConfig;
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

pub fn connect() -> Result<ScClient, Error> {
    let client = run_block_on(_connect())
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Error (connect): {}", e)))?;
    Ok(client)
}

pub fn create_topic(
    client: &mut ScClient,
    topic: String,
    partitions: i32,
    replicas: i16,
) -> Result<(), Error> {
    run_block_on(_create_topic(client, topic, partitions, replicas))
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("(create_topic): {}", e)))?;
    Ok(())
}

pub fn get_replica(
    client: &mut ScClient,
    topic: String,
    partition: i32,
) -> Result<SpuReplicaLeader, Error> {
    let replica = run_block_on(_get_replica(client, topic, partition))
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("(get_replica): {}", e)))?;
    Ok(replica)
}

pub fn produce(replica: &mut SpuReplicaLeader, msg: String) -> Result<(), Error> {
    let replica = run_block_on(_produce(replica, msg))
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("(produce): {}", e)))?;
    Ok(replica)
}

pub fn get_last_record(replica: &mut SpuReplicaLeader) -> Result<Option<String>, Error> {
    let record = run_block_on(_get_last_record(replica))
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("(get_last_record): {}", e)))?;
    Ok(record)
}

//=======================
//   Implementation
//=======================

async fn _connect() -> Result<ScClient, ClientError> {
    let config = ScConfig::new(None, None)?;
    let client = config.connect().await?;

    Ok(client)
}

async fn _create_topic(
    client: &mut ScClient,
    topic: String,
    partitions: i32,
    replicas: i16,
) -> Result<(), ClientError> {
    // crete topic (if doesn't exist)
    let res = client.topic_metadata(Some(vec![topic.clone()])).await?;
    if res.len() == 0 || res[0].error.is_some() {
        client
            .create_topic(
                topic,
                ReplicaConfig::Computed(partitions, replicas, true),
                false,
            )
            .await?;

        // allow time to propagate
        task::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
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

async fn _produce(replica: &mut SpuReplicaLeader, msg: String) -> Result<(), ClientError> {
    let record = msg.as_bytes().to_vec();
    println!("sending {} bytes", record.len());
    println!("{:?}", &msg);

    replica.send_record(record).await?;

    Ok(())
}

async fn _get_last_record(replica: &mut SpuReplicaLeader) -> Result<Option<String>, ClientError> {
    let last_offset = FetchOffset::Latest(Some(1));
    let response = replica
        .fetch_logs_once(last_offset, FetchLogOption::default())
        .await?;

    if !response.error_code.is_ok() {
        if response.error_code.to_string() == "OffsetOutOfRange".to_owned() {
            println!("fluvio: data stream is empty");
        } else {
            println!(
                "Error (fetch_logs_once): {:?}",
                response.error_code.to_sentence()
            );
        }
    } else {
        for batch in response.records.batches {
            for record in batch.records {
                if let Some(bytes) = record.value().inner_value() {
                    let msg = String::from_utf8(bytes).unwrap();
                    return Ok(Some(msg));
                }
            }
        }
    }

    Ok(None)
}
