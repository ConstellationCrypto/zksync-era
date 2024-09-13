use std::path::{Path, PathBuf};

use common::wallets::Wallet;
use rand::thread_rng;
use types::{L1Network, WalletCreation};
use xshell::Shell;

use crate::{
    consts::{BASE_PATH, CONFIGS_PATH, MAINNET_FILE, SEPOLIA_FILE, TEST_CONFIG_PATH, WALLETS_DIR},
    traits::{ReadConfig, SaveConfigWithBasePath},
    EthMnemonicConfig, WalletsConfig,
};

pub fn create_wallets(
    shell: &Shell,
    base_path: &Path,
    link_to_code: &Path,
    id: u32,
    wallet_creation: WalletCreation,
    initial_wallet_path: Option<PathBuf>,
    wallets: Option<WalletsConfig>,
) -> anyhow::Result<()> {
    let wallets = match wallet_creation {
        WalletCreation::Random => {
            match wallets {
                None => {
                    let rng = &mut thread_rng();
                    WalletsConfig::random(rng)
                }
                Some(i) => i
            }
        }
        WalletCreation::Empty => WalletsConfig::empty(),
        // Use id of chain for creating
        WalletCreation::Localhost => create_localhost_wallets(shell, link_to_code, id)?,
        WalletCreation::InFile => {
            let path = initial_wallet_path.ok_or(anyhow::anyhow!(
                "Wallet path for in file option is required"
            ))?;
            WalletsConfig::read(shell, path)?
        }
    };

    wallets.save_with_base_path(shell, base_path)?;
    Ok(())
}

// Create wallets based on id
pub fn create_localhost_wallets(
    shell: &Shell,
    link_to_code: &Path,
    id: u32,
) -> anyhow::Result<WalletsConfig> {
    let path = link_to_code.join(TEST_CONFIG_PATH);
    let eth_mnemonic = EthMnemonicConfig::read(shell, path)?;
    let base_path = format!("{}/{}", BASE_PATH, id);
    Ok(WalletsConfig {
        deployer: Some(Wallet::from_mnemonic(
            &eth_mnemonic.test_mnemonic,
            &base_path,
            0,
        )?),
        operator: Wallet::from_mnemonic(&eth_mnemonic.test_mnemonic, &base_path, 1)?,
        blob_operator: Wallet::from_mnemonic(&eth_mnemonic.test_mnemonic, &base_path, 2)?,
        fee_account: Wallet::from_mnemonic(&eth_mnemonic.test_mnemonic, &base_path, 3)?,
        governor: Wallet::from_mnemonic(&eth_mnemonic.test_mnemonic, &base_path, 4)?,
        token_multiplier_setter: Some(Wallet::from_mnemonic(
            &eth_mnemonic.test_mnemonic,
            &base_path,
            5,
        )?),
    })
}

pub fn copy_official_zksync_wallets(
    shell: &Shell,
    base_path: &Path,
    link_to_code: &Path,
    network: L1Network,
) -> anyhow::Result<()> {
    let path = link_to_code.join(CONFIGS_PATH).join(WALLETS_DIR);
    let wallets_path = match network {
        L1Network::Mainnet => path.join(MAINNET_FILE),
        L1Network::Sepolia => path.join(SEPOLIA_FILE),
        _ => anyhow::bail!("Official bridge is only available for sepolia and mainnet"),
    };
    let wallets = WalletsConfig::read(shell, wallets_path)?;
    wallets.save_with_base_path(shell, base_path)?;
    Ok(())
}
