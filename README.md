# SolDag

Multithreaded Solana data aggregator

## Design

SolDag is split into 2 main services, an indexer and a REST API. They both run parallel to each other on separate threads leveraging tokio.

Either one will continue to run in the event one of them fails. This is to ensure the indexer continues to fetch and track block data in the case the API goes down and vice versa. In the case where either of the services fails, a restart will be attempted 3 times before exiting.

### Indexer

The indexer service runs on three separate threads that communicate with each other via channels.

- The main thread fetches blocks at a configurable interval (defaulted to 400ms) `start`
- The second thread processes the retrieved blocks `process_block`
- The third thread figures out if any blocks were missed and sends a message to the second thread to process and store the blocks `catch_up`

### API

The API is a REST api leveraging the axum framework

### Database

The application uses a noSQL MongoDB to store and query indexed data

### Testing

There's a testing module to validate fuctionality of the application. Can be run with `cargo test`

## Usage

- Install [Rust](https://www.rust-lang.org/tools/install).

- Install [MongoDB](https://www.mongodb.com/docs/manual/installation/) and start your mongodb service.

- Create your `.env` file from the sample `.env.example` and replace the placeholders with your api key and mongodb url

- Start the aggregator

  ```console
  $ cargo run -- --api-listen 127.0.0.1:3004
  info: SolDag started, initializing services....
  info: Starting indexer service...
  info: Starting API server on 127.0.0.1:3004
  info: Latest block slot: 326296506
  info: Latest block slot: 326296513
  info: Missing 7 blocks 326296506 -> 326296513
  info: Block Slot: 326296506 stored
  info: Block Slot: 326296513 stored
  ```

  - See full help information with the `--help` flag

    ```console
    Solana data aggregator

    Usage: soldag [OPTIONS]

    Options:
      -k, --rpc-api-key <RPC_API_KEY>
              Helios RPC API key [env: RPC_API_KEY=86164f7e-4ac9-4af3-be93-912badf39f7d]
      -r, --rpc-url <RPC_URL>
              Solana RPC endpoint [default: https://mainnet.helius-rpc.com]
      -u, --update-interval <UPDATE_INTERVAL>
              Aggregator update interval in milliseconds [default: 400]
      -a, --api-listen <API_LISTEN>
              API server listen address [default: 127.0.0.1:8081]
      -h, --help
              Print help
      -V, --version
              Print version
    ```

  <br>

- In another terminal, make `curl` requests to fetch data from the API
  - Request for transactions. The transaction API endpoint is paginated

    ```console
    curl "127.0.0.1:3004/transactions?offset=0&count=2"
    ```

    <details>
    <summary>
    Truncated sample response
    </summary>
    ```json
    {
      "data": [
        {
          "signature": "27buXrMwymMGpH7f7hwVCfZKYn43qTJbrbLdL2TFoUqJjLjrBKBFJLBM6cwMWvqCvge5uZGMD67Zo3547zY3yfdA",
          "message": {
            "header": {
              "numRequiredSignatures": 1,
              "numReadonlySignedAccounts": 0,
              "numReadonlyUnsignedAccounts": 8
            },
            "accountKeys": [
              "4aRX4tq2mm5XS2PUUtJPcXUPvgrza5jvjKmoMZzUKcLM",
              "FrgX4DwXo4oUqLHXQBptgGzmDD3n6QuoJMwg4vsShJB3",
              "DsNE5dwdxycrSBtcbwJBDL6PX5zyHzCE1VxGjjt3pA13",
              "6hfptKnd3Gco5oeX4KEecgtrC2KGdZEtgrYCSpfUTH9Q",
              "H3toLGv3Jfm3okGJbXTSLAnCxjHRshBgeQbzR42EDKDr"
            ],
            "recentBlockhash": "936QCxNkveWhaubKMSerSh8YrMkT34tNEABkSEJB7Joo",
            "instructions": [
              {
                "programIdIndex": 14,
                "accounts": [0, 1],
                "data": "3ipZWfsEVvwfKuLHnCxucUGACrmwrFVcBg66chqPxZ4bfwW5CmUxeLdjQFMRKdTTnZiCc8BonyVESy45XyLoTuE4WUKHFdxf2SkbT8q7uvNtBUT8b5kvQ9qf5bmDMYtEobm56aSG14uwP3HwsNu824hWFFmybt9PnHa4DPjLc",
                "stackHeight": null
              },
              {
                "programIdIndex": 15,
                "accounts": [1, 16, 0, 17],
                "data": "2",
                "stackHeight": null
              }
            ],
            "addressTableLookups": []
          }
        },
        {
          "signature": "C8nq9Q732Hb7y9Ha96PaXXAjPLiBgoheCweAms2HVj9tqeUwmFVq8LjSP8dNSzsJNqbLXR9V6pCV8nfnc85YknQ",
          "message": {
            "header": {
              "numRequiredSignatures": 1,
              "numReadonlySignedAccounts": 0,
              "numReadonlyUnsignedAccounts": 1
            },
            "accountKeys": [
              "4aRX4tq2mm5XS2PUUtJPcXUPvgrza5jvjKmoMZzUKcLM",
              "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
              "11111111111111111111111111111111"
            ],
            "recentBlockhash": "936QCxNkveWhaubKMSerSh8YrMkT34tNEABkSEJB7Joo"
          }
        }
      ],
      "next": 10
    }
    ```
    </details>

  - Request for a transaction by its signature

    ```console
    curl "127.0.0.1:3004/transactions?id=G269hkhDQAnK3VNBCz5KVSaP36c5faMDXQuXUDx95PcaEb9cjsL4B7aaK3gqJSHKEvyzH2t9VESJAsQWeryUWNY"
    ```

    <details>
    <summary>
    Truncated sample response
    </summary>

    ```json
    {
      "data": [
        {
          "signature": "G269hkhDQAnK3VNBCz5KVSaP36c5faMDXQuXUDx95PcaEb9cjsL4B7aaK3gqJSHKEvyzH2t9VESJAsQWeryUWNY",
          "message": {
            "header": {
              "numRequiredSignatures": 1,
              "numReadonlySignedAccounts": 0,
              "numReadonlyUnsignedAccounts": 4
            },
            "accountKeys": [
              "66ZC9U8y1uYaAxt4WFYVW11YZeZohvi8ev6wBHsAxykh",
              "GqkoL5E6KemXssCgdRY2wMayPYLhqbaBPfvQmedinuzZ",
              "GDiwGW1o5d4M4TE6PPZM29MakiyXjDgkMPmaqE1RqWb8",
              "F431ucBAkDygeRYVt5eXHapsjKmJ6PbBEz9HabdXnbn1",
              "DKFL2M3TZHz1YrzQQrFmxHHY9YdiPszGZ9n42FFfDvk5",
              "98ACAEUMPE45oVRZF7Ac24LBeFgntg6SLbcNh3SnhaZ6",
              "ComputeBudget111111111111111111111111111111",
              "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
              "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
            ],
            "recentBlockhash": "GyMWAKtDp7hYuNqYMSsPDwWSL172ffMN3TTyPX6JphGF"
          }
        }
      ],
      "next": null
    }
    ```

    </details>

  - Request for all transactions on a particular day

    ```console
    curl "127.0.0.1:3004/transactions?offset=0&count=10&day=12/03/2025" | jq
    ```

    <details>
    <summary>Truncated response</summary>
    ```json
    {
      "data": [
        {
          "signature": "4k1koRtgKowKnrSj3AtrHfaFWjFZUpoHXTZ6mtrFSnDtj8r1UQyDrqfdPokXVV9LAGVCkaUJJzbhmEeoyCmuZzoH",
          "message": {
            "header": {
              "numRequiredSignatures": 1,
              "numReadonlySignedAccounts": 0,
              "numReadonlyUnsignedAccounts": 1
            },
            "accountKeys": [
              "TJxW8fs18KgZp1G4ghMkR5GsxdiKMbgpan4weFThaQ5",
              "8qPCNWqVehF1Sc7YgUKUr7DUZtt514WHF71Wah8ZTkgR",
              "Vote111111111111111111111111111111111111111"
            ],
            "recentBlockhash": "9tPTJskeaSRkrou9JbpPMFWdtVjynNkvsFDmNX4NT2b",
            "instructions": [
              {
                "programIdIndex": 2,
                "accounts": [1, 0],
                "data": "67MGn4zSJG416V2T17qzkfjvLXKLh8YH1PoXoCET5nmsgjsDT9JQXmDLzmfUDs7bjNaUXmipR3f1bQBmiq5VofW5tBXqGnf9DTD9esoWxiCsugLRosEJaqHZTaup8A2nk7cR2SqhKSgbBKapunmknf2TQpfyW9ita5URMXop7vAkujXJXQzctKCuw6UwBfB8EiaNpv8hZD",
                "stackHeight": null
              }
            ]
          }
        }
      ],
      "next": 10
    }
    ```
    </details>
  
  - Request for Account data by public key
    ```console
    curl "127.0.0.1:3004/accounts?pubkey=oQPnhXAbLbMuKHESaGrbXT17CyvWCpLyERSJA9HCYd7" | jq
    ```
    <details>
    <summary>Truncated response</summary>
    ```json
    {
      "data": {
        "lamports": 1141440,
        "data": [
          2, 0, 0, 0, 41, 117, 101, 173, 128, 196, 26, 61, 165, 216, 89, 144, 59,
          132, 58, 175
        ],
        "owner": [
          2, 168, 246, 145, 78, 136, 161, 176, 226, 16, 21, 62, 247, 99, 174, 43, 0,
          194, 185, 61, 22, 193, 36, 210, 192, 83, 122, 16, 4, 128, 0, 0
        ],
        "executable": true,
        "rentEpoch": 18446744073709551615
      }
    }
    ```
    </details>


### Future improvements

- Use `bolckSubscribe` WSS method to subscribe to finalized blocks instead of repeatedly calling `getBlock` via http
- Integrate monitoring to get alerts via sack or email whenever either of the indexer or API has a failure
- Scaling the database by indexing important fields (signature)
- Scaling the db for large amounts of records via sharding
- Scaling access to the db thorugh replication
