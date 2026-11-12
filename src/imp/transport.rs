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

    /// Append-only observations; no observer can authorize an operation.
    pub fn take_observations(&self) -> Vec<serde_json::Value> {
        let mut state = self.state.take();
        let observations = std::mem::take(&mut state.observations);
        self.state.set(state);
        observations
    }
}

#[derive(Default)]
struct IoQueues {
    frames: VecDeque<Frame>,
    waiters: Vec<Waiter>,
    next_frame: u64,
    observations: Vec<serde_json::Value>,
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
    fn observation(&self, action: &str) -> serde_json::Value {
        serde_json::json!({"type":"protocol_frame", "action":action,
            "frame_id":self.id, "session":self.session.raw(), "lane":self.lane,
            "source":self.source, "target":self.target, "frame_label":self.label,
            "payload":self.payload})
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
            state.observations.push(frame.observation("send"));
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
            if let Some(frame) = &frame {
                state.observations.push(frame.observation("receive"));
            }
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
        state.observations.push(frame.observation("requeue"));
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
