use std::io::Error;

use cdc_messages::{BinLogMessage, BnFile, FluvioMessage};
use flv_client::{ScClient, SpuReplicaLeader};

use super::{connect, create_topic, get_last_record, get_replica, produce};

pub struct FluvioManager {
    replica: SpuReplicaLeader,
    sequence: u64,
}

impl FluvioManager {
    pub fn new(topic: String, replicas: i16, sequence: Option<u64>) -> Result<Self, Error> {
        let sequence = sequence.unwrap_or(0);
        let mut client = connect()?;
        let replica = create_topic_and_get_replica(&mut client, topic, replicas)?;

        Ok(Self { replica, sequence })
    }

    pub fn get_last_file_offset(&mut self) -> Result<Option<BnFile>, Error> {
        let record = get_last_record(&mut self.replica)?;
        if let Some(json_msg) = record {
            let flv_message: FluvioMessage = serde_json::from_str(&json_msg)?;
            self.sequence = flv_message.sequence + 1;

            Ok(Some(flv_message.bn_file))
        } else {
            Ok(None)
        }
    }

    pub fn process_msg(&mut self, json_msg: String) -> Result<(), Error> {
        // translated binlog message to fluvio message
        let bn_message: BinLogMessage = serde_json::from_str(&json_msg)?;
        let flv_message = FluvioMessage::new(bn_message, self.sequence);
        let msg = serde_json::to_string(&flv_message).unwrap();

        // produce message
        produce(&mut self.replica, msg)?;

        // increment sequence
        self.sequence += 1;

        Ok(())
    }
}

fn create_topic_and_get_replica(
    client: &mut ScClient,
    topic: String,
    replicas: i16,
) -> Result<SpuReplicaLeader, Error> {
    create_topic(client, topic.clone(), 1, replicas)?;
    Ok(get_replica(client, topic, 0)?)
}
