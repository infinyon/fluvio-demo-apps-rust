use crate::error::CdcError;
use crate::messages::{BinLogMessage, BnFile, FluvioMessage};
use fluvio::{FluvioError, Offset, PartitionConsumer, TopicProducer};
use tracing::instrument;

pub struct FluvioManager {
    producer: TopicProducer,
    consumer: PartitionConsumer,
    sequence: u64,
}

impl FluvioManager {
    pub async fn new(
        topic: String,
        _replicas: i16,
        sequence: Option<u64>,
    ) -> Result<Self, CdcError> {
        let sequence = sequence.unwrap_or(0);
        let producer = fluvio::producer(&topic).await?;
        let consumer = fluvio::consumer(&topic, 0).await?;

        Ok(Self {
            producer,
            consumer,
            sequence,
        })
    }

    #[instrument(skip(self))]
    pub async fn get_last_file_offset(&mut self) -> Result<Option<BnFile>, CdcError> {
        let record = get_last_record(&self.consumer).await?;
        if let Some(json_msg) = record {
            let flv_message: FluvioMessage = serde_json::from_str(&json_msg)?;
            self.sequence = flv_message.sequence + 1;

            Ok(Some(flv_message.bn_file))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self, bn_message))]
    pub async fn process_msg(&mut self, bn_message: BinLogMessage) -> Result<(), CdcError> {
        let flv_message = FluvioMessage::new(bn_message, self.sequence);
        let msg = serde_json::to_string(&flv_message).unwrap();
        self.producer.send_record(msg, 0).await?;

        // increment sequence
        self.sequence += 1;

        Ok(())
    }
}

#[instrument(skip(consumer))]
pub async fn get_last_record(consumer: &PartitionConsumer) -> Result<Option<String>, FluvioError> {
    let response = consumer.fetch(Offset::end()).await?;

    if !response.error_code.is_ok() {
        if response.error_code.to_sentence() == "OffsetOutOfRange" {
            println!("fluvio: data stream is empty");
        } else {
            println!(
                "Error (fetch_logs_once): {:?}",
                response.error_code.to_sentence()
            );
        }
    } else if let Some(batch) = response.records.batches.first() {
        if let Some(record) = batch.records().first() {
            let bytes = record.value().as_ref();
            let msg = String::from_utf8(bytes.to_vec()).unwrap();
            return Ok(Some(msg));
        }
    }

    Ok(None)
}
