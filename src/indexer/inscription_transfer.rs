use bitcoin::hashes::Hash;
use bitcoin::{Network, ScriptBuf, Txid};
use serde::{Deserialize, Serialize};

use crate::index::entry::{Entry, InscriptionIdValue};
use crate::InscriptionId;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InscriptionTransfer {
  pub inscription_id: InscriptionId,
  pub network: Network,
  pub txid: Txid,
  pub to_script_pubkey: ScriptBuf,
  pub output_value: u64,
  pub height: u32,
  pub index: u32,
  pub vout: u32,
}

pub(crate) type InscriptionTransferValue = (
  InscriptionIdValue, // inscription_id
  u8,                 // network
  (u128, u128),       // txid
  (u128, u128, u128), // to_script_pubkey
  u64,                // output_value
  u32,                // height
  u32,                // index
  u32,                // vout
);

impl Entry for InscriptionTransfer {
  type Value = InscriptionTransferValue;

  #[rustfmt::skip]
  fn load(
    (
      inscription_id,
      network,
      txid,
      script_pubkey,
      output_value,
      height,
      index,
      vout,
    ): InscriptionTransferValue,
  ) -> Self {
    Self {
      inscription_id: InscriptionId::load(inscription_id),
      network: match network {
        0 => Network::Bitcoin,
        1 => Network::Testnet,
        2 => Network::Regtest,
        _ => Network::Signet,
      },
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
      to_script_pubkey: ScriptBuf::load(script_pubkey),
      output_value,
      height,
      index,
      vout,
    }
  }

  fn store(self) -> Self::Value {
    (
      self.inscription_id.store(),
      match self.network {
        Network::Bitcoin => 0,
        Network::Testnet => 1,
        Network::Regtest => 2,
        _ => 3,
      },
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
      ScriptBuf::store(self.to_script_pubkey),
      self.output_value,
      self.height,
      self.index,
      self.vout,
    )
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_inscription_transfer() {}
}
