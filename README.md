# Tendermint Light Client

PoC for a minimalistic TM Light Client based on [lib-rs](https://github.com/cosmos/ibc-rs)

## Cli interface 

### Fetch Consensus State and Header 

This command fetch consensus state and header from full-node. This only use for testing `Verify` command.  

Fetch current consensus state and save output to output_path. URL is a full-node endpoint.
```bash 
tendermint-lightclient fetch-consensus-state <URL> <OUTPUT_PATH>
```
Fetch header at height and save output to output_path. URL is a full-node endpoint.

```bash
tendermint-lightclient fetch-header <URL> <HEIGHT> <OUTPUT_PATH>
```

### Verify

This command verifies a new state (can extract from header) is valid state start from consensus state in cs_path file.

```bash
tendermint-lightclient verify <CS_PATH> <HEADER_PATH>

### Update 

This command work like verify command but create new consensus state and save to new_cs_path. 

```bash
tendermint-lightclient update <CS_PATH> <HEADER_PATH> <NEW_CS_PATH>

### State Proof 

In current version, we only support proof packet transfer on full-node. We only verify the Cosmos IAVL Store.  