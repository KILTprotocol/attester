mod did;
mod tx;
mod utils;
mod well_known_did_configuration;

use subxt::{
    config::polkadot::PolkadotExtrinsicParams,
    config::Config,
    ext::{
        sp_core, sp_runtime,
        sp_runtime::traits::{IdentifyAccount, Verify},
    },
};

pub use did::{get_encryption_key_from_fulldid_key_uri, parse_encryption_key_from_lightdid};
pub use tx::*;
pub use well_known_did_configuration::*;

#[cfg(feature = "spiritnet")]
#[subxt::subxt(runtime_metadata_path = "./metadata_spiritnet_11401.scale")]
pub mod runtime {}

#[cfg(feature = "spiritnet")]
pub type RuntimeCall = runtime::runtime_types::spiritnet_runtime::RuntimeCall;

#[cfg(feature = "peregrine")]
#[subxt::subxt(runtime_metadata_path = "./metadata_peregrine_11401.scale")]
pub mod runtime {}

#[cfg(feature = "peregrine")]
pub type RuntimeCall = runtime::runtime_types::peregrine_runtime::RuntimeCall;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KiltConfig;
impl Config for KiltConfig {
    type Hash = sp_core::H256;
    type Hasher = <subxt::config::SubstrateConfig as Config>::Hasher;
    type AccountId = <<Self::Signature as Verify>::Signer as IdentifyAccount>::AccountId;
    type Address = sp_runtime::MultiAddress<Self::AccountId, ()>;
    type Header = subxt::config::substrate::SubstrateHeader<u64, Self::Hasher>;
    type Signature = sp_runtime::MultiSignature;
    type ExtrinsicParams = PolkadotExtrinsicParams<Self>;
}
