use super::*;
use crate::indexer::rune_event::{RuneEvent, RuneEventEntry, RuneEventEntryValue};

pub(super) struct RuneUpdater<'a, 'tx, 'client> {
  pub(super) chain: Chain,
  pub(super) sequence_number_to_rune_event: &'a mut Table<'tx, u32, RuneEventEntryValue>,
  pub(super) next_sequence_number_rune_event: u32,
  pub(super) block_time: u32,
  pub(super) burned: HashMap<RuneId, Lot>,
  pub(super) client: &'client Client,
  pub(super) height: u32,
  pub(super) id_to_entry: &'a mut Table<'tx, RuneIdValue, RuneEntryValue>,
  pub(super) inscription_id_to_sequence_number: &'a Table<'tx, InscriptionIdValue, u32>,
  pub(super) minimum: Rune,
  pub(super) outpoint_to_balances: &'a mut Table<'tx, &'static OutPointValue, &'static [u8]>,
  pub(super) rune_to_id: &'a mut Table<'tx, u128, RuneIdValue>,
  pub(super) runes: u64,
  pub(super) sequence_number_to_rune_id: &'a mut Table<'tx, u32, RuneIdValue>,
  pub(super) statistic_to_count: &'a mut Table<'tx, u64, u64>,
  pub(super) transaction_id_to_rune: &'a mut Table<'tx, &'static TxidValue, u128>,
}

// @br-indexer - create rune event --> start
pub fn rune_event(rune_actions: &mut HashMap<RuneId, RuneEvent>, id: RuneId, event: RuneEvent) {
  if rune_actions.entry(id).or_insert(RuneEvent::Mint).to_u8() == 0 {
    rune_actions.insert(id, event);
  }
}
// @br-indexer - create rune event --> end

impl<'a, 'tx, 'client> RuneUpdater<'a, 'tx, 'client> {
  pub(super) fn index_runes(&mut self, tx_index: u32, tx: &Transaction, txid: Txid) -> Result<()> {
    let artifact = Runestone::decipher(tx);

    let mut rune_actions: HashMap<RuneId, RuneEvent> = HashMap::new(); // @br-indexer
    let mut rune_inputs: HashMap<RuneId, Vec<OutPoint>> = HashMap::new(); // @br-indexer
    let mut unallocated = self.unallocated(tx, txid, &mut rune_actions, &mut rune_inputs)?;

    let mut allocated: Vec<HashMap<RuneId, Lot>> = vec![HashMap::new(); tx.output.len()];

    if let Some(artifact) = &artifact {
      if let Some(id) = artifact.mint() {
        if let Some(amount) = self.mint(id)? {
          *unallocated.entry(id).or_default() += amount;
        }
      }

      let etched = self.etched(tx_index, tx, artifact)?;

      if let Artifact::Runestone(runestone) = artifact {
        if let Some((id, ..)) = etched {
          *unallocated.entry(id).or_default() +=
            runestone.etching.unwrap().premine.unwrap_or_default();
        }

        for Edict { id, amount, output } in runestone.edicts.iter().copied() {
          let amount = Lot(amount);

          // edicts with output values greater than the number of outputs
          // should never be produced by the edict parser
          let output = usize::try_from(output).unwrap();
          assert!(output <= tx.output.len());

          let id = if id == RuneId::default() {
            let Some((id, ..)) = etched else {
              continue;
            };

            id
          } else {
            id
          };

          let Some(balance) = unallocated.get_mut(&id) else {
            continue;
          };

          let mut allocate = |balance: &mut Lot, amount: Lot, output: usize| {
            if amount > 0 {
              *balance -= amount;
              *allocated[output].entry(id).or_default() += amount;
            }
          };

          if output == tx.output.len() {
            // find non-OP_RETURN outputs
            let destinations = tx
              .output
              .iter()
              .enumerate()
              .filter_map(|(output, tx_out)| {
                (!tx_out.script_pubkey.is_op_return()).then_some(output)
              })
              .collect::<Vec<usize>>();

            if amount == 0 {
              // if amount is zero, divide balance between eligible outputs
              let amount = *balance / destinations.len() as u128;
              let remainder = usize::try_from(*balance % destinations.len() as u128).unwrap();

              for (i, output) in destinations.iter().enumerate() {
                allocate(
                  balance,
                  if i < remainder { amount + 1 } else { amount },
                  *output,
                );
              }
            } else {
              // if amount is non-zero, distribute amount to eligible outputs
              for output in destinations {
                allocate(balance, amount.min(*balance), output);
              }
            }
          } else {
            // Get the allocatable amount
            let amount = if amount == 0 {
              *balance
            } else {
              amount.min(*balance)
            };

            allocate(balance, amount, output);
          }
        }
      }

      if let Some((id, rune)) = etched {
        self.create_rune_entry(txid, artifact, id, rune)?;
      }
    }

    let mut burned: HashMap<RuneId, Lot> = HashMap::new();

    if let Some(Artifact::Cenotaph(_)) = artifact {
      for (id, balance) in unallocated {
        *burned.entry(id).or_default() += balance;
      }
    } else {
      let pointer = artifact
        .map(|artifact| match artifact {
          Artifact::Runestone(runestone) => runestone.pointer,
          Artifact::Cenotaph(_) => unreachable!(),
        })
        .unwrap_or_default();

      // assign all un-allocated runes to the default output, or the first non
      // OP_RETURN output if there is no default, or if the default output is
      // too large
      if let Some(vout) = pointer
        .map(|pointer| pointer.into_usize())
        .inspect(|&pointer| assert!(pointer < allocated.len()))
        .or_else(|| {
          tx.output
            .iter()
            .enumerate()
            .find(|(_vout, tx_out)| !tx_out.script_pubkey.is_op_return())
            .map(|(vout, _tx_out)| vout)
        })
      {
        for (id, balance) in unallocated {
          if balance > 0 {
            *allocated[vout].entry(id).or_default() += balance;
          }
        }
      } else {
        for (id, balance) in unallocated {
          if balance > 0 {
            *burned.entry(id).or_default() += balance;
          }
        }
      }
    }

    // update outpoint balances
    let mut buffer: Vec<u8> = Vec::new();
    for (vout, balances) in allocated.into_iter().enumerate() {
      if balances.is_empty() {
        continue;
      }

      // increment burned balances
      if tx.output[vout].script_pubkey.is_op_return() {
        for (id, balance) in &balances {
          *burned.entry(*id).or_default() += *balance;
        }
        continue;
      }

      buffer.clear();

      let mut balances = balances.into_iter().collect::<Vec<(RuneId, Lot)>>();

      // Sort balances by id so tests can assert balances in a fixed order
      balances.sort();

      for (id, balance) in balances {
        Index::encode_rune_balance(id, balance.n(), &mut buffer);
        // @br-indexer - new output --> start
        let next_sequence_number_rune_event = self.next_sequence_number_rune_event;
        self.next_sequence_number_rune_event += 1;
        self.sequence_number_to_rune_event.insert(
          next_sequence_number_rune_event,
          &RuneEventEntry {
            rune_id: id,
            network: self.chain.network(),
            event: *rune_actions.entry(id).or_insert(RuneEvent::Mint),
            txid,
            source: txid,
            script_pubkey: tx.output[vout].script_pubkey.clone(),
            amount: balance.n(),
            vout: i32::try_from(vout).unwrap(),
            timestamp: self.block_time,
          }
          .store(),
        )?;
        // @br-indexer - new output --> end
      }

      self.outpoint_to_balances.insert(
        &OutPoint {
          txid,
          vout: vout.try_into().unwrap(),
        }
        .store(),
        buffer.as_slice(),
      )?;
    }

    // increment entries with burned runes
    for (id, amount) in burned {
      *self.burned.entry(id).or_default() += amount;
      // @br-indexer burn rune --> end
      if rune_inputs.entry(id).or_default().is_empty() {
        let next_sequence_number_rune_event = self.next_sequence_number_rune_event;
        self.next_sequence_number_rune_event += 1;
        self.sequence_number_to_rune_event.insert(
          next_sequence_number_rune_event,
          &RuneEventEntry {
            rune_id: id,
            network: self.chain.network(),
            event: RuneEvent::Burn,
            txid,
            amount: amount.n(),
            source: txid,
            script_pubkey: ScriptBuf::default(),
            vout: -1,
            timestamp: self.block_time,
          }
          .store(),
        )?;
      } else {
        for input in rune_inputs.entry(id).or_default().iter() {
          let next_sequence_number_rune_event = self.next_sequence_number_rune_event;
          self.next_sequence_number_rune_event += 1;
          self.sequence_number_to_rune_event.insert(
            next_sequence_number_rune_event,
            &RuneEventEntry {
              rune_id: RuneId::try_from(id).unwrap(),
              network: self.chain.network(),
              event: RuneEvent::Burn,
              source: txid,
              txid: input.txid,
              amount: amount.n(),
              script_pubkey: ScriptBuf::default(),
              vout: i32::try_from(input.vout).unwrap(),
              timestamp: self.block_time,
            }
            .store(),
          )?;
        }
      }
      // br-indexer --> end
    }

    Ok(())
  }

  pub(super) fn update(self) -> Result {
    for (rune_id, burned) in self.burned {
      let mut entry = RuneEntry::load(self.id_to_entry.get(&rune_id.store())?.unwrap().value());
      entry.burned = entry.burned.checked_add(burned.n()).unwrap();
      self.id_to_entry.insert(&rune_id.store(), entry.store())?;
    }

    Ok(())
  }

  fn create_rune_entry(
    &mut self,
    txid: Txid,
    artifact: &Artifact,
    id: RuneId,
    rune: Rune,
  ) -> Result {
    self.rune_to_id.insert(rune.store(), id.store())?;
    self
      .transaction_id_to_rune
      .insert(&txid.store(), rune.store())?;

    let number = self.runes;
    self.runes += 1;

    self
      .statistic_to_count
      .insert(&Statistic::Runes.into(), self.runes)?;

    let entry = match artifact {
      Artifact::Cenotaph(_) => RuneEntry {
        block: id.block,
        burned: 0,
        divisibility: 0,
        etching: txid,
        terms: None,
        mints: 0,
        number,
        premine: 0,
        spaced_rune: SpacedRune { rune, spacers: 0 },
        symbol: None,
        timestamp: self.block_time.into(),
      },
      Artifact::Runestone(Runestone { etching, .. }) => {
        let Etching {
          divisibility,
          terms,
          premine,
          spacers,
          symbol,
          ..
        } = etching.unwrap();

        RuneEntry {
          block: id.block,
          burned: 0,
          divisibility: divisibility.unwrap_or_default(),
          etching: txid,
          terms,
          mints: 0,
          number,
          premine: premine.unwrap_or_default(),
          spaced_rune: SpacedRune {
            rune,
            spacers: spacers.unwrap_or_default(),
          },
          symbol,
          timestamp: self.block_time.into(),
        }
      }
    };

    self.id_to_entry.insert(id.store(), entry.store())?;

    let inscription_id = InscriptionId { txid, index: 0 };

    if let Some(sequence_number) = self
      .inscription_id_to_sequence_number
      .get(&inscription_id.store())?
    {
      self
        .sequence_number_to_rune_id
        .insert(sequence_number.value(), id.store())?;
    }

    Ok(())
  }

  fn etched(
    &mut self,
    tx_index: u32,
    tx: &Transaction,
    artifact: &Artifact,
  ) -> Result<Option<(RuneId, Rune)>> {
    let rune = match artifact {
      Artifact::Runestone(runestone) => match runestone.etching {
        Some(etching) => etching.rune,
        None => return Ok(None),
      },
      Artifact::Cenotaph(cenotaph) => match cenotaph.etching {
        Some(rune) => Some(rune),
        None => return Ok(None),
      },
    };

    let rune = if let Some(rune) = rune {
      if rune < self.minimum
        || rune.is_reserved()
        || self.rune_to_id.get(rune.0)?.is_some()
        || !self.tx_commits_to_rune(tx, rune)?
      {
        return Ok(None);
      }
      rune
    } else {
      let reserved_runes = self
        .statistic_to_count
        .get(&Statistic::ReservedRunes.into())?
        .map(|entry| entry.value())
        .unwrap_or_default();

      self
        .statistic_to_count
        .insert(&Statistic::ReservedRunes.into(), reserved_runes + 1)?;

      Rune::reserved(self.height.into(), tx_index)
    };

    Ok(Some((
      RuneId {
        block: self.height.into(),
        tx: tx_index,
      },
      rune,
    )))
  }

  fn mint(&mut self, id: RuneId) -> Result<Option<Lot>> {
    let Some(entry) = self.id_to_entry.get(&id.store())? else {
      return Ok(None);
    };

    let mut rune_entry = RuneEntry::load(entry.value());

    let Ok(amount) = rune_entry.mintable(self.height.into()) else {
      return Ok(None);
    };

    drop(entry);

    rune_entry.mints += 1;

    self.id_to_entry.insert(&id.store(), rune_entry.store())?;

    Ok(Some(Lot(amount)))
  }

  fn tx_commits_to_rune(&self, tx: &Transaction, rune: Rune) -> Result<bool> {
    let commitment = rune.commitment();

    for input in &tx.input {
      // extracting a tapscript does not indicate that the input being spent
      // was actually a taproot output. this is checked below, when we load the
      // output's entry from the database
      let Some(tapscript) = input.witness.tapscript() else {
        continue;
      };

      for instruction in tapscript.instructions() {
        // ignore errors, since the extracted script may not be valid
        let Ok(instruction) = instruction else {
          break;
        };

        let Some(pushbytes) = instruction.push_bytes() else {
          continue;
        };

        if pushbytes.as_bytes() != commitment {
          continue;
        }

        let Some(tx_info) = self
          .client
          .get_raw_transaction_info(&input.previous_output.txid, None)
          .into_option()?
        else {
          panic!("input not in UTXO set: {}", input.previous_output);
        };

        let taproot = tx_info.vout[input.previous_output.vout.into_usize()]
          .script_pub_key
          .script()?
          .is_v1_p2tr();

        let mature = tx_info
          .confirmations
          .map(|confirmations| confirmations >= Runestone::COMMIT_INTERVAL.into())
          .unwrap_or_default();

        if taproot && mature {
          return Ok(true);
        }
      }
    }

    Ok(false)
  }

  fn unallocated(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    rune_actions: &mut HashMap<RuneId, RuneEvent>,
    rune_inputs: &mut HashMap<RuneId, Vec<OutPoint>>,
  ) -> Result<HashMap<RuneId, Lot>> {
    // map of rune ID to un-allocated balance of that rune
    let mut unallocated: HashMap<RuneId, Lot> = HashMap::new();

    // increment unallocated runes with the runes in tx inputs
    for input in &tx.input {
      if let Some(guard) = self
        .outpoint_to_balances
        .remove(&input.previous_output.store())?
      {
        let buffer = guard.value();
        let mut i = 0;
        while i < buffer.len() {
          let ((id, balance), len) = Index::decode_rune_balance(&buffer[i..]).unwrap();
          i += len;
          *unallocated.entry(id).or_default() += balance;

          // @br-indexer - new input --> start
          rune_inputs
            .entry(id)
            .or_default()
            .push(input.previous_output);

          rune_event(rune_actions, id, RuneEvent::Transfer);
          let next_sequence_number_rune_event = self.next_sequence_number_rune_event;
          self.next_sequence_number_rune_event += 1;
          self.sequence_number_to_rune_event.insert(
            next_sequence_number_rune_event,
            &RuneEventEntry {
              rune_id: RuneId::try_from(id).unwrap(),
              network: self.chain.network(),
              event: RuneEvent::Used,
              source: txid,
              txid: input.previous_output.txid,
              script_pubkey: ScriptBuf::default(),
              amount: balance,
              vout: i32::try_from(input.previous_output.vout).unwrap(),
              timestamp: self.block_time,
            }
            .store(),
          )?;
          // @br-indexer - new input --> end
        }
      }
    }

    Ok(unallocated)
  }
}
