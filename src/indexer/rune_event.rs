use bitcoin::hashes::Hash;
use bitcoin::{Network, ScriptBuf, Txid};
use serde::{Deserialize, Serialize};

use crate::index::entry::Entry;

use super::*;

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Serialize)]
pub enum RuneEvent {
  Mint,
  Transfer,
  Burn,
}

impl RuneEvent {
  pub fn from_u8(value: u8) -> Self {
    match value {
      0 => RuneEvent::Mint,
      1 => RuneEvent::Transfer,
      2 => RuneEvent::Burn,
      _ => RuneEvent::Transfer,
    }
  }

  pub fn to_u8(&self) -> u8 {
    match self {
      RuneEvent::Mint => 0,
      RuneEvent::Transfer => 1,
      RuneEvent::Burn => 2,
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RuneEventEntry {
  pub rune_id: RuneId,
  pub network: Network,
  pub event: RuneEvent,
  pub source: Txid,
  pub txid: Txid,
  pub height: u32,
  pub script_pubkey: ScriptBuf,
  pub amount: u128,
  pub timestamp: u32,
  pub vout: i32,
}

pub(crate) type RuneEventEntryValue = (
  (u64, u32),         // rune_id
  u8,                 // network
  u8,                 // event
  (u128, u128),       // source
  u32,                // height
  (u128, u128),       // txid
  (u128, u128, u128), // script_pubkey
  u128,               // amount
  u32,                // timestamp
  i32,                // vout
);

impl Entry for RuneEventEntry {
  type Value = RuneEventEntryValue;

  #[rustfmt::skip]
  fn load(
    (
      rune_id,
      network,
      event,
      source,
      height,
      txid,
      script_pubkey,
      amount,
      timestamp,
      vout,
    ): RuneEventEntryValue,
  ) -> Self {
    Self {
      rune_id: RuneId {
        block: rune_id.0,
        tx: rune_id.1,
      },
      network: match network {
        0 => Network::Bitcoin,
        1 => Network::Testnet,
        2 => Network::Regtest,
        _ => Network::Signet,
      },
      event: RuneEvent::from_u8(event),
      source: {
        let low = source.0.to_le_bytes();
        let high = source.1.to_le_bytes();
        Txid::from_byte_array([
          low[0], low[1], low[2], low[3], low[4], low[5], low[6], low[7], low[8], low[9], low[10],
          low[11], low[12], low[13], low[14], low[15], high[0], high[1], high[2], high[3], high[4],
          high[5], high[6], high[7], high[8], high[9], high[10], high[11], high[12], high[13],
          high[14], high[15],
        ])
      },
      height,
      txid: {
        let low = txid.0.to_le_bytes();
        let high = txid.1.to_le_bytes();
        Txid::from_byte_array([
          low[0], low[1], low[2], low[3], low[4], low[5], low[6], low[7], low[8], low[9], low[10],
          low[11], low[12], low[13], low[14], low[15], high[0], high[1], high[2], high[3], high[4],
          high[5], high[6], high[7], high[8], high[9], high[10], high[11], high[12], high[13],
          high[14], high[15],
        ])
      },
      script_pubkey: ScriptBuf::load(script_pubkey),
      amount,
      timestamp,
      vout,
    }
  }

  fn store(self) -> Self::Value {
    (
      (self.rune_id.block, self.rune_id.tx),
      match self.network {
        Network::Bitcoin => 0,
        Network::Testnet => 1,
        Network::Regtest => 2,
        _ => 3,
      },
      self.event.to_u8(),
      {
        let bytes = self.source.to_byte_array();
        (
          u128::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
          ]),
          u128::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
            bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
          ]),
        )
      },
      self.height,
      {
        let bytes = self.txid.to_byte_array();
        (
          u128::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
          ]),
          u128::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
            bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
          ]),
        )
      },
      self.script_pubkey.store(),
      self.amount,
      self.timestamp,
      self.vout,
    )
  }
}
