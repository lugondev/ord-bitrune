use bitcoin::{Network, OutPoint};
use ciborium::Value;
use serde::{Deserialize, Serialize};

use crate::InscriptionId;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InscriptionEntry {
  pub inscription_id: InscriptionId,
  pub network: Network,
  pub fee: u64,
  pub inscription_no: i32,
  pub info: InscriptionInfo,
  pub outpoint: OutPoint,
  pub seq_no: u32,
  pub height: u32,
  pub timestamp: u32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct InscriptionInfo {
  pub body: Option<String>,
  pub content_encoding: Option<String>,
  pub content_type: Option<String>,
  pub content_length: usize,
  pub metadata: Option<String>,
  pub metaprotocol: Option<String>,
  pub is_json: bool,
}

pub fn is_json_inscription_content(
  inscription_content_type_option: &Option<Vec<u8>>,
  inscription_content_option: &Option<Vec<u8>>,
) -> bool {
  if inscription_content_option.is_none()
    || inscription_content_type_option.is_none()
    || !is_text_inscription_content_type(inscription_content_type_option)
  {
    return false;
  }
  let inscription_content = inscription_content_option.as_ref().unwrap();
  let inscription_content_str = std::str::from_utf8(inscription_content).unwrap_or("");

  return serde_json::from_slice::<Value>(&inscription_content).is_ok()
    && inscription_content_str.contains("{")
    && inscription_content_str.contains("}");
}

pub fn is_text_inscription_content_type(inscription_content_type_option: &Option<Vec<u8>>) -> bool {
  if inscription_content_type_option.is_none() {
    return false;
  }

  let inscription_content_type = inscription_content_type_option.as_ref().unwrap();
  let inscription_content_type_str = std::str::from_utf8(&inscription_content_type).unwrap_or("");
  return inscription_content_type_str == "text/plain"
    || inscription_content_type_str.starts_with("text/plain;")
    || inscription_content_type_str == "application/json"
    || inscription_content_type_str.starts_with("application/json;"); // NOTE: added application/json for JSON5 etc.
}
// br-indexer: modified --> start
