use crate::interpreter::instructions::memory::get_memory_region;
use fil_actors_evm_shared::uints::U256;
use fil_actors_runtime_v11::ActorError;
use fvm_ipld_encoding::IPLD_RAW;
use fvm_shared::event::{Entry, Flags};
use {
    crate::interpreter::{ExecutionState, System},
    fil_actors_runtime_v11::runtime::Runtime,
};

/// The event key for the Ethereum log data.
const EVENT_DATA_KEY: &str = "d";

/// The event keys for the Ethereum log topics.
const EVENT_TOPIC_KEYS: &[&str] = &["t1", "t2", "t3", "t4"];

#[inline]
pub fn log(
    state: &mut ExecutionState,
    system: &System<impl Runtime>,
    num_topics: usize,
    mem_index: U256,
    size: U256,
    topics: &[U256],
) -> Result<(), ActorError> {
    if system.readonly {
        return Err(ActorError::read_only("log called while read-only".into()));
    }

    // Handle the data.
    // Passing in a zero-sized memory region omits the data key entirely.
    // LOG0 + a zero-sized memory region emits an event with no entries whatsoever. In this case,
    // the FVM will record a hollow event carrying only the emitter actor ID.
    let region = get_memory_region(&mut state.memory, mem_index, size)?;

    // Extract the topics. Prefer to allocate an extra item than to incur in the cost of a
    // decision based on the size of the data.
    let mut entries: Vec<Entry> = Vec::with_capacity(num_topics + 1);
    for i in 0..num_topics {
        let key = EVENT_TOPIC_KEYS[i];
        let topic = topics[i];
        let entry = Entry {
            flags: Flags::FLAG_INDEXED_ALL,
            key: (*key).to_owned(),
            codec: IPLD_RAW,
            value: topic.to_bytes().into(), // U256 serializes as a byte string.
        };
        entries.push(entry);
    }

    // Skip adding the data if it's zero-sized.
    if let Some(r) = region {
        let data = state.memory[r.offset..r.offset + r.size.get()].to_vec();
        let entry = Entry {
            flags: Flags::FLAG_INDEXED_ALL,
            key: EVENT_DATA_KEY.to_owned(),
            codec: IPLD_RAW,
            value: data,
        };
        entries.push(entry);
    }

    system.rt.emit_event(&entries.into())?;

    Ok(())
}
