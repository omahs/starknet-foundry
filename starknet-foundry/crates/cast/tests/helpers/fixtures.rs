use crate::helpers::constants::{ACCOUNT, ACCOUNT_FILE_PATH, CONTRACTS_DIR, NETWORK, URL};
use camino::Utf8PathBuf;
use cast::{get_account, get_provider, parse_number};
use serde_json::{json, Value};
use starknet::accounts::{Account, Call};
use starknet::contract::ContractFactory;
use starknet::core::types::contract::{CompiledClass, SierraClass};
use starknet::core::types::FieldElement;
use starknet::core::types::TransactionReceipt;
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use url::Url;

pub async fn declare_deploy_contract(path: &str) {
    let provider = get_provider(URL, NETWORK)
        .await
        .expect("Could not get the provider");
    let account = get_account(
        ACCOUNT,
        &Utf8PathBuf::from(ACCOUNT_FILE_PATH),
        &provider,
        NETWORK,
    )
    .expect("Could not get the account");

    let contract_definition: SierraClass = {
        let file_contents = std::fs::read(CONTRACTS_DIR.to_string() + path + ".sierra.json")
            .expect("Could not read contract's sierra file");
        serde_json::from_slice(&file_contents).expect("Could not cast sierra file to SierraClass")
    };
    let casm_contract_definition: CompiledClass = {
        let file_contents = std::fs::read(CONTRACTS_DIR.to_string() + path + ".casm.json")
            .expect("Could not read contract's casm file");
        serde_json::from_slice(&file_contents).expect("Could not cast casm file to CompiledClass")
    };

    let casm_class_hash = casm_contract_definition
        .class_hash()
        .expect("Could not compute class_hash");

    let declaration = account.declare(
        Arc::new(
            contract_definition
                .flatten()
                .expect("Could not flatten SierraClass"),
        ),
        casm_class_hash,
    );
    let declared = declaration.send().await.unwrap();

    let factory = ContractFactory::new(declared.class_hash, account);
    let deployment = factory.deploy(Vec::new(), FieldElement::ONE, false);
    deployment.send().await.unwrap();
}

pub async fn invoke_map_contract(key: &str, value: &str, account: &str, contract_address: &str) {
    let provider = get_provider(URL, NETWORK)
        .await
        .expect("Could not get the provider");
    let account = get_account(
        account,
        &Utf8PathBuf::from(ACCOUNT_FILE_PATH),
        &provider,
        NETWORK,
    )
    .expect("Could not get the account");

    let call = Call {
        to: parse_number(contract_address).expect("Could not parse the contract address"),
        selector: get_selector_from_name("put").expect("Could not get selector from put"),
        calldata: vec![
            parse_number(key).expect("Could not parse the key"),
            parse_number(value).expect("Could not parse the value"),
        ],
    };
    let execution = account.execute(vec![call]);

    execution.send().await.unwrap();
}

#[must_use]
pub fn default_cli_args() -> Vec<&'static str> {
    vec![
        "--url",
        URL,
        "--network",
        NETWORK,
        "--accounts-file",
        ACCOUNT_FILE_PATH,
    ]
}

#[must_use]
pub fn get_transaction_hash(output: &[u8]) -> FieldElement {
    let output: HashMap<String, String> =
        serde_json::from_slice(output).expect("Could not serialize transaction output to HashMap");
    parse_number(
        output
            .get("transaction_hash")
            .expect("Could not get transaction_hash from output"),
    )
    .expect("Could not parse a number")
}

pub async fn get_transaction_receipt(tx_hash: FieldElement) -> TransactionReceipt {
    let client = reqwest::Client::new();
    let json = json!(
        {
            "jsonrpc": "2.0",
            "method": "starknet_getTransactionReceipt",
            "params": {
                "transaction_hash": format!("{tx_hash:#x}"),
            },
            "id": 0,
        }
    );
    let resp: Value = serde_json::from_str(
        &client
            .post(URL)
            .header("Content-Type", "application/json")
            .body(json.to_string())
            .send()
            .await
            .expect("Error occurred while getting transaction receipt")
            .text()
            .await
            .expect("Could not get response from getTransactionReceipt"),
    )
    .expect("Could not serialize getTransactionReceipt response");

    let result = resp
        .get("result")
        .expect("There is no `result` field in getTransactionReceipt response");
    serde_json::from_str(&result.to_string())
        .expect("Could not serialize result to `TransactionReceipt`")
}

#[must_use]
pub fn create_test_provider() -> JsonRpcClient<HttpTransport> {
    let parsed_url = Url::parse(URL).unwrap();
    JsonRpcClient::new(HttpTransport::new(parsed_url))
}

#[must_use]
pub fn duplicate_directory_with_salt(src_path: String, to_be_salted: &str, salt: &str) -> String {
    let dest_path = src_path.clone() + salt;

    let src_dir = Utf8PathBuf::from(src_path);
    let dest_dir = Utf8PathBuf::from(&dest_path);

    fs::create_dir_all(&dest_dir).expect("Unable to create directory");

    fs_extra::dir::copy(
        src_dir.join("src"),
        &dest_dir,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy the src directory");
    fs_extra::file::copy(
        src_dir.join("Scarb.toml"),
        dest_dir.join("Scarb.toml"),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy Scarb.toml");

    let contract_code =
        fs::read_to_string(src_dir.join("src/lib.cairo")).expect("Unable to get contract code");
    let updated_code = contract_code.replace(to_be_salted, &(to_be_salted.to_string() + salt));
    fs::write(dest_dir.join("src/lib.cairo"), updated_code)
        .expect("Unable to change contract code");

    dest_path
}
