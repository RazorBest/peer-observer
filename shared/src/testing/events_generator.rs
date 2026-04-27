use crate::proptest::prelude::{any, prop_compose, Strategy};
use crate::proptest::strategy::ValueTree;
use crate::proptest::test_runner::TestRunner;
use crate::protobuf::event::event::PeerObserverEvent;
use crate::protobuf::event::Event;

prop_compose! {
    fn event_sample()(peer_observer_evt in any::<PeerObserverEvent>()) -> Event {
        Event::new(peer_observer_evt).unwrap()
    }
}

pub fn generate_events(count: usize) -> impl Iterator<Item = Event> {
    generate_proptest_samples(event_sample(), count)
}

pub fn generate_proptest_samples<T: Sized>(
    strategy: impl Strategy<Value = T>,
    count: usize,
) -> impl Iterator<Item = T> {
    let mut runner = TestRunner::default();
    (0..count).map(move |_| strategy.new_tree(&mut runner).unwrap().current())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::protobuf::ebpf_extractor::ebpf;

    #[test]
    /// Dummy test
    fn test_event_generation() {
        for event in generate_events(10) {
            let _name = match event.peer_observer_event.as_ref().unwrap() {
                PeerObserverEvent::EbpfExtractor(e) => match e.ebpf_event.as_ref().unwrap() {
                    ebpf::EbpfEvent::Message(_) => "messages",
                    ebpf::EbpfEvent::Connection(_) => "connections",
                    ebpf::EbpfEvent::Mempool(_) => "mempool",
                    ebpf::EbpfEvent::Validation(_) => "validation",
                },
                PeerObserverEvent::RpcExtractor(_) => "rpc",
                PeerObserverEvent::P2pExtractor(_) => "p2p_extractor",
                PeerObserverEvent::LogExtractor(_) => "log_extractor",
            };
        }
    }
}
