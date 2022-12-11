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

pub type Extension = Metadata;      
                                   

// TODO!! If we use cw721-base and cw721 version 0.16.0 Cw721Contract will ask for two more parameters. TODO!!!
pub type Cw721MetadataContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;
pub type Cw721ExecuteMsg = cw721_base::ExecuteMsg<Extension>;
pub type QueryMsg = cw721_base::QueryMsg;
pub type Cw721LiteExecuteMsg = crate::msg::ExecuteMsg<Extension>;

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
        msg: ExecuteMsg<Metadata>,
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
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Cw721MetadataContract::default().query(deps, env, msg)
        // Query that returns metadata?????    <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
        // Cw721QueryMsg::NftInfo { token_id: String }  
                //  #[returns(cw721::NftInfoResponse<Q>)] 
                //    struct NftInfoResponse<T> {token_uri: Option<String>, extension: T,}
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
                        token_info.token_uri = Some(token_uri.clone());
                        token_info.extension = metadata;
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
            name: "Greeks".to_string(),
            symbol: "drachma".to_string(),
            minter: CREATOR.to_string(),
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        let token_id = "1";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "bob".to_string(),
            token_uri: None,
            extension: Metadata {
                native: Some(coins(1000, "earth")),
                cw20: None,
            },
        };
        let exec_msg = Cw721ExecuteMsg::Mint(mint_msg.clone());
        contract
            .execute(deps.as_mut(), mock_env(), info, exec_msg)
            .unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
        assert_eq!(res.token_uri, mint_msg.token_uri);
        assert_eq!(res.extension, mint_msg.extension);
    }

    #[test]
    fn update_metadata() {
        let mut deps = mock_dependencies();
        let contract = Cw721MetadataContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "Greeks".to_string(),
            symbol: "drachma".to_string(),
            minter: CREATOR.to_string(),
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        let token_id = "1";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "bob".to_string(),
            token_uri: None,
            extension: Metadata {
                native: Some(coins(1000, "mars")),
                cw20: None,
            },
        };
        let exec_msg = Cw721ExecuteMsg::Mint(mint_msg.clone());
        contract
            .execute(deps.as_mut(), mock_env(), info, exec_msg)
            .unwrap();

        // Update Metadata
        let new_metadata = Metadata {
            native: Some(coins(2000, "mars")),
            cw20: None,
        };

        // <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<< How can I reference Cw721ExecuteMsg::UpdateMetadata ????
 
        // let exec_msg = 
        //  { token_id: "1".to_string(), token_uri: "".to_string(), metadata: new_metadata };
        // contract
        //     .execute(deps.as_mut(), mock_env(), info, exec_msg)
        //     .unwrap();


    }


}