use crate::Chain;

pub struct IndexerHeight {}

impl IndexerHeight {
  pub(crate) fn get_first_height(
    chain: Chain,
    index_inscriptions: bool,
    first_inscription_height: u32,
    first_rune: u32,
  ) -> u32 {
    let mut height = first_inscription_height;
    if chain == Chain::Testnet {
      if !index_inscriptions {
        height = 2583200; // first testnet block which has rune
      }
      return height;
    } else if chain == Chain::Signet {
      if !index_inscriptions {
        height = 188710; // first signet block which has rune
      }
      return height;
    } else if chain == Chain::Mainnet {
      if !index_inscriptions {
        height = 840000; // first mainnet block which has rune
      }
      return height;
    }
    if !index_inscriptions {
      height = first_rune;
    }
    height
  }
}
