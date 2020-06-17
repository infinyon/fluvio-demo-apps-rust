mod fluvio_manager;
mod fluvio_ops;

pub use fluvio_manager::FluvioManager;
pub use fluvio_ops::connect;
pub use fluvio_ops::create_topic;
pub use fluvio_ops::get_last_record;
pub use fluvio_ops::get_replica;
pub use fluvio_ops::produce;
