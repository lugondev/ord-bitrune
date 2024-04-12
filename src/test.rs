pub(crate) use {
  super::*,
  bitcoin::{
    blockdata::script::{PushBytes, PushBytesBuf},
    constants::COIN_VALUE,
    opcodes, WPubkeyHash,
  },
  mockcore::TransactionTemplate,
  pretty_assertions::assert_eq as pretty_assert_eq,
  std::iter,
  tempfile::TempDir,
  unindent::Unindent,
};

pub(crate) fn txid(n: u64) -> Txid {
  let hex = format!("{n:x}");

  if hex.is_empty() || hex.len() > 1 {
    panic!();
  }

  hex.repeat(64).parse().unwrap()
}

pub(crate) fn outpoint(n: u64) -> OutPoint {
  format!("{}:{}", txid(n), n).parse().unwrap()
}

pub(crate) fn satpoint(n: u64, offset: u64) -> SatPoint {
  SatPoint {
    outpoint: outpoint(n),
    offset,
  }
}

pub(crate) fn address() -> Address {
  "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
    .parse::<Address<NetworkUnchecked>>()
    .unwrap()
    .assume_checked()
}

pub(crate) fn recipient() -> Address {
  "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
    .parse::<Address<NetworkUnchecked>>()
    .unwrap()
    .assume_checked()
}

pub(crate) fn change(n: u64) -> Address {
  match n {
    0 => "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww",
    1 => "tb1qakxxzv9n7706kc3xdcycrtfv8cqv62hnwexc0l",
    2 => "tb1qxz9yk0td0yye009gt6ayn7jthz5p07a75luryg",
    3 => "tb1qe62s57n77pfhlw2vtqlhm87dwj75l6fguavjjq",
    _ => panic!(),
  }
  .parse::<Address<NetworkUnchecked>>()
  .unwrap()
  .assume_checked()
}

pub(crate) fn tx_in(previous_output: OutPoint) -> TxIn {
  TxIn {
    previous_output,
    script_sig: ScriptBuf::new(),
    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
    witness: Witness::new(),
  }
}

pub(crate) fn tx_out(value: u64, address: Address) -> TxOut {
  TxOut {
    value,
    script_pubkey: address.script_pubkey(),
  }
}

#[derive(Default, Debug)]
pub(crate) struct InscriptionTemplate {
  pub(crate) parents: Vec<InscriptionId>,
  pub(crate) pointer: Option<u64>,
}

impl From<InscriptionTemplate> for Inscription {
  fn from(template: InscriptionTemplate) -> Self {
    Self {
      parents: template.parents.into_iter().map(|id| id.value()).collect(),
      pointer: template.pointer.map(Inscription::pointer_value),
      ..default()
    }
  }
}

pub(crate) fn inscription(content_type: &str, body: impl AsRef<[u8]>) -> Inscription {
  Inscription {
    content_type: Some(content_type.into()),
    body: Some(body.as_ref().into()),
    ..default()
  }
}

pub(crate) fn inscription_id(n: u32) -> InscriptionId {
  let hex = format!("{n:x}");

  if hex.is_empty() || hex.len() > 1 {
    panic!();
  }

  format!("{}i{n}", hex.repeat(64)).parse().unwrap()
}

pub(crate) fn envelope(payload: &[&[u8]]) -> Witness {
  let mut builder = script::Builder::new()
    .push_opcode(opcodes::OP_FALSE)
    .push_opcode(opcodes::all::OP_IF);

  for data in payload {
    let mut buf = PushBytesBuf::new();
    buf.extend_from_slice(data).unwrap();
    builder = builder.push_slice(buf);
  }

  let script = builder.push_opcode(opcodes::all::OP_ENDIF).into_script();

  Witness::from_slice(&[script.into_bytes(), Vec::new()])
}

pub(crate) fn default_address(chain: Chain) -> Address {
  Address::from_script(
    &ScriptBuf::new_v0_p2wpkh(&WPubkeyHash::all_zeros()),
    chain.network(),
  )
  .unwrap()
}
