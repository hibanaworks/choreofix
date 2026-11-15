//! In-process transport for one experimental execution.

use std::{
    cell::Cell,
    collections::VecDeque,
    task::{Context, Poll, Waker},
};

use hibana::runtime::{
    ids::SessionId,
    transport::{
        EndpointBinding, FrameHeader, Outgoing, PortOpen, ReceivedFrame, Transport, TransportError,
    },
    wire::Payload,
};

pub struct InMemoryTransport {
    state: Cell<IoQueues>,
    bindings: Vec<InitialBinding>,
}

struct InitialBinding {
    session: SessionId,
    role: u8,
    binding: EndpointBinding,
}

impl InMemoryTransport {
    pub fn new(bindings: impl IntoIterator<Item = (SessionId, u8, EndpointBinding)>) -> Self {
        Self {
            state: Cell::new(IoQueues::default()),
            bindings: bindings
                .into_iter()
                .map(|(session, role, binding)| InitialBinding {
                    session,
                    role,
                    binding,
                })
                .collect(),
        }
    }

    /// One immutable record per actual transport acceptance, including its payload.
    /// Endpoint receive commits belong to the complete EPF producer capture.
    pub fn take_observations(
        &self,
    ) -> impl DoubleEndedIterator<Item = serde_json::Value> + ExactSizeIterator {
        let mut state = self.state.take();
        let observations = std::mem::take(&mut state.observations);
        self.state.set(state);
        observations.into_iter().map(|bytes| {
            serde_json::from_slice(&bytes).expect("transport serialized this complete JSON record")
        })
    }
}

#[derive(Default)]
struct IoQueues {
    frames: VecDeque<Frame>,
    waiters: Vec<Waiter>,
    next_frame: u64,
    // Retain each immutable record as bytes, not one heap Value per payload byte.
    // Draining parses one record at a time and preserves every observation.
    observations: Vec<Vec<u8>>,
}

struct Frame {
    id: u64,
    session: SessionId,
    lane: u8,
    source: u8,
    target: u8,
    label: u16,
    payload: Vec<u8>,
}

impl Frame {
    fn observation(&self) -> Vec<u8> {
        serde_json::to_vec(
            &serde_json::json!({"type":"protocol_frame", "action":"send",
            "frame_id":self.id, "session":self.session.raw(), "lane":self.lane,
            "source":self.source, "target":self.target, "frame_label":self.label,
            "payload":self.payload}),
        )
        .expect("a transport observation is serializable JSON")
    }

    fn received(&self) -> ReceivedFrame<'_> {
        let session = self.session.raw().to_be_bytes();
        let label = self.label.to_be_bytes();
        let header = FrameHeader::from_bytes([
            session[0],
            session[1],
            session[2],
            session[3],
            self.lane,
            self.source,
            self.target,
            label[0],
            label[1],
        ]);
        ReceivedFrame::framed(header, Payload::new(&self.payload))
    }

    fn is_for(&self, rx: &Rx) -> bool {
        self.session == rx.session && self.lane == rx.lane && self.target == rx.local_role
    }
}

struct Waiter {
    session: SessionId,
    lane: u8,
    local_role: u8,
    waker: Waker,
}

impl Waiter {
    fn is_for_frame(&self, frame: &Frame) -> bool {
        self.session == frame.session && self.lane == frame.lane && self.local_role == frame.target
    }

    fn is_for_rx(&self, rx: &Rx) -> bool {
        self.session == rx.session && self.lane == rx.lane && self.local_role == rx.local_role
    }
}

pub struct Tx {
    session: SessionId,
    lane: u8,
    local_role: u8,
}

pub struct Rx {
    session: SessionId,
    lane: u8,
    local_role: u8,
    current: Option<Frame>,
}

impl Transport for &InMemoryTransport {
    fn initial_endpoint_binding(&self, session: SessionId, role: u8) -> Option<&EndpointBinding> {
        self.bindings
            .iter()
            .find(|binding| binding.session == session && binding.role == role)
            .map(|binding| &binding.binding)
    }

    type Tx<'a>
        = Tx
    where
        Self: 'a;
    type Rx<'a>
        = Rx
    where
        Self: 'a;

    fn open(&self, port: PortOpen) -> (Self::Tx<'_>, Self::Rx<'_>) {
        let session = port.session_id();
        let lane = port.lane();
        let local_role = port.local_role();
        (
            Tx {
                session,
                lane,
                local_role,
            },
            Rx {
                session,
                lane,
                local_role,
                current: None,
            },
        )
    }

    fn poll_send<'f>(
        &self,
        tx: &mut Tx,
        outgoing: Outgoing<'f>,
        _context: &mut Context<'_>,
    ) -> Poll<Result<(), TransportError>> {
        if outgoing.lane() != tx.lane {
            return Poll::Ready(Err(TransportError::Failed));
        }
        let mut frame = Frame {
            id: 0,
            session: tx.session,
            lane: tx.lane,
            source: tx.local_role,
            target: outgoing.target_role(),
            label: outgoing.frame_label().raw(),
            payload: outgoing.payload().as_bytes().to_vec(),
        };
        let wake = {
            let mut state = self.state.take();
            frame.id = state.next_frame;
            state.next_frame += 1;
            state.observations.push(frame.observation());
            let wake = state
                .waiters
                .iter()
                .position(|waiter| waiter.is_for_frame(&frame))
                .map(|index| state.waiters.swap_remove(index).waker);
            state.frames.push_back(frame);
            self.state.set(state);
            wake
        };
        if let Some(waker) = wake {
            waker.wake();
        }
        Poll::Ready(Ok(()))
    }

    fn cancel_send(&self, _tx: &mut Tx) {
        // Sends complete atomically and never retain pending payload state.
    }

    fn poll_recv<'a>(
        &'a self,
        rx: &'a mut Rx,
        context: &mut Context<'_>,
    ) -> Poll<Result<ReceivedFrame<'a>, TransportError>> {
        rx.current = None;
        let next_waker = context.waker().clone();
        let (frame, replaced_waker) = {
            let mut state = self.state.take();
            let frame = state
                .frames
                .iter()
                .position(|frame| frame.is_for(rx))
                .and_then(|index| state.frames.remove(index));
            let replaced_waker = if frame.is_none() {
                if let Some(waiter) = state.waiters.iter_mut().find(|waiter| waiter.is_for_rx(rx)) {
                    Some(std::mem::replace(&mut waiter.waker, next_waker))
                } else {
                    state.waiters.push(Waiter {
                        session: rx.session,
                        lane: rx.lane,
                        local_role: rx.local_role,
                        waker: next_waker,
                    });
                    None
                }
            } else {
                None
            };
            self.state.set(state);
            (frame, replaced_waker)
        };
        drop(replaced_waker);
        match frame {
            Some(frame) => {
                rx.current = Some(frame);
                Poll::Ready(Ok(rx.current.as_ref().expect("stored frame").received()))
            }
            None => Poll::Pending,
        }
    }

    fn requeue(&self, rx: &mut Rx) -> Result<(), TransportError> {
        let frame = rx.current.take().ok_or(TransportError::Failed)?;
        let mut state = self.state.take();
        let wake = state
            .waiters
            .iter()
            .position(|waiter| waiter.is_for_frame(&frame))
            .map(|index| state.waiters.swap_remove(index).waker);
        state.frames.push_front(frame);
        self.state.set(state);
        if let Some(waker) = wake {
            waker.wake();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hibana::{
        g,
        runtime::{SessionKitStorage, program::project},
    };

    #[hibana_runtime::main]
    #[test]
    async fn endpoint_commit_remains_distinct_from_repeated_offers() {
        use hibana::runtime::tap;
        type Message = g::Msg<1, u32>;
        let source = g::send::<1, 2, Message>();
        let sender_program = project::<1>(&source);
        let receiver_program = project::<2>(&source);
        let transport = InMemoryTransport::new([]);
        let observed = Cell::new(Vec::new());
        let capture = |event: tap::TapEvent| {
            let mut events = observed.take();
            events.push(event);
            observed.set(events);
        };
        let mut slab = [0; 16 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &transport).unwrap();
        rv.observe(&capture);
        let session = SessionId::new(12);
        let mut sender = rv.enter(session, &sender_program).unwrap();
        let mut receiver = rv.enter(session, &receiver_program).unwrap();
        sender.send::<Message>(&42).await.unwrap();
        for _ in 0..64 {
            drop(receiver.offer().await.unwrap());
        }
        let before = observed.take();
        assert!(before.iter().all(|event| event.id() != tap::ENDPOINT_RECV));
        observed.set(before);
        assert_eq!(receiver.recv::<Message>().await.unwrap(), 42);
        let events = observed.take();
        assert_eq!(
            events
                .iter()
                .filter(|event| event.id() == tap::ENDPOINT_SEND)
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| event.id() == tap::ENDPOINT_RECV)
                .count(),
            1
        );
        let mut accepted = transport.take_observations();
        assert_eq!(accepted.len(), 1);
        assert_eq!(
            accepted.next().unwrap()["payload"],
            serde_json::json!(42u32.to_be_bytes())
        );
    }

    #[hibana_runtime::main]
    #[test]
    async fn repeated_previews_preserve_the_one_actual_transport_acceptance() {
        type Message<'a> = g::Msg<1, &'a [u8]>;
        let source = g::send::<1, 2, Message>();
        let program = project::<1>(&source);
        let transport = InMemoryTransport::new([]);
        let session = SessionId::new(11);
        let mut slab = [0; 16 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &transport).unwrap();
        let mut sender = rv.enter(session, &program).unwrap();
        let payload = vec![90u8; 4096];
        sender.send::<Message>(&payload.as_slice()).await.unwrap();
        let state = transport.state.take();
        let sent = state.frames.front().unwrap();
        let mut rx = Rx {
            session,
            lane: sent.lane,
            local_role: sent.target,
            current: None,
        };
        transport.state.set(state);
        let mut context = Context::from_waker(Waker::noop());
        for _ in 0..64 {
            assert!(matches!(
                (&transport).poll_recv(&mut rx, &mut context),
                Poll::Ready(Ok(_))
            ));
            (&transport).requeue(&mut rx).unwrap();
        }
        let state = transport.state.take();
        assert_eq!(
            state.frames.len(),
            1,
            "previewing never duplicates the frame"
        );
        assert_eq!(state.observations.len(), 1);
        let retained: usize = state.observations.iter().map(Vec::capacity).sum();
        let expanded_payloads =
            state.observations.len() * payload.len() * core::mem::size_of::<serde_json::Value>();
        eprintln!(
            "one complete acceptance after 64 previews: retained_bytes={retained}, expanded_payload_bytes={expanded_payloads}"
        );
        assert!(
            retained < expanded_payloads / 4,
            "retained={retained}, expanded payload arrays alone={expanded_payloads}"
        );
        transport.state.set(state);
        let mut records = transport.take_observations();
        assert_eq!(records.len(), 1);
        let expected = serde_json::json!(payload);
        for record in records.by_ref() {
            assert_eq!(record["frame_id"], 0);
            assert_eq!(record["payload"], expected);
            assert_eq!(record["action"], "send");
        }
        assert!(records.next().is_none());
        assert!(transport.take_observations().next().is_none());
    }
}
