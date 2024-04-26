use super::*;
use crate::indexer::inscription_entries::InscriptionEntry;
use crate::indexer::inscription_transfer::InscriptionTransfer;
use crate::indexer::rune_event::{BlockId, RuneEventResponse};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RunesEventsJson {
  pub events: Vec<(BlockId, RuneEventResponse)>,
  pub total: u64,
  pub block: u64,
  pub size: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InscriptionsTransfersJson {
  pub transfers: Vec<(u32, String, InscriptionTransfer)>,
  pub total: u32,
  pub page_index: u32,
  pub page_size: u32,
  pub more: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InscriptionsEntriesJson {
  pub entries: Vec<(u32, Option<InscriptionEntry>)>,
  pub total: u32,
  pub page_index: u32,
  pub page_size: u32,
  pub more: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct StatsUpdaterJson {
  pub network: Network,
  pub height: u32,
  pub runes: u32,
  pub inscriptions: u32,
  pub runes_events: u64,
  pub inscriptions_transfer: u32,
}
