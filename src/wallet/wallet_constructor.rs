use super::*;

#[derive(Clone)]
pub(crate) struct WalletConstructor {
  ord_client: reqwest::blocking::Client,
  name: String,
  no_sync: bool,
  rpc_url: Url,
  settings: Settings,
}

impl WalletConstructor {
  pub(crate) fn construct(
    name: String,
    no_sync: bool,
    settings: Settings,
    rpc_url: Url,
  ) -> Result<Wallet> {
    let mut headers = HeaderMap::new();
    headers.insert(
      header::ACCEPT,
      header::HeaderValue::from_static("application/json"),
    );

    if let Some((username, password)) = settings.credentials() {
      let credentials =
        base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
      headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Basic {credentials}")).unwrap(),
      );
    }

    Self {
      ord_client: reqwest::blocking::ClientBuilder::new()
        .default_headers(headers.clone())
        .build()?,
      name,
      no_sync,
      rpc_url,
      settings,
    }
    .build()
  }

  pub(crate) fn build(self) -> Result<Wallet> {
    let database = Wallet::open_database(&self.name, &self.settings)?;

    let bitcoin_client = {
      let client =
        Wallet::check_version(self.settings.bitcoin_rpc_client(Some(self.name.clone()))?)?;

      if !client.list_wallets()?.contains(&self.name) {
        client.load_wallet(&self.name)?;
      }

      if client.get_wallet_info()?.private_keys_enabled {
        Wallet::check_descriptors(&self.name, client.list_descriptors(None)?.descriptors)?;
      }

      client
    };

    let chain_block_count = bitcoin_client.get_block_count().unwrap() + 1;

    if !self.no_sync {
      for i in 0.. {
        let response = self.get("/blockcount")?;

        if response
          .text()?
          .parse::<u64>()
          .expect("wallet failed to talk to server. Make sure `ord server` is running.")
          >= chain_block_count
        {
          break;
        } else if i == 20 {
          bail!("wallet failed to synchronize with `ord server` after {i} attempts");
        }
        std::thread::sleep(Duration::from_millis(50));
      }
    }

    let mut utxos = Self::get_utxos(&bitcoin_client)?;
    let locked_utxos = Self::get_locked_utxos(&bitcoin_client)?;
    utxos.extend(locked_utxos.clone());

    let output_info = self.get_output_info(utxos.clone().into_keys().collect())?;

    let inscriptions = output_info
      .iter()
      .flat_map(|(_output, info)| info.inscriptions.clone())
      .collect::<Vec<InscriptionId>>();

    let (inscriptions, inscription_info) = self.get_inscriptions(&inscriptions)?;

    let status = self.get_server_status()?;

    Ok(Wallet {
      bitcoin_client,
      database,
      has_rune_index: status.rune_index,
      has_sat_index: status.sat_index,
      inscription_info,
      inscriptions,
      locked_utxos,
      ord_client: self.ord_client,
      output_info,
      rpc_url: self.rpc_url,
      settings: self.settings,
      utxos,
    })
  }

  fn get_output_info(&self, outputs: Vec<OutPoint>) -> Result<BTreeMap<OutPoint, api::Output>> {
    let response = self.post("/outputs", &outputs)?;

    if !response.status().is_success() {
      bail!("wallet failed get outputs: {}", response.text()?);
    }

    let output_info: BTreeMap<OutPoint, api::Output> = outputs
      .into_iter()
      .zip(serde_json::from_str::<Vec<api::Output>>(&response.text()?)?)
      .collect();

    for (output, info) in &output_info {
      if !info.indexed {
        bail!("output in wallet but not in ord server: {output}");
      }
    }

    Ok(output_info)
  }

  fn get_inscriptions(
    &self,
    inscriptions: &Vec<InscriptionId>,
  ) -> Result<(
    BTreeMap<SatPoint, Vec<InscriptionId>>,
    BTreeMap<InscriptionId, api::Inscription>,
  )> {
    let response = self.post("/inscriptions", inscriptions)?;

    if !response.status().is_success() {
      bail!("wallet failed get inscriptions: {}", response.text()?);
    }

    let mut inscriptions = BTreeMap::new();
    let mut inscription_infos = BTreeMap::new();
    for info in serde_json::from_str::<Vec<api::Inscription>>(&response.text()?)? {
      inscriptions
        .entry(info.satpoint)
        .or_insert_with(Vec::new)
        .push(info.id);

      inscription_infos.insert(info.id, info);
    }

    Ok((inscriptions, inscription_infos))
  }

  fn get_utxos(bitcoin_client: &Client) -> Result<BTreeMap<OutPoint, TxOut>> {
    Ok(
      bitcoin_client
        .list_unspent(None, None, None, None, None)?
        .into_iter()
        .map(|utxo| {
          let outpoint = OutPoint::new(utxo.txid, utxo.vout);
          let txout = TxOut {
            script_pubkey: utxo.script_pub_key,
            value: utxo.amount.to_sat(),
          };

          (outpoint, txout)
        })
        .collect(),
    )
  }

  fn get_locked_utxos(bitcoin_client: &Client) -> Result<BTreeMap<OutPoint, TxOut>> {
    #[derive(Deserialize)]
    pub(crate) struct JsonOutPoint {
      txid: Txid,
      vout: u32,
    }

    let outpoints = bitcoin_client.call::<Vec<JsonOutPoint>>("listlockunspent", &[])?;

    let mut utxos = BTreeMap::new();

    for outpoint in outpoints {
      let Some(tx_out) = bitcoin_client.get_tx_out(&outpoint.txid, outpoint.vout, Some(false))?
      else {
        continue;
      };

      utxos.insert(
        OutPoint::new(outpoint.txid, outpoint.vout),
        TxOut {
          value: tx_out.value.to_sat(),
          script_pubkey: ScriptBuf::from_bytes(tx_out.script_pub_key.hex),
        },
      );
    }

    Ok(utxos)
  }

  fn get_server_status(&self) -> Result<api::Status> {
    let response = self.get("/status")?;

    if !response.status().is_success() {
      bail!("could not get status: {}", response.text()?)
    }

    Ok(serde_json::from_str(&response.text()?)?)
  }

  pub fn get(&self, path: &str) -> Result<reqwest::blocking::Response> {
    self
      .ord_client
      .get(self.rpc_url.join(path)?)
      .send()
      .map_err(|err| anyhow!(err))
  }

  pub fn post(&self, path: &str, body: &impl Serialize) -> Result<reqwest::blocking::Response> {
    self
      .ord_client
      .post(self.rpc_url.join(path)?)
      .json(body)
      .header(reqwest::header::ACCEPT, "application/json")
      .send()
      .map_err(|err| anyhow!(err))
  }
}
