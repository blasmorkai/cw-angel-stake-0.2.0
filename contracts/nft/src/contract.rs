use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Empty, Coin, Uint128};
use cw2::set_contract_version;
pub use cw721_base::{Cw721Contract, ContractError, InstantiateMsg, MintMsg, MinterResponse};

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw721-angel";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Cw20 {
    pub contract_address: String, 
    pub amount: Uint128,
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub native: Option<Vec<Coin>>,    
    pub cw20: Option<Vec<Cw20>>,
}

impl Metadata {
    pub fn has_native_or_cw20(&self) -> bool {
        self.native.is_some() || self.cw20.is_some()
    }
}

// In the cw721, the MintMsg<Option<Metadata>>, hence this option for the Extension
// ExecuteMsg::Mint requires Option<Metadata>  --> Defined by cw721
// ExecuteMsg::UpdateMetadata requires Metadata  (Not Option)
pub type Extension = Option<Metadata>;

// If we use cw721-base and cw721 version 0.16.0 Cw721Contract will ask for two more parameters. TODO!!!
pub type Cw721MetadataContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension>;
pub type QueryMsg = cw721_base::QueryMsg;

#[cfg(not(feature = "library"))]
pub mod entry {
    use crate::msg::ExecuteMsg;

    use super::*;

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
    use cw721::Cw721Execute;

    #[entry_point]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        let res = Cw721MetadataContract::default().instantiate(deps.branch(), env, info, msg)?;
        // Explicitly set contract name and version, otherwise set to cw721-base info
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
            .map_err(ContractError::Std)?;
        Ok(res)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<Option<Metadata>>,
    ) -> Result<Response, ContractError> {
       // Cw721MetadataContract::default().execute(deps, env, info, msg)
       match msg {
            ExecuteMsg::Mint(mint_msg) => {
                Cw721MetadataContract::default().mint(deps, env, info, mint_msg)
            }
            ExecuteMsg::UpdateMetadata {
                token_id,
                token_uri,
                metadata,
            } => execute_update_metadata(deps, env, info, token_id, token_uri, metadata),
            ExecuteMsg::Burn{token_id} => {
                Cw721MetadataContract::default().burn(deps, env, info, token_id)
            },
            ExecuteMsg::TransferNft { recipient, token_id } => {
                Cw721MetadataContract::default().transfer_nft(deps, env, info, recipient, token_id)
            },
            ExecuteMsg::SendNft { contract, token_id, msg } => {
                Cw721MetadataContract::default().send_nft(deps, env, info, contract, token_id, msg)
            }
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Cw721MetadataContract::default().query(deps, env, msg)
    }

    fn execute_update_metadata(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        token_id: String,
        token_uri: String,
        metadata: Metadata
    ) -> Result<Response, ContractError> {
        let contract = Cw721MetadataContract::default();
        let minter = contract.minter.load(deps.storage)?;
        if info.sender != minter {
            Err(ContractError::Unauthorized {})
        } else {
            contract
                .tokens
                .update(deps.storage, &token_id, |token| match token {
                    Some(mut token_info) => {
                        token_info.extension = Some(metadata);
                        token_info.token_uri = Some(token_uri.clone());
                        Ok(token_info)
                    },
                    None => Err(ContractError::Unauthorized {}),
                })?;
            Ok(Response::new())
        }
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, coins};
    use cw721::Cw721Query;

    const CREATOR: &str = "creator";

    #[test]
    fn use_metadata_extension() {
        let mut deps = mock_dependencies();
        let contract = Cw721MetadataContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: CREATOR.to_string(),
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        let token_id = "Enterprise";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Some(Metadata {
                native: Some(coins(1000, "earth")),
                cw20: None,
            }),
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        contract
            .execute(deps.as_mut(), mock_env(), info, exec_msg)
            .unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
        assert_eq!(res.token_uri, mint_msg.token_uri);
        assert_eq!(res.extension, mint_msg.extension);
    }
}