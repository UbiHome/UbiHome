use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use smoltcp::phy::{Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

/// Queue of inbound IP packets decapsulated from the tunnel, awaiting injection
/// into the smoltcp stack. Filled by the WireGuard produce task, drained by the
/// virtual interface poll loop through [`VirtualIpDevice::receive`].
pub type InboundQueue = Arc<Mutex<VecDeque<Vec<u8>>>>;

/// A smoltcp [`Device`] whose "wire" is the WireGuard tunnel: received frames come
/// from the inbound queue, transmitted frames are handed to the WireGuard consume
/// task over an unbounded channel to be encapsulated and sent.
pub struct VirtualIpDevice {
    inbound: InboundQueue,
    outbound: UnboundedSender<Vec<u8>>,
    mtu: usize,
}

impl VirtualIpDevice {
    pub fn new(inbound: InboundQueue, outbound: UnboundedSender<Vec<u8>>, mtu: usize) -> Self {
        Self {
            inbound,
            outbound,
            mtu,
        }
    }
}

impl Device for VirtualIpDevice {
    type RxToken<'a>
        = RxToken
    where
        Self: 'a;
    type TxToken<'a>
        = TxToken
    where
        Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let next = self
            .inbound
            .lock()
            .expect("inbound queue poisoned")
            .pop_front();
        next.map(|buffer| {
            (
                RxToken { buffer },
                TxToken {
                    outbound: self.outbound.clone(),
                },
            )
        })
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(TxToken {
            outbound: self.outbound.clone(),
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.medium = Medium::Ip;
        cap.max_transmission_unit = self.mtu;
        cap
    }
}

pub struct RxToken {
    buffer: Vec<u8>,
}

impl smoltcp::phy::RxToken for RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.buffer)
    }
}

pub struct TxToken {
    outbound: UnboundedSender<Vec<u8>>,
}

impl smoltcp::phy::TxToken for TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = vec![0u8; len];
        let result = f(&mut buffer);
        let _ = self.outbound.send(buffer);
        result
    }
}
