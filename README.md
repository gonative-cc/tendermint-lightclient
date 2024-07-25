# Tendermint Light Client

PoC for a minimalistic TM Light Client based on [lib-rs](https://github.com/cosmos/ibc-rs)

## Cli interface 

### Fetch Consensus State and Header 
This command fetch consensus state and header from full-node. This only use for testing `Verify` command.  

### Verify 

```bash
tendermint-lightclient verify <CS_PATH> <HEADER_PATH> <NEW_CS_PATH>
```

Verify a new state (can extract from header) is valid state from consensus state in cs_path file. This also create new consesus state and save to new_cs_path. 

### State Proof 

In current version, we only support proof packet transfer on full-node. We only verify the Cosmos IAVL Store.  