use std::sync::Arc;

use async_trait::async_trait;
use log::debug;
use smol::Executor;

use crate::{util::async_util, Result};

use super::{
    super::{
        message, message_subscriber::MessageSubscription, ChannelPtr, HostsPtr, P2pPtr,
        SettingsPtr, SESSION_OUTBOUND,
    },
    ProtocolBase, ProtocolBasePtr, ProtocolJobsManager, ProtocolJobsManagerPtr,
};

const SEND_ADDR_SLEEP_SECONDS: u64 = 900;

/// Defines address and get-address messages.
pub struct ProtocolAddress {
    channel: ChannelPtr,
    addrs_sub: MessageSubscription<message::AddrsMessage>,
    ext_addrs_sub: MessageSubscription<message::ExtAddrsMessage>,
    get_addrs_sub: MessageSubscription<message::GetAddrsMessage>,
    hosts: HostsPtr,
    jobsman: ProtocolJobsManagerPtr,
    settings: SettingsPtr,
}

impl ProtocolAddress {
    /// Create a new address protocol. Makes an address, an external address
    /// and a  get-address subscription and adds them to the address protocol instance.
    pub async fn init(channel: ChannelPtr, p2p: P2pPtr) -> ProtocolBasePtr {
        let settings = p2p.settings();
        let hosts = p2p.hosts();

        // Creates a subscription to address message.
        let addrs_sub = channel
            .clone()
            .subscribe_msg::<message::AddrsMessage>()
            .await
            .expect("Missing addrs dispatcher!");

        // Creates a subscription to external address message.
        let ext_addrs_sub = channel
            .clone()
            .subscribe_msg::<message::ExtAddrsMessage>()
            .await
            .expect("Missing ext_addrs dispatcher!");

        // Creates a subscription to get-address message.
        let get_addrs_sub = channel
            .clone()
            .subscribe_msg::<message::GetAddrsMessage>()
            .await
            .expect("Missing getaddrs dispatcher!");

        Arc::new(Self {
            channel: channel.clone(),
            addrs_sub,
            ext_addrs_sub,
            get_addrs_sub,
            hosts,
            jobsman: ProtocolJobsManager::new("ProtocolAddress", channel),
            settings,
        })
    }

    /// Handles receiving the address message. Loops to continually recieve
    /// address messages on the address subsciption. Adds the recieved
    /// addresses to the list of hosts.
    async fn handle_receive_addrs(self: Arc<Self>) -> Result<()> {
        debug!(target: "net", "ProtocolAddress::handle_receive_addrs() [START]");
        loop {
            let addrs_msg = self.addrs_sub.receive().await?;
            debug!(target: "net", "ProtocolAddress::handle_receive_addrs() received {} addrs", addrs_msg.addrs.len());
            self.hosts.store(addrs_msg.addrs.clone()).await;
        }
    }

    /// Handles receiving the external address message. Loops to continually recieve
    /// external address messages on the address subsciption. Adds the recieved
    /// external addresses to the list of hosts.
    async fn handle_receive_ext_addrs(self: Arc<Self>) -> Result<()> {
        debug!(target: "net", "ProtocolAddress::handle_receive_ext_addrs() [START]");
        loop {
            let ext_addrs_msg = self.ext_addrs_sub.receive().await?;
            debug!(target: "net", "ProtocolAddress::handle_receive_ext_addrs() received {} addrs", ext_addrs_msg.ext_addrs.len());
            self.hosts.store_ext(self.channel.address(), ext_addrs_msg.ext_addrs.clone()).await;
        }
    }

    /// Handles receiving the get-address message. Continually recieves
    /// get-address messages on the get-address subsciption. Then replies
    /// with an address message.
    async fn handle_receive_get_addrs(self: Arc<Self>) -> Result<()> {
        debug!(target: "net", "ProtocolAddress::handle_receive_get_addrs() [START]");
        loop {
            let _get_addrs = self.get_addrs_sub.receive().await?;

            debug!(target: "net", "ProtocolAddress::handle_receive_get_addrs() received GetAddrs message");

            // Loads the list of hosts.
            let addrs = self.hosts.load_all().await;
            debug!(target: "net", "ProtocolAddress::handle_receive_get_addrs() sending {} addrs", addrs.len());
            // Creates an address messages containing host address.
            let addrs_msg = message::AddrsMessage { addrs };
            // Sends the address message across the channel.
            self.channel.clone().send(addrs_msg).await?;
        }
    }

    async fn send_my_addrs(self: Arc<Self>) -> Result<()> {
        debug!(target: "net", "ProtocolAddress::send_addrs() [START]");
        loop {
            let ext_addrs = self.settings.external_addr.clone();
            let ext_addr_msg = message::ExtAddrsMessage { ext_addrs };
            self.channel.clone().send(ext_addr_msg).await?;
            async_util::sleep(SEND_ADDR_SLEEP_SECONDS).await;
        }
    }
}

#[async_trait]
impl ProtocolBase for ProtocolAddress {
    /// Starts the address protocol. Runs receive address and get address
    /// protocols on the protocol task manager. Then sends get-address
    /// message.
    async fn start(self: Arc<Self>, executor: Arc<Executor<'_>>) -> Result<()> {
        let type_id = self.channel.session_type_id();

        // if it's an outbound session + has an external address
        // send our address
        if type_id == SESSION_OUTBOUND && !self.settings.external_addr.is_empty() {
            self.jobsman.clone().start(executor.clone());
            self.jobsman.clone().spawn(self.clone().send_my_addrs(), executor.clone()).await;
        }

        debug!(target: "net", "ProtocolAddress::start() [START]");
        self.jobsman.clone().start(executor.clone());
        self.jobsman.clone().spawn(self.clone().handle_receive_addrs(), executor.clone()).await;
        self.jobsman.clone().spawn(self.clone().handle_receive_ext_addrs(), executor.clone()).await;
        self.jobsman.clone().spawn(self.clone().handle_receive_get_addrs(), executor).await;

        // Send get_address message.
        let get_addrs = message::GetAddrsMessage {};
        let _ = self.channel.clone().send(get_addrs).await;
        debug!(target: "net", "ProtocolAddress::start() [END]");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ProtocolAddress"
    }
}
