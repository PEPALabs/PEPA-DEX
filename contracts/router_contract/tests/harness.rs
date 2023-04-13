use fuels::{prelude,prelude::*, tx::ContractId, types::Identity,tx::Bytes32,types::Bits256};
use std::str::FromStr;
use rand::{Fill,*};

// abigen macro generates code
abigen!(Contract(
    name = "Factory",
    abi = "../factory_contract/out/debug/factory_contract-abi.json"
),Contract(
    name = "Swap",
    abi = "../swap_contract/out/debug/swap_contract-abi.json"
),Contract(
    name = "Router",
    abi = "./out/debug/router_contract-abi.json"
)
);

// Helper function to create a custom wallet with custom assets
#[allow(unused_variables)]
async fn get_wallet_instance(asset_id_1:AssetId, asset_id_2:AssetId) -> WalletUnlocked {
    let mut rng = rand::thread_rng();
    let mut wallet = WalletUnlocked::new_random(None);

    // Add native assets and two custom assets
    let asset_base = AssetConfig {
        id: BASE_ASSET_ID,
        num_coins: 2,
        coin_amount: 10000000,
    };

    let asset_1 = AssetConfig {
        id: asset_id_1,
        num_coins: 6,
        coin_amount: 10000000,
    };
    let asset_2 = AssetConfig {
        id: asset_id_2,
        num_coins: 10,
        coin_amount: 10000000,
    };

    let assets = vec![asset_base, asset_1, asset_2];

    // custom wallet    
    let coins = setup_custom_assets_coins(wallet.address(), &assets);
    let (provider, _socket_addr) = setup_test_provider(coins, vec![], None, None).await;
    wallet.set_provider(provider);
    wallet 
}

// Helper function to create a Swap contract instance and setup two custom assets
#[allow(unused_variables)]
async fn get_contract_instance() -> (Swap, Bech32ContractId,  AssetId,AssetId,WalletUnlocked) {

    let mut rng = rand::thread_rng();
    let mut asset_id_1 = AssetId::zeroed();
    asset_id_1.try_fill(&mut rng);

    let mut asset_id_2 = AssetId::zeroed();
    asset_id_2.try_fill(&mut rng);

    let wallet = get_wallet_instance(asset_id_1,asset_id_2).await;


    let id = Contract::deploy(
        "../swap_contract/out/debug/swap_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "../swap_contract/out/debug/swap_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = Swap::new(id.clone(), wallet.clone());

    (instance, id, asset_id_1, asset_id_2,wallet.clone())
}

// Helper function to create a factory contract instance
#[allow(unused_variables)]
async fn get_factory_contract_instance(asset_id_1:AssetId, asest_id_2:AssetId, wallet:WalletUnlocked) -> (Factory,Bech32ContractId) {
    let id = Contract::deploy(
        "../factory_contract/out/debug/factory_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "../factory_contract/out/debug/factory_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = Factory::new(id.clone(), wallet.clone());
    (instance,id)
}

// Helper function to create another swap contract instance and setup two custom assets
#[allow(unused_variables)]
async fn get_swap_contract_instance(asset_id_1:AssetId, asest_id_2:AssetId, wallet:WalletUnlocked) -> (Swap,Bech32ContractId) {
    let id = Contract::deploy(
        "../swap_contract/out/debug/swap_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "../swap_contract/out/debug/swap_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = Swap::new(id.clone(), wallet.clone());
    (instance,id)
}

// Helper function to create a swap contract instance and setup two custom assets
#[allow(unused_variables)]
async fn get_router_contract_instance(asset_id_1:AssetId, asest_id_2:AssetId, wallet:WalletUnlocked) -> (Router,Bech32ContractId) {
    let id = Contract::deploy(
        "./out/debug/router_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/router_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = Router::new(id.clone(), wallet.clone());
    (instance,id)
}


//helper function to create contracts and wallets
#[allow(unused_variables)]
async fn create_everything() -> (AssetId, AssetId, Swap, Factory, Router, WalletUnlocked) {
    let (_instance, _id,  asset_id_1, asset_id_2,wallet) = get_contract_instance().await;
    let (factory_instance, factory_id) = get_factory_contract_instance(asset_id_1, asset_id_2, wallet.clone()).await;
    let (router_instance, router_id) = get_router_contract_instance(asset_id_1, asset_id_2, wallet.clone()).await;
    let result0 = _instance.methods().initialize(ContractId::from(*asset_id_1),ContractId::from(*asset_id_2)).call().await.unwrap();
    let result1 = factory_instance.methods().initialize(Bits256(*(_id.hash()))).call().await.unwrap();
    let result2 = router_instance.methods().initialize(Bits256(*(factory_id.hash()))).call().await.unwrap();
    let result3 = factory_instance.methods()
        .create_swap(Bits256(*(_instance.contract_id().hash())))
        .set_contracts(&[&router_instance,&_instance, &factory_instance])
        .call().await.unwrap();
    (asset_id_1, asset_id_2, _instance, factory_instance, router_instance, wallet.clone())
}

// h
#[allow(unused_variables)]
async fn intialize_pool(asset_id_1:AssetId, asset_id_2:AssetId, wallet: WalletUnlocked, swap_instance: Swap)  -> Result<()> {
    let amount0 = 100;
    let amount1 = 100;
    let result_tx2 = wallet
            .force_transfer_to_contract(&swap_instance.contract_id(), amount0, asset_id_1, TxParameters::default())
            .await?;
    let result_tx3 = wallet
            .force_transfer_to_contract(&swap_instance.contract_id(), amount1, asset_id_2, TxParameters::default())
            .await?;

    // TODO: test min amount functionality
    let result = swap_instance.methods()
    .add_liquidity()
    .set_contracts(&[&swap_instance])
    .append_variable_outputs(1).call().await?;
    Ok(())
}

// test initialize
#[tokio::test]
async fn can_initialize()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance, router_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function
    Ok(())
}

// test add liquidity
#[tokio::test]
async fn can_add_liquidity()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance, router_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function
    let amount0 = 100;
    let amount1 = 100;
    let result_tx2 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount0, asset_id_1, TxParameters::default())
            .await?;
    let result_tx3 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount1, asset_id_2, TxParameters::default())
            .await?;

    // TODO: test min amount functionality
    let result = router_instance.methods()
    .add_liquidity(Bits256(*(swap_instance.contract_id().hash())), amount0, amount1, amount0, amount1)
    .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
    .append_variable_outputs(1).call().await?;

    // check output lp token
    let balances= wallet.get_balances().await?;
    let lp_token = format!("{:#x}",ContractId::from(*(swap_instance.contract_id().hash())));
    assert_eq!(balances.get(&lp_token).unwrap(),&94);
    Ok(())
}

#[tokio::test]
async fn can_remove_liquidity()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance, router_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function
    let amount0 = 100;
    let amount1 = 100;
    let result_tx2 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount0, asset_id_1, TxParameters::default())
            .await?;
    let result_tx3 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount1, asset_id_2, TxParameters::default())
            .await?;

    // TODO: test min amount functionality
    let result = router_instance.methods()
    .add_liquidity(Bits256(*(swap_instance.contract_id().hash())), amount0, amount1, amount0, amount1)
    .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
    .append_variable_outputs(1).call().await?;

    // check output lp token
    let balances= wallet.get_balances().await?;
    let lp_token = format!("{:#x}",ContractId::from(*(swap_instance.contract_id().hash())));
    let token1 = format!("{:#x}",ContractId::from(*(asset_id_1)));
    let token2 = format!("{:#x}",ContractId::from(*(asset_id_2)));
    assert_eq!(balances.get(&lp_token).unwrap(),&94);

    // remove liquidity
    let amount_lp = *balances.get(&lp_token).unwrap() - 1;
    assert_eq!(amount_lp, 93);
    let amount_1 = *balances.get(&token1).unwrap();
    let amount_2 = *balances.get(&token2).unwrap();
    let result_tx4 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount_lp, AssetId::from(*(swap_instance.contract_id().hash())),TxParameters::default())
            .await?;
    let result = router_instance.methods()
        .remove_liquidity(Bits256(*(swap_instance.contract_id().hash())), amount_lp,amount0, amount1, amount0, amount1)
        .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
        .append_variable_outputs(2).call().await?;

    let new_balances= wallet.get_balances().await?;
    let token1 = format!("{:#x}",ContractId::from(*(asset_id_1)));
    let token2 = format!("{:#x}",ContractId::from(*(asset_id_2)));
    assert_eq!(new_balances.get(&lp_token).unwrap(),&1);
    assert_eq!(new_balances.get(&token1).unwrap(),&(amount_1 + 98));
    assert_eq!(new_balances.get(&token2).unwrap(),&(amount_2 + 98));

    Ok(())
}

#[tokio::test]
async fn can_swap_exact_input_for_output()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance, router_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function
    let amount0 = 1000;
    let amount1 = 1000;
    let result_tx2 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount0, asset_id_1, TxParameters::default())
            .await?;
    let result_tx3 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount1, asset_id_2, TxParameters::default())
            .await?;

    // TODO: test min amount functionality
    let result = router_instance.methods()
        .add_liquidity(Bits256(*(swap_instance.contract_id().hash())), amount0, amount1, amount0, amount1)
        .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
        .append_variable_outputs(1).call().await?;

    // check output lp token
    let balances= wallet.get_balances().await?;
    let lp_token = format!("{:#x}",ContractId::from(*(swap_instance.contract_id().hash())));
    assert_eq!(balances.get(&lp_token).unwrap(),&999);

    // Swap
    let swap_amount = 100;
    let min_out = 90;
    let result_send = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), swap_amount, asset_id_1, TxParameters::default())
            .await?;

    // check token before swap
    let balances= wallet.get_balances().await?;
    let token = format!("{:#x}",ContractId::from(*(asset_id_2)));
    let token_amount = balances.get(&token).unwrap();

    let result_swap = router_instance.methods()
        .swap_exact_input_for_output(Bits256(*(swap_instance.contract_id().hash())), ContractId::from(*(asset_id_1)), ContractId::from(*(asset_id_2)) , swap_amount, 0, min_out, Identity::ContractId(ContractId::from(*(wallet.address().hash()))))
        .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
        .append_variable_outputs(1).call().await?;

    let new_balances= wallet.get_balances().await?;
    let new_token_amount = new_balances.get(&token).unwrap();
    assert_eq!(*new_token_amount, token_amount + 90);
    Ok(())

}

#[tokio::test]
async fn can_swap_exact_input_for_output_multihop()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance, router_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function
    let amount0 = 1000;
    let amount1 = 1000;
    let result_tx2 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount0, asset_id_1, TxParameters::default())
            .await?;
    let result_tx3 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount1, asset_id_2, TxParameters::default())
            .await?;

    // TODO: test min amount functionality
    let result = router_instance.methods()
        .add_liquidity(Bits256(*(swap_instance.contract_id().hash())), amount0, amount1, amount0, amount1)
        .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
        .append_variable_outputs(1).call().await?;

    // check output lp token
    let balances= wallet.get_balances().await?;
    let lp_token = format!("{:#x}",ContractId::from(*(swap_instance.contract_id().hash())));
    assert_eq!(balances.get(&lp_token).unwrap(),&999);

    
    // check token before swap
    let balances= wallet.get_balances().await?;
    let token = format!("{:#x}",ContractId::from(*(asset_id_1)));
    let token_amount = balances.get(&token).unwrap();
    let min_token_out = 0;
    let path = vec![Bits256(*(asset_id_1)), Bits256(*(asset_id_2)), Bits256(*(asset_id_1))];
    // Swap
    let swap_amount = 100;
    let result_send = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), swap_amount, asset_id_1, TxParameters::default())
            .await?;
    
    let result_swap = router_instance.methods()
        .swap_exact_input_for_output_multihop(Bits256(*(factory_instance.contract_id().hash())),path, swap_amount.clone(), min_token_out, Identity::ContractId(ContractId::from(*(wallet.address().hash()))))
        .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
        .append_variable_outputs(2).call().await?;
        
    let new_balances= wallet.get_balances().await?;
    let new_token_amount = new_balances.get(&token).unwrap();
    assert_eq!(*new_token_amount, token_amount-1);
    Ok(())
}



#[tokio::test]
async fn can_swap_input_for_exact_output()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance, router_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function
    let amount0 = 1000;
    let amount1 = 1000;
    let result_tx2 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount0, asset_id_1, TxParameters::default())
            .await?;
    let result_tx3 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount1, asset_id_2, TxParameters::default())
            .await?;

    // TODO: test min amount functionality
    let result = router_instance.methods()
        .add_liquidity(Bits256(*(swap_instance.contract_id().hash())), amount0, amount1, amount0, amount1)
        .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
        .append_variable_outputs(1).call().await?;

    // check output lp token
    let balances= wallet.get_balances().await?;
    let lp_token = format!("{:#x}",ContractId::from(*(swap_instance.contract_id().hash())));
    assert_eq!(balances.get(&lp_token).unwrap(),&999);

    // Swap
    let swap_amount = 100;
    let result_send = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), swap_amount, asset_id_1, TxParameters::default())
            .await?;

    // check token before swap
    let balances= wallet.get_balances().await?;

    let token0 = format!("{:#x}",ContractId::from(*(asset_id_1)));
    let token1 = format!("{:#x}",ContractId::from(*(asset_id_2)));
    let token_amount = balances.get(&token1).unwrap();
    // interface: swap_address, asset0, asset1,asset_0_amount:u64, asset_1_amount:u64 ,  amount_in_max:u64, amount_out:u64, to:Identity
    let amount_out = 55;
    let result_swap = router_instance.methods()
        .swap_input_for_exact_output(Bits256(*(swap_instance.contract_id().hash())), ContractId::from(*(asset_id_1)), ContractId::from(*(asset_id_2)), swap_amount.clone(), 0, swap_amount.clone(),amount_out, Identity::ContractId(ContractId::from(*(wallet.address().hash()))))
        .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
        .append_variable_outputs(2).call().await?;

    let new_balances= wallet.get_balances().await?;
    let new_token_amount = new_balances.get(&token1).unwrap();
    assert_eq!(*new_token_amount, token_amount + amount_out.clone());
    Ok(())

}

#[tokio::test]
async fn can_swap_input_for_exact_output_multihop()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance, router_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function
    let amount0 = 1000;
    let amount1 = 1000;
    let result_tx2 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount0, asset_id_1, TxParameters::default())
            .await?;
    let result_tx3 = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), amount1, asset_id_2, TxParameters::default())
            .await?;

    // TODO: test min amount functionality
    let result = router_instance.methods()
        .add_liquidity(Bits256(*(swap_instance.contract_id().hash())), amount0, amount1, amount0, amount1)
        .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
        .append_variable_outputs(1).call().await?;

    // check output lp token
    let balances= wallet.get_balances().await?;
    let lp_token = format!("{:#x}",ContractId::from(*(swap_instance.contract_id().hash())));
    assert_eq!(balances.get(&lp_token).unwrap(),&999);

    // Swap
    let swap_amount = 100;
    let result_send = wallet
            .force_transfer_to_contract(&router_instance.contract_id(), swap_amount, asset_id_1, TxParameters::default())
            .await?;

    // check token before swap
    let balances= wallet.get_balances().await?;
    let token = format!("{:#x}",ContractId::from(*(asset_id_1)));
    let token_amount = balances.get(&token).unwrap();
    let amount_out = 45;
    let path = vec![Bits256(*(asset_id_1)), Bits256(*(asset_id_2)), Bits256(*(asset_id_1))];
    // interface: swap_factory: b256, path:Vec<b256>, amount_out:u64 , amount_in_max:u64, to:Identity
    let result_swap = router_instance.methods()
        .swap_input_for_exact_output_multihop(Bits256(*(factory_instance.contract_id().hash())), path, amount_out.clone(), swap_amount, Identity::ContractId(ContractId::from(*(wallet.address().hash()))))
        .set_contracts(&[&swap_instance, &factory_instance, &router_instance])
        .append_variable_outputs(2).call().await?;

    // TODO: add a third token
    let new_balances= wallet.get_balances().await?;
    let new_token_amount = new_balances.get(&token).unwrap();
    assert_eq!(*new_token_amount, token_amount+amount_out.clone());
    Ok(())
}

