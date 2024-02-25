use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatusHtml {
  pub blessed_inscriptions: u64,
  pub chain: Chain,
  pub content_type_counts: Vec<(Option<Vec<u8>>, u64)>,
  pub cursed_inscriptions: u64,
  pub height: Option<u32>,
  pub inscriptions: u64,
  pub lost_sats: u64,
  pub minimum_rune_for_next_block: Rune,
  pub rune_index: bool,
  pub runes: u64,
  pub sat_index: bool,
  pub started: DateTime<Utc>,
  pub transaction_index: bool,
  pub unrecoverably_reorged: bool,
  pub uptime: Duration,
}

impl PageContent for StatusHtml {
  fn title(&self) -> String {
    "Status".into()
  }
}
