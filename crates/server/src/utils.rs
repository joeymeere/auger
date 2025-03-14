use solana_client::rpc_client::RpcClient;
use solana_sdk::bpf_loader_upgradeable::UpgradeableLoaderState;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{bpf_loader, bpf_loader_deprecated, bpf_loader_upgradeable};

pub fn process_dump(
    rpc_client: &RpcClient,
    account_pubkey: Option<Pubkey>,
) -> Result<Vec<u8>, String> {
    if let Some(account_pubkey) = account_pubkey {
        if let Some(account) = rpc_client
            .get_account_with_commitment(&account_pubkey, CommitmentConfig::confirmed())
            .expect("Failed to get account")
            .value
        {
            if account.owner == bpf_loader::id() || account.owner == bpf_loader_deprecated::id() {
                Ok(account.data.to_vec())
            } else if account.owner == bpf_loader_upgradeable::id() {
                if let Ok(UpgradeableLoaderState::Program {
                    programdata_address,
                }) = account.deserialize_data()
                {
                    if let Some(programdata_account) = rpc_client
                        .get_account_with_commitment(
                            &programdata_address,
                            CommitmentConfig::confirmed(),
                        )
                        .expect("Failed to get programdata account")
                        .value
                    {
                        if let Ok(UpgradeableLoaderState::ProgramData { .. }) =
                            programdata_account.deserialize_data()
                        {
                            let offset = UpgradeableLoaderState::size_of_programdata_metadata();
                            let program_data = &programdata_account.data[offset..];
                            Ok(program_data.to_vec())
                        } else {
                            Err(format!("Program {account_pubkey} has been closed").into())
                        }
                    } else {
                        Err(format!("Program {account_pubkey} has been closed").into())
                    }
                } else if let Ok(UpgradeableLoaderState::Buffer { .. }) = account.deserialize_data()
                {
                    let offset = UpgradeableLoaderState::size_of_buffer_metadata();
                    let program_data = &account.data[offset..];
                    Ok(program_data.to_vec())
                } else {
                    Err(format!(
                        "{account_pubkey} is not an upgradeable loader buffer or program account"
                    )
                    .into())
                }
            } else {
                Err(format!("{account_pubkey} is not an SBF program").into())
            }
        } else {
            Err(format!("Unable to find the account {account_pubkey}").into())
        }
    } else {
        Err("No account specified".into())
    }
}
