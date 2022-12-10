import { SigningCosmWasmClient, Secp256k1HdWallet, GasPrice } from "cosmwasm";

import * as fs from 'fs';

const cw_core_wasm = fs.readFileSync("../artifacts/cw_core.wasm");
const cw721_stake_wasm = fs.readFileSync("../artifacts/cw721_stake.wasm");
const cw_proposal_single_wasm = fs.readFileSync("../artifacts/cw_proposal_single.wasm");

const cw_core_id = 4274;
const cw721_stake_id = 4269;
const cw_proposal_single_id = 4270;
const cw721_id = 3582;

const rpcEndpoint = "https://rpc.uni.juno.deuslabs.fi";

const mnemonic =
    "test peanut elevator motor proud globe obtain gasp sad balance nature ladder";

const prefix = "juno";

const nft_contract_address = "juno1q4hgxfrl4wzyyynpn92ytn39fttwx7yyhmm09lecrctucr724s6q3s7spc";
const cw_core_contract_address = "juno1p5jq04hf6tvgl98k2pew92jw4pw8zpvl5ah9avafjsrf8cwang4qmcjpgt";
const cw721_staking_address = "juno1lsxl2hs2tcvkmlaz7a4ctuagk9rjxdqjz3t9y3sxp4zgf4qlzzns5p60dd";
const cw_single_proposal_address = "juno15j80fyf2nutq600kuupvjy3rpa6czxelxwzl8c4a2r833yyz8xuqk2kmp3";

async function setupClient(mnemonic: string): Promise<SigningCosmWasmClient> {
    let gas = GasPrice.fromString("0.025ujunox");
    let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: prefix });
    let client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet, { gasPrice: gas });
    return client;
}

async function getAddress(mnemonic: string) {
    let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: prefix });
    let accounts = await wallet.getAccounts();
    return accounts[0].address;
}

describe("DAODAO NFT Staking Tests", () => {

    xit("Upload contract and get code_id", async() => {
        let client = await setupClient(mnemonic);
        let res = await client.upload(await getAddress(mnemonic), cw_core_wasm, "auto", undefined);
        console.log(res);
    }).timeout(10000000000);

    xit("Instantiate NFT Contract", async () => {
        let client = await setupClient(mnemonic);
        let res = await client.instantiate(await getAddress(mnemonic), cw721_id, { name: "NFT", symbol: "DAODAO", minter:await getAddress(mnemonic) }, "DAO NFT", "auto", undefined);
        console.log(res);
    }).timeout(100000);

    xit("Mint NFT", async () => {
        let client = await setupClient(mnemonic);
        let res = await client.execute(await getAddress(mnemonic), nft_contract_address, {mint:{token_id:"1", token_uri:"url", owner:await getAddress(mnemonic)}}, "auto", undefined);
        console.log(res);
    }).timeout(100000);

    xit("Instantiate Core Contract", async () => {
        let client = await setupClient(mnemonic);
        let proposal_instantiate_msg = {
            threshold: {absolute_percentage:{percentage: {majority:{}}}},
            max_voting_period:{height:1000},
            only_members_execute:false,
            allow_revoting:false,
            close_proposal_on_execution_failure:true
        };

        let cw721_stake_instantiate_msg = {
            owner: {instantiator:{}},
            nft_address: nft_contract_address,
        };

        let cw_core_instantiate_msg = {
            name: "DAO DAO",
            description: "A DAO that builds DAOs",
            automatically_add_cw20s:true,
            automatically_add_cw721s:false,
            voting_module_instantiate_info: {
                code_id:cw721_stake_id,
                msg:Buffer.from(JSON.stringify(cw721_stake_instantiate_msg)).toString("base64"),
                admin:{none:{}},
                label: "DAO DAO voting module",
            },
            proposal_modules_instantiate_info: [{
                code_id:cw_proposal_single_id,
                msg:Buffer.from(JSON.stringify(proposal_instantiate_msg)).toString("base64"),
                admin:{core_contract:{}},
                label: "DAO DAO governance module."
            }]
        };

        //let res = await client.instantiate(await getAddress(mnemonic), cw_proposal_single_id, proposal_instantiate_msg, "DAO DAO", "auto", undefined);
        //let res = await client.instantiate(await getAddress(mnemonic), cw721_stake_id, cw721_stake_instantiate_msg, "DAO DAO", "auto", undefined);
        let res = await client.instantiate(await getAddress(mnemonic), cw_core_id, cw_core_instantiate_msg, "DAO DAO 2", "auto", undefined);
        console.log(res);
    }).timeout(100000000);

    xit("Get Staking contract address", async () => {
        let client = await setupClient(mnemonic);
        let res = await client.queryContractSmart(cw_core_contract_address, {dump_state:{}});
        console.log(res);
    }).timeout(1000000);

    xit("Stake NFTs", async () => {
        let client = await setupClient(mnemonic);
        let res = await client.execute(await getAddress(mnemonic), nft_contract_address, {send_nft:{contract:cw721_staking_address, token_id:"1", msg:Buffer.from(JSON.stringify({})).toString("base64")}}, "auto", undefined);
        console.log(res);
    }).timeout(100000);

    xit("Create Proposal", async () => {
        let client = await setupClient(mnemonic);
        let res = await client.execute(await getAddress(mnemonic), cw_single_proposal_address, {propose:{title: "A simple text proposal", description: "This is a simple text proposal", msgs:[]}}, "auto", undefined);
        console.log(res);
    }).timeout(100000);

    xit("List proposals", async() => {
        let client = await setupClient(mnemonic);
        let res = await client.queryContractSmart(cw_single_proposal_address, {list_proposals:{}});
        console.log(res);
    }).timeout(100000);

    xit("Get Proposal id1", async() => {
        let client = await setupClient(mnemonic);
        let res = await client.queryContractSmart(cw_single_proposal_address, {proposal:{proposal_id:1}});
        console.log(res);
    }).timeout(100000);

    xit("Vote on Proposal", async () => {
        let client = await setupClient(mnemonic);
        let res = await client.execute(await getAddress(mnemonic), cw_single_proposal_address, {vote:{proposal_id:1, vote: "yes"}}, "auto", undefined);
        console.log(res);
    }).timeout(100000);
    


});