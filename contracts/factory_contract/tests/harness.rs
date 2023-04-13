use fuels::{prelude,prelude::*, tx::ContractId,tx::Bytes32,types::Bits256};
use std::str::FromStr;
use rand::{Fill,*};

// abigen macro generates code
abigen!(Contract(
    name = "Factory",
    abi = "./out/debug/factory_contract-abi.json"
),Contract(
    name = "Swap",
    abi = "../swap_contract/out/debug/swap_contract-abi.json"
),
Contract(
    name = "Token",
    abi = "../token_contract/out/debug/token_contract-abi.json"
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
        coin_amount: 10000,
    };

    let asset_1 = AssetConfig {
        id: asset_id_1,
        num_coins: 6,
        coin_amount: 10000,
    };
    let asset_2 = AssetConfig {
        id: asset_id_2,
        num_coins: 10,
        coin_amount: 10000,
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

// Helper function to create a swap contract instance and setup two custom assets
#[allow(unused_variables)]
async fn get_factory_contract_instance(asset_id_1:AssetId, asest_id_2:AssetId, wallet:WalletUnlocked) -> (Factory,Bech32ContractId) {
    let id = Contract::deploy(
        "./out/debug/factory_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/factory_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = Factory::new(id.clone(), wallet.clone());
    (instance,id)
}

// Helper function to create another swap contract instance and setup two custom assets
#[allow(unused_variables)]
async fn get_swap_contract_instance(asset_id_1:AssetId, asest_id_2:AssetId, wallet:WalletUnlocked) -> (Factory,Bech32ContractId) {
    let id = Contract::deploy(
        "./out/debug/factory_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/factory_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = Factory::new(id.clone(), wallet.clone());
    (instance,id)
}


//helper function to create contracts and wallets
#[allow(unused_variables)]
async fn create_everything() -> (AssetId, AssetId, Swap, Factory, WalletUnlocked) {
    let (_instance, _id,  asset_id_1, asset_id_2,wallet) = get_contract_instance().await;
    let (factory_instance, factory_idt) = get_factory_contract_instance(asset_id_1, asset_id_2, wallet.clone()).await;
    let result0 = _instance.methods().initialize(ContractId::from(*asset_id_1),ContractId::from(*asset_id_2)).call().await.unwrap();
    let result1 = factory_instance.methods().initialize(Bits256(*(_id.hash()))).call().await.unwrap();
    (asset_id_1, asset_id_2, _instance, factory_instance, wallet.clone())
}

// test initialize
#[tokio::test]
async fn can_initialize()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function
    Ok(())
}

//
// test initialize
#[tokio::test]
async fn can_create_swap()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function

    // Note: Calling external contracts requires specify external contract in call
    let result = factory_instance.methods()
        .exist_swap(Bits256(*(swap_instance.contract_id().hash())))
        .set_contracts(&[&swap_instance, &factory_instance])
        .call().await?;
    assert_eq!(result.value, false);
    // create swap
    let result0 = factory_instance.methods()
        .create_swap(Bits256(*(swap_instance.contract_id().hash())))
        .set_contracts(&[&swap_instance, &factory_instance])
        .call().await?;
    let result1 = factory_instance.methods()
        .exist_swap(Bits256(*(swap_instance.contract_id().hash())))
        .set_contracts(&[&swap_instance, &factory_instance])
        .call().await?;
    assert_eq!(result1.value, true);
    Ok(())
}


#[tokio::test]
async fn can_get_swap()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function

    // Note: Calling external contracts requires specify external contract in call
    // create swap
    let result0 = factory_instance.methods()
        .create_swap(Bits256(*(swap_instance.contract_id().hash())))
        .set_contracts(&[&swap_instance, &factory_instance])
        .call().await?;
    let result1 = factory_instance.methods()
        .exist_swap(Bits256(*(swap_instance.contract_id().hash())))
        .set_contracts(&[&swap_instance, &factory_instance])
        .call().await?;
    assert_eq!(result1.value, true);

    // get swap
    let swap_address = factory_instance.methods()
        .get_swap(Bits256(*(asset_id_1)), Bits256(*(asset_id_2)))
        .set_contracts(&[&swap_instance, &factory_instance])
        .call().await?;
    assert!(swap_address.value.is_some());
    assert_eq!(swap_address.value.unwrap(), Bits256(*swap_instance.contract_id().hash()));

    Ok(())
}

#[tokio::test]
async fn can_get_swap_not_exist()-> Result<()> {
    let (asset_id_1, asset_id_2, swap_instance,factory_instance ,wallet) = create_everything().await;
    // Now you have an instance of your contract you can use to test each function

    // Note: Calling external contracts requires specify external contract in call
    // create swap
    let result0 = factory_instance.methods()
        .create_swap(Bits256(*(swap_instance.contract_id().hash())))
        .set_contracts(&[&swap_instance, &factory_instance])
        .call().await?;
    let result1 = factory_instance.methods()
        .exist_swap(Bits256(*(swap_instance.contract_id().hash())))
        .set_contracts(&[&swap_instance, &factory_instance])
        .call().await?;
    assert_eq!(result1.value, true);

    // get swap
    let swap_address = factory_instance.methods()
        .get_swap(Bits256(*(BASE_ASSET_ID)), Bits256(*(asset_id_2)))
        .set_contracts(&[&swap_instance, &factory_instance])
        .call().await?;
    assert!(swap_address.value.is_none());

    Ok(())
}

// // test deposit
// #[tokio::test]
// async fn can_deposit() -> Result<()>{
//     let (_instance, _id,  asset_id_1, asset_id_2,wallet) = get_contract_instance().await;
//     let my_tx_params = TxParameters::new(None, Some(1_000_000), None);
//     let result0 = _instance.methods().initialize(ContractId::from(*asset_id_1),ContractId::from(*asset_id_2)).tx_params(my_tx_params).call().await.unwrap();

//     let amount = 1000;

//     // Deposit 1000 token each
//     let contract_balances = wallet
//             .get_provider().unwrap()
//             .get_contract_balances(&Bech32ContractId::from(ContractId::from(*asset_id_1)))
//             .await;
//     assert!(contract_balances.unwrap().is_empty());

//     let result1 = _instance.methods().deposit(amount, ContractId::from(*asset_id_1))
//     .call_params(CallParameters::new(Some(1000),Some(asset_id_1),None))
//     .tx_params(my_tx_params).call().await.unwrap();

//     let result2 =  _instance.methods().deposit(amount, ContractId::from(*asset_id_2))
//     .call_params(CallParameters::new(Some(1000),Some(asset_id_2),None))
//     .tx_params(my_tx_params).call().await.unwrap();

//     // Now you have an instance of your contract you can use to test each function
//     let result = _instance.methods().quote(1000,0).tx_params(my_tx_params).call().await;
//     assert_eq!(result.unwrap().value ,500 );

//     Ok(())
// }

// // test add initial liquidity
// #[tokio::test]
// async fn can_add_liquidity_initial_no_transfer()-> Result<()>{
//     let (_instance, _id,  asset_id_1, asset_id_2,wallet) = get_contract_instance().await;
//     let my_tx_params = TxParameters::new(None, Some(1_000_000), None);
//     let result0 = _instance.methods().initialize(ContractId::from(*asset_id_1),ContractId::from(*asset_id_2)).tx_params(my_tx_params).call().await.unwrap();

//     let result = _instance.methods().add_liquidity().tx_params(my_tx_params).append_variable_outputs(1).call().await;
//     assert!(matches!(result, Err(prelude::Error::RevertTransactionError { .. })));
//     Ok(())
// }

// // test add initial liquidity
// #[tokio::test]
// async fn can_add_liquidity_initial() -> Result<()>{
//     let (_instance, _id,  asset_id_1, asset_id_2,wallet) = get_contract_instance().await;
//     let my_tx_params = TxParameters::new(None, Some(1_000_000), None);
//     let result0 = _instance.methods().initialize(ContractId::from(*asset_id_1),ContractId::from(*asset_id_2)).tx_params(my_tx_params).call().await.unwrap();
//     let amount = 1000;

//     // Now you have an instance of your contract you can use to test each function

//     let amount_0 = 1000;
//     let amount_1 = 1000;
//     let result_tx0 = wallet
//             .force_transfer_to_contract(&_id, amount_0, asset_id_1, my_tx_params)
//             .await?;
//     let result_tx1 = wallet
//             .force_transfer_to_contract(&_id, amount_1, asset_id_2, my_tx_params)
//             .await?;
//     let result = _instance.methods().add_liquidity().tx_params(my_tx_params).append_variable_outputs(1).call().await?;

//     assert_eq!(result.value, 999);

//     let contract_balances1 = _instance.get_balances().await?;
//     assert_eq!(contract_balances1.len(),4);
//     assert_eq!(contract_balances1.get(&format!("{asset_id_1:#x}")).unwrap(),&amount_0);
//     assert_eq!(contract_balances1.get(&format!("{asset_id_2:#x}")).unwrap(),&amount_1);

//     // check output lp token
//     let balances= wallet.get_balances().await?;
//     let lp_token = format!("{:#x}",ContractId::from(_id));
//     assert_eq!(balances.get(&lp_token).unwrap(),&999);
//     Ok(())
// }

// // test add additional liquidity
// #[tokio::test]
// async fn can_add_liquidity_additional() -> Result<()>{
//     let (_instance, _id,  asset_id_1, asset_id_2,wallet) = get_contract_instance().await;
//     let my_tx_params = TxParameters::new(None, Some(1_000_000), None);
//     let result0 = _instance.methods().initialize(ContractId::from(*asset_id_1),ContractId::from(*asset_id_2)).tx_params(my_tx_params).call().await.unwrap();
//     let amount = 1000;

//     // initial liquidity
//     let result_tx = wallet
//             .force_transfer_to_contract(&_instance.contract_id(), amount, asset_id_1, my_tx_params)
//             .await?;
//     let result_tx1 = wallet
//             .force_transfer_to_contract(&_instance.contract_id(), amount, asset_id_2, my_tx_params)
//             .await?;
//     let result = _instance.methods().add_liquidity().tx_params(my_tx_params).append_variable_outputs(1).call().await?;
//     assert_eq!(result.value, 999);

//     // Now you have an instance of your contract you can use to test each function

//     let amount0 = 100;
//     let amount1 = 100;
//     let result_tx2 = wallet
//             .force_transfer_to_contract(&_id, amount0, asset_id_1, my_tx_params)
//             .await?;
//     let result_tx3 = wallet
//             .force_transfer_to_contract(&_id, amount1, asset_id_2, my_tx_params)
//             .await?;
//     let result = _instance.methods().add_liquidity().tx_params(my_tx_params).append_variable_outputs(1).call().await?;

//     assert_eq!(result.value, 99);

//     let contract_balances1 = _instance.get_balances().await?;
//     assert_eq!(contract_balances1.len(),4);
//     assert_eq!(contract_balances1.get(&format!("{asset_id_1:#x}")).unwrap(),&(amount+amount0));
//     assert_eq!(contract_balances1.get(&format!("{asset_id_2:#x}")).unwrap(),&(amount+amount1));

//     // check output lp token
//     let balances= wallet.get_balances().await?;
//     let lp_token = format!("{:#x}",ContractId::from(_id));
//     assert_eq!(balances.get(&lp_token).unwrap(),&1098);

//     Ok(())
// }

// // test remove liquidity
// #[tokio::test]
// async fn can_remove_liquidity()-> Result<()> {
//     let (_instance, _id,  asset_id_1, asset_id_2,wallet) = get_contract_instance().await;
//     let my_tx_params = TxParameters::new(None, Some(1_000_000), None);
//     let result0 = _instance.methods().initialize(ContractId::from(*asset_id_1),ContractId::from(*asset_id_2)).tx_params(my_tx_params).call().await.unwrap();
//     let amount = 1000;

//     let result_tx = wallet
//             .force_transfer_to_contract(&_instance.contract_id(), amount, asset_id_1, my_tx_params)
//             .await?;
//     let contract_balances = _instance.get_balances().await?;

//     let result_tx1 = wallet
//             .force_transfer_to_contract(&_instance.contract_id(), amount, asset_id_2, my_tx_params)
//             .await?;
//     let contract_balances1 = _instance.get_balances().await?;

//     let result = _instance.methods().add_liquidity().tx_params(my_tx_params).append_variable_outputs(1).call().await?;
//     let balances1= wallet.get_balances().await?;
//     let value0 = balances1.get(&format!("{:#x}",ContractId::from(_instance.contract_id()))).unwrap();
//     let value1 = balances1.get(&format!("{asset_id_1:#x}")).unwrap();
//     let value2 = balances1.get(&format!("{asset_id_2:#x}")).unwrap();
//     // Now you have an instance of your contract you can use to test each function

//     let to_remove = 100;    
//     let result_tx1 = wallet
//         .force_transfer_to_contract(&_instance.contract_id(), to_remove, AssetId::from(*_id.hash()), my_tx_params)
//         .await?;
            
//     let result_remove = _instance.methods().remove_liquidity().tx_params(my_tx_params).append_variable_outputs(2).call().await?;
//     // compare wallet balances
//     let balances= wallet.get_balances().await?;
//     let lp_token = format!("{:#x}",ContractId::from(_id));
//     assert_eq!(balances.get(&format!("{:#x}",ContractId::from(_instance.contract_id()))).unwrap(),&(value0-100));
//     assert_eq!(balances.get(&format!("{asset_id_1:#x}")).unwrap(),&(value1+100));
//     assert_eq!(balances.get(&format!("{asset_id_2:#x}")).unwrap(),&(value2+100));

//     Ok(())
// }

// // test swap
// #[tokio::test]
// async fn can_swap() -> Result<()>{

//     let (_instance, _id,  asset_id_1, asset_id_2,wallet) = get_contract_instance().await;
//     let my_tx_params = TxParameters::new(None, Some(1_000_000), None);
//     let result0 = _instance.methods().initialize(ContractId::from(*asset_id_1),ContractId::from(*asset_id_2)).tx_params(my_tx_params).call().await.unwrap();
//     let amount = 1000;

//     let result_tx = wallet
//             .force_transfer_to_contract(&_instance.contract_id(), amount, asset_id_1, my_tx_params)
//             .await?;
//     let contract_balances = _instance.get_balances().await?;

//     let result_tx1 = wallet
//             .force_transfer_to_contract(&_instance.contract_id(), amount, asset_id_2, my_tx_params)
//             .await?;
//     let contract_balances1 = _instance.get_balances().await?;

//     let result = _instance.methods().add_liquidity().tx_params(my_tx_params).append_variable_outputs(1).call().await?;
//     let balances1= wallet.get_balances().await?;
//     let value1 = balances1.get(&format!("{asset_id_1:#x}")).unwrap();
//     let value2 = balances1.get(&format!("{asset_id_2:#x}")).unwrap();

//     // Now you have an instance of your contract you can use to test each function
//     let trade_amount = 100;
//     let result_tx2 = wallet
//             .force_transfer_to_contract(&_instance.contract_id(), trade_amount, asset_id_2, my_tx_params)
//             .await?;
//     let result = _instance.methods().swap().tx_params(my_tx_params).append_variable_outputs(1).call().await?;    

//     let balances= wallet.get_balances().await?;
//     let lp_token = format!("{:#x}",ContractId::from(_id));
//     assert_eq!(balances.get(&format!("{asset_id_1:#x}")).unwrap(),&(value1+90));
//     assert_eq!(balances.get(&format!("{asset_id_2:#x}")).unwrap(),&(value2-100));

//     Ok(())
// }