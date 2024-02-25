use redb::{Range, RedbValue};

pub struct IndexerFunction {}

impl IndexerFunction {
  pub(crate) fn get_next_sequence_number<T: RedbValue>(
    mut sequence_number_range: Range<u32, T>,
  ) -> u32 {
    let next_sequence_number = sequence_number_range
      .next_back()
      .and_then(|result| result.ok())
      .map(|(number, _id)| number.value() + 1)
      .unwrap_or(0);

    next_sequence_number
  }

  pub(crate) fn count_seq_data<T: RedbValue>(mut sequence_number_range: Range<u32, T>) -> u32 {
    let count = sequence_number_range
      .next_back()
      .map(|result| result.map(|(number, _entry)| number.value()))
      .transpose()
      .unwrap();

    if let Some(count_value) = count {
      count_value + 1
    } else {
      0
    }
  }
}
