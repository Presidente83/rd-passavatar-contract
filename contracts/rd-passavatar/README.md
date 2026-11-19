# RD Station — Avatar Club 100

Questa specifica sostituisce la precedente proposta Genesis 10 a pagamento. La produzione resta disattivata finché il contratto, l'asset, le verifiche e il rilascio non sono pronti.

## Regole della collezione

- Supply massima: 100 token, AV-000001–AV-000100.
- Ogni Founding Pass #001–#100 dà diritto a un claim gratuito, una sola volta per Pass ID. Il diritto segue il Pass e vale per il titolare on-chain al momento del mint.
- Il Founder approva ogni richiesta e firma manualmente con NFT000 tramite Keplr. NFT000 paga la fee di rete.
- NFT000 è escluso dai claim della collezione, anche se dovesse possedere un Pass #001–#100.
- Lupo Biancoverde è un NFT legacy separato: non usa uno dei 100 slot e non impedisce i claim degli altri holder. Un eventuale Avatar Base per NFT000 sarà definito e creato separatamente.
- Ogni wallet può ricevere al massimo un Avatar Club 100. Il contratto verifica ownership del Pass, claim già consumato, cap e occupazione del wallet nella collezione.
- Wallet senza Pass possono solo consultare lo Store. Il Pass sblocca le funzioni della dApp previste dal progetto.
- Il contratto non deve custodire seed o chiavi private; Supabase registra solo richieste e audit. L'ownership e il claim completato sono verificati on-chain.

## Interfaccia operativa e sicurezza

La coda amministrativa è accessibile solo con sessione Founder autenticata. L'approvazione della richiesta e la firma Keplr sono due passaggi distinti. Prima della firma la schermata mostra Pass ID, beneficiario, Avatar ID previsto, rete e commissione stimata; la transazione deve essere firmata da NFT000. Il client verifica che Keplr non abbia modificato messaggio, beneficiario o fee rispetto alla preparazione.

La richiesta viene registrata come broadcast prima dell'invio per permettere la riconciliazione dopo un timeout. Il completamento è mostrato solo dopo la verifica del claim e dell'owner on-chain. In caso di esito ambiguo, non ritentare alla cieca: riconciliare la transazione prima.

La proxy RPC accetta per il flusso Avatar soltanto le letture necessarie e le transazioni sponsorizzate consentite. Il contratto deve rifiutare ogni richiesta non proveniente dal Founder, senza fee o token inviati dal beneficiario, con Pass eleggibile ancora posseduto dal beneficiario.

## Configurazione

- PASSAVATAR_GENESIS_ENABLED resta `false` finché release candidate, deploy e configurazione non sono verificati.
- PASSAVATAR_GENESIS_CONTRACT, PASSAVATAR_GENESIS_CODE_ID e PASSAVATAR_GENESIS_WASM_SHA256 identificano il contratto esatto.
- La rete ammessa è Terra Classic `columbus-5`; checksum Wasm, chain ID, creator, Founder, contratto Founding Pass, hash immagine e supply sono controllati server-side.
- Il mint Base ha prezzo `0`; la fee di rete è pagata dal wallet NFT000 e viene mostrata prima della firma.
- Nessuna chiave privata del Founder viene inserita in variabili d'ambiente o applicazioni server.

Il mercato secondario resta disattivato. Qualsiasi futura funzione che consenta trasferimenti o vendita richiederà una decisione e una revisione separate.

## Rilascio

Prima di abilitare i claim occorre:
1. completare build riproducibile e revisione del contratto aggiornato a 100 claim sponsorizzati;
2. verificare immagine, metadata e checksum approvati;
3. applicare in produzione la migrazione della coda solo tramite il processo di rilascio;
4. distribuire e verificare il contratto su una rete isolata/testnet;
5. convalidare autenticazione, doppio invio, cambio titolare, approvazione/rifiuto, firma Founder, fee, timeout e riconciliazione;
6. configurare Vercel solo quando il contratto esatto è deployato e verificato;
7. abilitare la produzione con una transazione keplr firmata dal Founder.

Il merge e il deploy non sostituiscono la firma manuale on-chain. Nessun claim mainnet va effettuato senza una conferma esplicita del Founder nella schermata Keplr.
