use std::{fmt::Display, str::FromStr};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, PartialOrd, Ord, ValueEnum)]
pub enum Blockchain {
    Bitcoin,
    Ethereum,
    Solana,
    Cardano,
    Base,
    Akash,
    Algorand,
    Aptos,
    Arbitrum,
    Cosmos,
    Avalanche,
    Axelar,
    BitcoinCash,
    Optimism,
    Polygon,
    Celo,
    Dash,
    Deso,
    Dogecoin,
    Polkadot,
    Elrond,
    Eosio,
    EthereumClassic,
    Filecoin,
    Flow,
    Flare,
    Hedera,
    Dfinity,
    Kava,
    Kusama,
    Litecoin,
    Mina,
    Near,
    Osmosis,
    Ronin,
    Oasis,
    Sei,
    Stacks,
    Sui,
    Celestia,
    Noble,
    Vara,
    VeChain,
    Stellar,
    Ripple,
    Tezos,
    Zcash,
    Horizen,
    Icp,
    Injective,
    Tron,
    Loki,
    Energi,
    Monero,
    RSK,
    BinanceSmartChain,
    TRTL,
    KucoinCommunityChain,
    Komodo,
    Nix,
    ThunderCore,
    Nimiq,
    Coti,
    Pivx,
    NEM,
    Sero,
    EOSForce,
    #[clap(skip)]
    Other(String)
}

impl Display for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Blockchain::Other(s) => s.fmt(f),
            _ => format!("{:?}", self).fmt(f)
        }
    }
}

impl<'de> Deserialize<'de> for Blockchain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;

        Ok(s.try_into().unwrap())
    }
}

impl FromStr for Blockchain {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_s = s.to_lowercase();
        match lower_s.as_str() {
            "eth" | "ethereum" | "erc20" => Ok(Self::Ethereum),
            "sol" | "solana" => Ok(Self::Solana),
            "btx" | "bitcoin" | "ordinals - brc20" => Ok(Self::Bitcoin),
            "ada" | "cardano" => Ok(Self::Cardano),
            "base" => Ok(Self::Base),
            "akash" => Ok(Self::Akash),
            "algo" | "algorand" => Ok(Self::Algorand),
            "apt" | "aptos" => Ok(Self::Aptos),
            "arb" | "arbitrum" => Ok(Self::Arbitrum),
            "atom" | "cosmos" => Ok(Self::Cosmos),
            "avax" | "avacchain" | "avalanche" | "avalanche c-chain" => Ok(Self::Avalanche),
            "axl" | "axelar" => Ok(Self::Axelar),
            "bch" | "bitcoin cash" => Ok(Self::BitcoinCash),
            "op" | "optimism" => Ok(Self::Optimism),
            "matic" | "polygon" => Ok(Self::Polygon),
            "celo" => Ok(Self::Celo),
            "dash" => Ok(Self::Dash),
            "deso" => Ok(Self::Deso),
            "doge" | "dogecoin" => Ok(Self::Dogecoin),
            "dot" | "polkadot" => Ok(Self::Polkadot),
            "elrond" => Ok(Self::Elrond),
            "eosio" => Ok(Self::Eosio),
            "etc" | "ethereumclassic" | "ethereum classic" => Ok(Self::EthereumClassic),
            "fil" | "filecoin" => Ok(Self::Filecoin),
            "flow" => Ok(Self::Flow),
            "flare" => Ok(Self::Flare),
            "hbar" | "hedera" => Ok(Self::Hedera),
            "dfinity" => Ok(Self::Dfinity),
            "kava" => Ok(Self::Kava),
            "ksm" | "kusama" => Ok(Self::Kusama),
            "ltc" | "litecoin" => Ok(Self::Litecoin),
            "mina" => Ok(Self::Mina),
            "near" => Ok(Self::Near),
            "osmo" | "osmosis" => Ok(Self::Osmosis),
            "ronin" => Ok(Self::Ronin),
            "oasis" => Ok(Self::Oasis),
            "sei" => Ok(Self::Sei),
            "stacks" => Ok(Self::Stacks),
            "sui" | "sui network" => Ok(Self::Sui),
            "celestia" => Ok(Self::Celestia),
            "noble" => Ok(Self::Noble),
            "vara" => Ok(Self::Vara),
            "vet" | "vechain" => Ok(Self::VeChain),
            "xlm" | "stellar" => Ok(Self::Stellar),
            "xrp" | "ripple" => Ok(Self::Ripple),
            "xtz" | "tezos" => Ok(Self::Tezos),
            "zec" | "zcash" => Ok(Self::Zcash),
            "zen" | "horizen" => Ok(Self::Horizen),
            "icp" => Ok(Self::Icp),
            "inj" | "injective" => Ok(Self::Injective),
            "trx" | "tron20" => Ok(Self::Tron),
            "loki" => Ok(Self::Loki),
            "nrg" => Ok(Self::Energi),
            "xmr" => Ok(Self::Monero),
            "rbtc" => Ok(Self::RSK),
            "bep20" | "bep2" => Ok(Self::BinanceSmartChain),
            "trtl" => Ok(Self::TRTL),
            "kcc" => Ok(Self::KucoinCommunityChain),
            "kmd" => Ok(Self::Komodo),
            "nix" => Ok(Self::Nix),
            "tt" => Ok(Self::ThunderCore),
            "nim" => Ok(Self::Nimiq),
            "coti" => Ok(Self::Coti),
            "pivx" => Ok(Self::Pivx),
            "nem" => Ok(Self::NEM),
            "sero" => Ok(Self::Sero),
            "eosc" => Ok(Self::EOSForce),
            _ => Ok(Self::Other(s.to_string()))
        }
    }
}
