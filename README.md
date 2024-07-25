# Tendermint Light Client

PoC for a minimalistic TM Light Client based on [lib-rs](https://github.com/cosmos/ibc-rs)

## Cli interface 

### Fetch Consensus State and Header 
This command fetch consensus state and header from full-node. This only use for testing `Verify` command.  

### Verify

```bash
tendermint-lightclient verify <CS_PATH> <HEADER_PATH>
```

The Consensus State format follow [ics-007-tendermint-client](https://github.com/cosmos/ibc/tree/main/spec/client/ics-007-tendermint-client#consensus-state) and `root` is `app_hash`. Other data can easy to get from block header.


```rust
pub struct ConsensusState {
    pub timestamp: Time,
    pub root: CommitmentRoot,
    pub next_validators_hash: Hash,
}
```

This command verifies a new state (can extract from header) is valid state start from consensus state in cs_path file.

### Update 
```bash
tendermint-lightclient update <CS_PATH> <HEADER_PATH> <NEW_CS_PATH>
```
This command work like verify command but create new consensus state and save to new_cs_path. 

### State Proof 

In current version, we only support proof packet transfer on full-node. We only verify the Cosmos IAVL Store.  