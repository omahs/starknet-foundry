use anyhow::{anyhow, Result};
use clap::Args;

use cast::print_formatted;
use cast::{handle_rpc_error, handle_wait_for_tx_result};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, Call, ConnectedAccount, SingleOwnerAccount};
use starknet::core::types::FieldElement;
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;

#[derive(Args)]
#[command(about = "Invoke a contract on Starknet")]
pub struct Invoke {
    /// Address of contract to invoke
    #[clap(short = 'a', long)]
    pub contract_address: FieldElement,

    /// Name of the function to invoke
    #[clap(short, long)]
    pub function: String,

    /// Calldata for the invoked function
    #[clap(short, long, value_delimiter = ' ')]
    pub calldata: Vec<FieldElement>,

    /// Max fee for the transaction. If not provided, max fee will be automatically estimated
    #[clap(short, long)]
    pub max_fee: Option<FieldElement>,
}

#[allow(clippy::unused_async)]
pub fn print_invoke_result(
    invoke_result: Result<FieldElement>,
    int_format: bool,
    json: bool,
) -> Result<()> {
    match invoke_result {
        Ok(transaction_hash) => print_formatted(
            vec![
                ("command", "Invoke".to_string()),
                ("transaction_hash", format!("{transaction_hash}")),
            ],
            int_format,
            json,
            false,
        )?,
        Err(error) => {
            print_formatted(vec![("error", error.to_string())], int_format, json, true)?;
        }
    };
    Ok(())
}

pub async fn invoke(
    contract_address: FieldElement,
    entry_point_name: &str,
    calldata: Vec<FieldElement>,
    max_fee: Option<FieldElement>,
    account: &mut SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<FieldElement> {
    let call = Call {
        to: contract_address,
        selector: get_selector_from_name(entry_point_name)?,
        calldata,
    };
    let execution = account.execute(vec![call]);

    let execution = if let Some(max_fee) = max_fee {
        execution.max_fee(max_fee)
    } else {
        execution
    };

    let result = execution.send().await;

    match result {
        Ok(result) => {
            handle_wait_for_tx_result(
                account.provider(),
                result.transaction_hash,
                result.transaction_hash,
            )
            .await
        }
        Err(Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}
