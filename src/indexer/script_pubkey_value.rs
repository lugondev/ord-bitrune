use crate::index::entry::Entry;
use bitcoin::ScriptBuf;

pub type ScriptPubkeyValue = (u128, u128, u128);

impl Entry for ScriptBuf {
  type Value = ScriptPubkeyValue;

  fn load(value: Self::Value) -> Self {
    let low = value.0.to_le_bytes();
    let mid = value.1.to_le_bytes();
    let high = value.2.to_le_bytes();
    ScriptBuf::from_bytes(vec![
      low[0], low[1], low[2], low[3], low[4], low[5], low[6], low[7], low[8], low[9], low[10],
      low[11], low[12], low[13], low[14], low[15], mid[0], mid[1], mid[2], mid[3], mid[4], mid[5],
      mid[6], mid[7], mid[8], mid[9], mid[10], mid[11], mid[12], mid[13], mid[14], mid[15],
      high[0], high[1], high[2], high[3], high[4], high[5], high[6], high[7], high[8], high[9],
      high[10], high[11], high[12], high[13], high[14], high[15],
    ])
  }

  fn store(self) -> Self::Value {
    let bytes = self.to_bytes();
    let new_bytes = if bytes.len() < 48 {
      let mut new_bytes = vec![0; 48];
      new_bytes[..bytes.len()].copy_from_slice(&bytes);
      new_bytes
    } else {
      bytes
    };
    (
      u128::from_le_bytes([
        new_bytes[0],
        new_bytes[1],
        new_bytes[2],
        new_bytes[3],
        new_bytes[4],
        new_bytes[5],
        new_bytes[6],
        new_bytes[7],
        new_bytes[8],
        new_bytes[9],
        new_bytes[10],
        new_bytes[11],
        new_bytes[12],
        new_bytes[13],
        new_bytes[14],
        new_bytes[15],
      ]),
      u128::from_le_bytes([
        new_bytes[16],
        new_bytes[17],
        new_bytes[18],
        new_bytes[19],
        new_bytes[20],
        new_bytes[21],
        new_bytes[22],
        new_bytes[23],
        new_bytes[24],
        new_bytes[25],
        new_bytes[26],
        new_bytes[27],
        new_bytes[28],
        new_bytes[29],
        new_bytes[30],
        new_bytes[31],
      ]),
      u128::from_le_bytes([
        new_bytes[32],
        new_bytes[33],
        new_bytes[34],
        new_bytes[35],
        new_bytes[36],
        new_bytes[37],
        new_bytes[38],
        new_bytes[39],
        new_bytes[40],
        new_bytes[41],
        new_bytes[42],
        new_bytes[43],
        new_bytes[44],
        new_bytes[45],
        new_bytes[46],
        new_bytes[47],
      ]),
    )
  }
}
