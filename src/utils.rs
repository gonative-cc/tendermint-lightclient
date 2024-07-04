
#[cfg(test)]
mod tests {
    use ibc_client_tendermint::types::Header;

    #[test]
    fn tests() {
        serde_json::from_str::<Header>(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/data/header.json"
        ))).unwrap();

    }
}