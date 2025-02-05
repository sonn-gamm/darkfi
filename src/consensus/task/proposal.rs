use std::time::Duration;

use async_std::sync::Arc;
use log::{debug, error, info};
use smol::Executor;

use super::{consensus_sync_task, keep_alive_task};
use crate::{
    consensus::{Participant, ValidatorStatePtr},
    net::P2pPtr,
    util::async_util::sleep,
};

/// async task used for participating in the consensus protocol
pub async fn proposal_task(
    consensus_p2p: P2pPtr,
    sync_p2p: P2pPtr,
    state: ValidatorStatePtr,
    ex: Arc<Executor<'_>>,
) {
    // TODO: [PLACEHOLDER] Add balance proof creation

    // Node waits just before the current or next slot end, so it can
    // start syncing latest state.
    let mut seconds_until_next_slot = state.read().await.next_n_slot_start(1);
    let one_sec = Duration::new(1, 0);

    loop {
        if seconds_until_next_slot > one_sec {
            seconds_until_next_slot -= one_sec;
            break
        }

        info!("consensus: Waiting for next slot ({:?} sec)", seconds_until_next_slot);
        sleep(seconds_until_next_slot.as_secs()).await;
        seconds_until_next_slot = state.read().await.next_n_slot_start(1);
    }

    info!("consensus: Waiting for next slot ({:?} sec)", seconds_until_next_slot);
    sleep(seconds_until_next_slot.as_secs()).await;

    // Node syncs its consensus state
    if let Err(e) = consensus_sync_task(consensus_p2p.clone(), state.clone()).await {
        error!("Failed syncing consensus state: {}. Quitting consensus.", e);
        // TODO: Perhaps notify over a channel in order to
        // stop consensus p2p protocols.
        return
    };

    // Node signals the network that it will start participating
    let public = state.read().await.public;
    let address = state.read().await.address;
    let cur_slot = state.read().await.current_slot();
    let participant = Participant::new(public, address, cur_slot);
    state.write().await.append_participant(participant.clone());

    match consensus_p2p.broadcast(participant).await {
        Ok(()) => info!("consensus: Participation message broadcasted successfully."),
        Err(e) => error!("Failed broadcasting consensus participation: {}", e),
    }

    // Node initiates the background task to send keep alive messages
    match keep_alive_task(consensus_p2p.clone(), state.clone(), ex).await {
        Ok(()) => info!("consensus: Keep alive background task initiated successfully."),
        Err(e) => error!("Failed to initiate keep alive background task: {}", e),
    }

    // Node modifies its participating slot to next.
    match state.write().await.set_participating() {
        Ok(()) => info!("consensus: Node will start participating in the next slot"),
        Err(e) => error!("Failed to set participation slot: {}", e),
    }

    loop {
        let seconds_next_slot = state.read().await.next_n_slot_start(1).as_secs();
        info!("consensus: Waiting for next slot ({} sec)", seconds_next_slot);
        sleep(seconds_next_slot).await;

        // Node refreshes participants records
        match state.write().await.refresh_participants() {
            Ok(()) => debug!("Participants refreshed successfully."),
            Err(e) => error!("Failed refreshing consensus participants: {}", e),
        }

        // Node checks if it's the slot leader to generate a new proposal
        // for that slot.
        let result = if state.write().await.is_slot_leader() {
            state.read().await.propose()
        } else {
            Ok(None)
        };

        let proposal = match result {
            Ok(prop) => {
                if prop.is_none() {
                    info!("consensus: Node is not the slot lead");
                    continue
                }
                prop.unwrap()
            }
            Err(e) => {
                error!("consensus: Block proposal failed: {}", e);
                continue
            }
        };

        info!("consensus: Node is the slot leader: Proposed block: {}", proposal);
        debug!("consensus: Full proposal: {:?}", proposal);
        match state.write().await.receive_proposal(&proposal).await {
            Ok(to_broadcast) => {
                info!("consensus: Block proposal  saved successfully");
                // Broadcast block to other consensus nodes
                match consensus_p2p.broadcast(proposal).await {
                    Ok(()) => info!("consensus: Proposal broadcasted successfully"),
                    Err(e) => error!("consensus: Failed broadcasting proposal: {}", e),
                }
                // Broadcast finalized blocks info, if any:
                if let Some(blocks) = to_broadcast {
                    info!("consensus: Broadcasting finalized blocks");
                    for info in blocks {
                        match sync_p2p.broadcast(info).await {
                            Ok(()) => info!("consensus: Broadcasted block"),
                            Err(e) => error!("consensus: Failed broadcasting block: {}", e),
                        }
                    }
                } else {
                    info!("consensus: No finalized blocks to broadcast");
                }
            }
            Err(e) => {
                error!("consensus: Block proposal save failed: {}", e);
            }
        }
    }
}
