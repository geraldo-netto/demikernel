// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//==============================================================================
// Imports
//==============================================================================

use super::runtime::SharedDummyRuntime;
use ::demikernel::{
    demi_sgarray_t,
    demikernel::libos::network::libos::SharedNetworkLibOS,
    inetstack::SharedInetStack,
    runtime::{
        fail::Fail,
        logging,
        memory::{
            DemiBuffer,
            MemoryRuntime,
        },
        network::{
            config::{
                ArpConfig,
                TcpConfig,
                UdpConfig,
            },
            types::MacAddress,
        },
        QDesc,
        QToken,
        SharedDemiRuntime,
    },
    OperationResult,
};
use crossbeam_channel::{
    Receiver,
    Sender,
};
use std::{
    collections::HashMap,
    net::Ipv4Addr,
    ops::{
        Deref,
        DerefMut,
    },
    time::{
        Duration,
        Instant,
    },
};

//==============================================================================
// Structures
//==============================================================================

pub struct DummyLibOS(SharedNetworkLibOS<SharedInetStack<SharedDummyRuntime>>);

//==============================================================================
// Associated Functons
//==============================================================================

impl DummyLibOS {
    /// Initializes the libOS.
    pub fn new(
        link_addr: MacAddress,
        ipv4_addr: Ipv4Addr,
        tx: Sender<DemiBuffer>,
        rx: Receiver<DemiBuffer>,
        arp: HashMap<Ipv4Addr, MacAddress>,
    ) -> Result<Self, Fail> {
        let runtime: SharedDemiRuntime = SharedDemiRuntime::default();
        let arp_config: ArpConfig = ArpConfig::new(
            Some(Duration::from_secs(600)),
            Some(Duration::from_secs(1)),
            Some(2),
            Some(arp.clone()),
            Some(false),
        );
        let udp_config: UdpConfig = UdpConfig::default();
        let tcp_config: TcpConfig = TcpConfig::default();
        let network: SharedDummyRuntime = SharedDummyRuntime::new(rx, tx, arp_config, tcp_config, udp_config);

        logging::initialize();
        let transport = SharedInetStack::new_test(runtime.clone(), network, link_addr, ipv4_addr)?;
        Ok(Self(SharedNetworkLibOS::<SharedInetStack<SharedDummyRuntime>>::new(
            runtime, transport,
        )))
    }

    /// Cooks a buffer.
    pub fn cook_data(&self, size: usize) -> Result<demi_sgarray_t, Fail> {
        let fill_char: u8 = b'a';

        let mut buf: DemiBuffer = DemiBuffer::new(size as u16);
        for a in &mut buf[..] {
            *a = fill_char;
        }
        let data: demi_sgarray_t = self.get_transport().into_sgarray(buf)?;
        Ok(data)
    }

    #[allow(dead_code)]
    pub fn wait(&mut self, qt: QToken, timeout: Duration) -> Result<(QDesc, OperationResult), Fail> {
        let now: Instant = Instant::now();
        // Run for one second.
        while !self.get_runtime().has_completed(qt)? && Instant::now() - now < timeout {
            self.get_runtime().poll();
        }
        match self.get_runtime().remove_coroutine(qt).get_result() {
            Some(result) => Ok(result),
            None => Err(Fail::new(libc::ETIMEDOUT, "wait timed out after one second")),
        }
    }
}

//======================================================================================================================
// Trait Implementations
//======================================================================================================================

impl Deref for DummyLibOS {
    type Target = SharedNetworkLibOS<SharedInetStack<SharedDummyRuntime>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DummyLibOS {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
