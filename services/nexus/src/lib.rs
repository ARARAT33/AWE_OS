//! AWE-Nexus Microkernel IPC Service Bus
//!
//! Provides capability-gated service discovery, message routing, pub/sub topics,
//! and high-performance channel abstractions across microkernel processes.

#![no_std]

pub const MAX_SERVICES: usize = 16;
pub const MAX_TOPICS: usize = 16;
pub const MAX_SUBSCRIBERS_PER_TOPIC: usize = 8;
pub const MAX_QUEUE_DEPTH: usize = 8;
pub const MAX_PAYLOAD_SIZE: usize = 256;

/// Service endpoint identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceEndpoint {
    pub service_id: u32,
    pub capability_mask: u64,
}

impl ServiceEndpoint {
    pub const fn new(service_id: u32, capability_mask: u64) -> Self {
        Self {
            service_id,
            capability_mask,
        }
    }
}

/// AWE-Nexus IPC Message header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct NexusHeader {
    pub sender_id: u32,
    pub receiver_id: u32,
    pub opcode: u16,
    pub payload_len: u16,
    pub sequence_num: u32,
    pub required_capability: u64,
}

/// IPC Message payload buffer.
#[derive(Debug, Clone, Copy)]
pub struct NexusMessage {
    pub header: NexusHeader,
    pub payload: [u8; MAX_PAYLOAD_SIZE],
}

impl NexusMessage {
    pub fn new(header: NexusHeader, data: &[u8]) -> Result<Self, &'static str> {
        if data.len() > MAX_PAYLOAD_SIZE {
            return Err("Payload size exceeds maximum allowed buffer");
        }
        let mut payload = [0u8; MAX_PAYLOAD_SIZE];
        payload[..data.len()].copy_from_slice(data);
        let mut msg_header = header;
        msg_header.payload_len = data.len() as u16;
        Ok(Self {
            header: msg_header,
            payload,
        })
    }
}

/// Fixed-capacity IPC message queue.
#[derive(Debug, Clone, Copy)]
pub struct MessageRing {
    messages: [Option<NexusMessage>; MAX_QUEUE_DEPTH],
    head: usize,
    tail: usize,
    count: usize,
}

impl MessageRing {
    pub const fn new() -> Self {
        Self {
            messages: [None; MAX_QUEUE_DEPTH],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn enqueue(&mut self, msg: NexusMessage) -> Result<(), &'static str> {
        if self.count >= MAX_QUEUE_DEPTH {
            return Err("Queue overflow: backpressure triggered");
        }
        self.messages[self.tail] = Some(msg);
        self.tail = (self.tail + 1) % MAX_QUEUE_DEPTH;
        self.count += 1;
        Ok(())
    }

    pub fn dequeue(&mut self) -> Option<NexusMessage> {
        if self.count == 0 {
            return None;
        }
        let msg = self.messages[self.head].take();
        self.head = (self.head + 1) % MAX_QUEUE_DEPTH;
        self.count -= 1;
        msg
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

/// Pub/Sub topic definition.
#[derive(Debug, Clone, Copy)]
pub struct PubSubTopic {
    pub topic_id: u32,
    pub required_capability: u64,
    pub subscribers: [u32; MAX_SUBSCRIBERS_PER_TOPIC],
    pub subscriber_count: usize,
}

impl PubSubTopic {
    pub const fn new(topic_id: u32, required_capability: u64) -> Self {
        Self {
            topic_id,
            required_capability,
            subscribers: [0; MAX_SUBSCRIBERS_PER_TOPIC],
            subscriber_count: 0,
        }
    }

    pub fn subscribe(&mut self, service_id: u32, capability_mask: u64) -> Result<(), &'static str> {
        if (capability_mask & self.required_capability) != self.required_capability {
            return Err("Insufficient capability for topic subscription");
        }
        for i in 0..self.subscriber_count {
            if self.subscribers[i] == service_id {
                return Ok(());
            }
        }
        if self.subscriber_count >= MAX_SUBSCRIBERS_PER_TOPIC {
            return Err("Topic subscriber limit reached");
        }
        self.subscribers[self.subscriber_count] = service_id;
        self.subscriber_count += 1;
        Ok(())
    }
}

/// AWE-Nexus Core Microkernel Router.
#[derive(Debug)]
pub struct NexusRouter {
    endpoints: [Option<ServiceEndpoint>; MAX_SERVICES],
    queues: [MessageRing; MAX_SERVICES],
    topics: [Option<PubSubTopic>; MAX_TOPICS],
    active_services: usize,
    sequence_counter: u32,
}

impl NexusRouter {
    pub const fn new() -> Self {
        const EMPTY_QUEUE: MessageRing = MessageRing::new();
        Self {
            endpoints: [None; MAX_SERVICES],
            queues: [EMPTY_QUEUE; MAX_SERVICES],
            topics: [None; MAX_TOPICS],
            active_services: 0,
            sequence_counter: 1,
        }
    }

    pub fn register_service(&mut self, endpoint: ServiceEndpoint) -> Result<(), &'static str> {
        let idx = endpoint.service_id as usize;
        if idx >= MAX_SERVICES {
            return Err("Service ID out of bounds");
        }
        if self.endpoints[idx].is_some() {
            return Err("Service ID already registered");
        }
        self.endpoints[idx] = Some(endpoint);
        self.active_services += 1;
        Ok(())
    }

    pub fn send_message(&mut self, sender_id: u32, msg: NexusMessage) -> Result<(), &'static str> {
        let sender_idx = sender_id as usize;
        let recv_idx = msg.header.receiver_id as usize;

        if sender_idx >= MAX_SERVICES || recv_idx >= MAX_SERVICES {
            return Err("Invalid endpoint ID");
        }

        let sender_ep = self.endpoints[sender_idx].ok_or("Sender not registered")?;
        let _recv_ep = self.endpoints[recv_idx].ok_or("Receiver not registered")?;

        if (sender_ep.capability_mask & msg.header.required_capability)
            != msg.header.required_capability
        {
            return Err("Sender lacks required capability bit for message dispatch");
        }

        self.queues[recv_idx].enqueue(msg)?;
        self.sequence_counter += 1;
        Ok(())
    }

    pub fn receive_message(&mut self, service_id: u32) -> Option<NexusMessage> {
        let idx = service_id as usize;
        if idx >= MAX_SERVICES {
            return None;
        }
        self.queues[idx].dequeue()
    }

    pub fn create_topic(
        &mut self,
        topic_id: u32,
        required_capability: u64,
    ) -> Result<(), &'static str> {
        let idx = topic_id as usize;
        if idx >= MAX_TOPICS {
            return Err("Topic ID out of bounds");
        }
        if self.topics[idx].is_some() {
            return Err("Topic already exists");
        }
        self.topics[idx] = Some(PubSubTopic::new(topic_id, required_capability));
        Ok(())
    }

    pub fn publish_event(
        &mut self,
        sender_id: u32,
        topic_id: u32,
        data: &[u8],
    ) -> Result<usize, &'static str> {
        let topic_idx = topic_id as usize;
        if topic_idx >= MAX_TOPICS {
            return Err("Topic ID out of bounds");
        }
        let topic = self.topics[topic_idx].ok_or("Topic does not exist")?;

        let header = NexusHeader {
            sender_id,
            receiver_id: 0,
            opcode: 0xEE,
            payload_len: data.len() as u16,
            sequence_num: self.sequence_counter,
            required_capability: topic.required_capability,
        };

        let mut delivered = 0;
        for i in 0..topic.subscriber_count {
            let target_id = topic.subscribers[i];
            let mut msg = NexusMessage::new(header, data)?;
            msg.header.receiver_id = target_id;
            if self.send_message(sender_id, msg).is_ok() {
                delivered += 1;
            }
        }

        Ok(delivered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nexus_service_registration_and_routing() {
        let mut router = NexusRouter::new();
        let s1 = ServiceEndpoint::new(1, 0b11);
        let s2 = ServiceEndpoint::new(2, 0b11);

        assert!(router.register_service(s1).is_ok());
        assert!(router.register_service(s2).is_ok());

        let header = NexusHeader {
            sender_id: 1,
            receiver_id: 2,
            opcode: 0x01,
            payload_len: 4,
            sequence_num: 1,
            required_capability: 0b01,
        };
        let msg = NexusMessage::new(header, &[10, 20, 30, 40]).unwrap();

        assert!(router.send_message(1, msg).is_ok());

        let recv = router.receive_message(2).expect("Should receive message");
        assert_eq!(recv.header.sender_id, 1);
        assert_eq!(&recv.payload[..4], &[10, 20, 30, 40]);
    }

    #[test]
    fn test_nexus_capability_denial() {
        let mut router = NexusRouter::new();
        let s1 = ServiceEndpoint::new(1, 0b00); // No capabilities
        let s2 = ServiceEndpoint::new(2, 0b11);

        router.register_service(s1).unwrap();
        router.register_service(s2).unwrap();

        let header = NexusHeader {
            sender_id: 1,
            receiver_id: 2,
            opcode: 0x01,
            payload_len: 2,
            sequence_num: 1,
            required_capability: 0b10, // Requires cap 0b10
        };
        let msg = NexusMessage::new(header, &[1, 2]).unwrap();

        assert!(router.send_message(1, msg).is_err());
    }
}
