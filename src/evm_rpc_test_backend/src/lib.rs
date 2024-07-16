mod evm_types;
mod logger;

use evm_types::{
    BlockTag, EthMainnetService, EthSepoliaService, FeeHistoryArgs, FeeHistoryResult,
    L2MainnetService, MultiFeeHistoryResult, RpcServices, EVM_RPC,
};
use ic_cdk::{init, query};
use lazy_static::lazy_static;
use logger::{error, info, LogItem};
use num_traits::Zero;
use std::time::Duration;

const CYCLES: u128 = 30_000_000_000;
const BLOCK_COUNT: u64 = 4;

lazy_static! {
    static ref RPC_SERVICES: Vec<RpcServices> = vec![
        RpcServices::EthMainnet(Some(vec![EthMainnetService::Alchemy])),
        RpcServices::EthMainnet(Some(vec![EthMainnetService::BlockPi])),
        RpcServices::EthMainnet(Some(vec![EthMainnetService::Cloudflare])),
        RpcServices::EthMainnet(Some(vec![EthMainnetService::PublicNode])),
        RpcServices::EthMainnet(Some(vec![EthMainnetService::Ankr])),
        RpcServices::EthSepolia(Some(vec![EthSepoliaService::Alchemy])),
        RpcServices::EthSepolia(Some(vec![EthSepoliaService::BlockPi])),
        RpcServices::EthSepolia(Some(vec![EthSepoliaService::PublicNode])),
        RpcServices::EthSepolia(Some(vec![EthSepoliaService::Ankr])),
        RpcServices::OptimismMainnet(Some(vec![L2MainnetService::Alchemy])),
        RpcServices::OptimismMainnet(Some(vec![L2MainnetService::BlockPi])),
        RpcServices::OptimismMainnet(Some(vec![L2MainnetService::PublicNode])),
        RpcServices::OptimismMainnet(Some(vec![L2MainnetService::Ankr])),
        RpcServices::BaseMainnet(Some(vec![L2MainnetService::Alchemy])),
        RpcServices::BaseMainnet(Some(vec![L2MainnetService::BlockPi])),
        RpcServices::BaseMainnet(Some(vec![L2MainnetService::PublicNode])),
        RpcServices::BaseMainnet(Some(vec![L2MainnetService::Ankr])),
        RpcServices::ArbitrumOne(Some(vec![L2MainnetService::Alchemy])),
        RpcServices::ArbitrumOne(Some(vec![L2MainnetService::BlockPi])),
        RpcServices::ArbitrumOne(Some(vec![L2MainnetService::PublicNode])),
        RpcServices::ArbitrumOne(Some(vec![L2MainnetService::Ankr])),
    ];
}

fn get_fees(rpc_services: RpcServices) {
    ic_cdk::spawn(async move {
        let result = EVM_RPC
            .eth_fee_history(
                rpc_services.clone(),
                None,
                FeeHistoryArgs {
                    blockCount: BLOCK_COUNT.into(),
                    newestBlock: BlockTag::Latest,
                    rewardPercentiles: None,
                },
                CYCLES,
            )
            .await;
        match result {
            Ok((MultiFeeHistoryResult::Consistent(FeeHistoryResult::Ok(fee_history)),)) => {
                match fee_history {
                    Some(fee_history) => {
                        if fee_history.baseFeePerGas.is_empty() {
                            error(
                                format!("🛑, {:?}, No baseFeePerGas returned.", rpc_services)
                                    .as_str(),
                            );
                            return;
                        }

                        for base_fee in fee_history.baseFeePerGas.iter() {
                            if base_fee.0.is_zero() {
                                error(
                                    format!(
                                        "🛑, {:?}, baseFeePerGas is 0. baseFeePerGas: {:?}",
                                        rpc_services, fee_history.baseFeePerGas
                                    )
                                    .as_str(),
                                );
                                return;
                            }
                        }

                        info(format!("✅, {:?}", rpc_services).as_str());
                    }
                    None => {
                        error(format!("🛑, {:?}, No fee history returned.", rpc_services).as_str());
                    }
                };
            }
            Ok((MultiFeeHistoryResult::Consistent(FeeHistoryResult::Err(e)),)) => {
                error(format!("🛑, {:?}, Consistent result / Err: {:?}", rpc_services, e).as_str());
            }
            Ok((MultiFeeHistoryResult::Inconsistent(_),)) => {
                error(format!("🛑, {:?}, Inconsistent result", rpc_services).as_str());
            }
            Err(e) => {
                error(format!("🛑, {:?}, Err: {:?}", rpc_services, e).as_str());
            }
        }
    });
}

#[init]
fn init() {
    let wait = 10;
    for (index, rpc_services) in RPC_SERVICES.iter().enumerate() {
        let delay = Duration::from_secs(wait * index as u64);
        ic_cdk_timers::set_timer(delay, move || {
            get_fees(rpc_services.clone());
        });
    }
}

#[query]
pub fn logs() -> Vec<LogItem> {
    logger::get()
}
