use std::path::Path;

use config::{
    forge_interface::deploy_ecosystem::input::{Erc20DeploymentConfig, InitialDeploymentConfig},
    traits::{ReadConfig, SaveConfigWithBasePath, SaveConfigWithCommentAndBasePath},
    AppsEcosystemConfig, ContractsConfig, ECOSYSTEM_PATH, MAINNET_FILE, SEPOLIA_FILE,
};
use types::L1Network;
use xshell::Shell;

use crate::messages::{MSG_SAVE_ERC20_CONFIG_ATTENTION, MSG_SAVE_INITIAL_CONFIG_ATTENTION};

pub fn create_initial_deployments_config(
    shell: &Shell,
    ecosystem_configs_path: &Path,
) -> anyhow::Result<InitialDeploymentConfig> {
    let config = InitialDeploymentConfig::default();
    config.save_with_comment_and_base_path(
        shell,
        ecosystem_configs_path,
        MSG_SAVE_INITIAL_CONFIG_ATTENTION,
    )?;
    Ok(config)
}

pub fn create_erc20_deployment_config(
    shell: &Shell,
    ecosystem_configs_path: &Path,
) -> anyhow::Result<Erc20DeploymentConfig> {
    let config = Erc20DeploymentConfig::default();
    config.save_with_comment_and_base_path(
        shell,
        ecosystem_configs_path,
        MSG_SAVE_ERC20_CONFIG_ATTENTION,
    )?;
    Ok(config)
}

pub fn create_apps_config(
    shell: &Shell,
    ecosystem_configs_path: &Path,
) -> anyhow::Result<AppsEcosystemConfig> {
    let config = AppsEcosystemConfig::default();
    config.save_with_base_path(shell, ecosystem_configs_path)?;
    Ok(config)
}

pub fn copy_official_zksync_contracts(
    shell: &Shell,
    base_path: &Path,
    link_to_code: &Path,
    network: L1Network,
) -> anyhow::Result<()> {
    let path = link_to_code.join(ECOSYSTEM_PATH);
    let contracts_path = match network {
        L1Network::Mainnet => path.join(MAINNET_FILE),
        L1Network::Sepolia => path.join(SEPOLIA_FILE),
        _ => anyhow::bail!("Official bridge is only available for sepolia and mainnet"),
    };
    let contracts = ContractsConfig::read(shell, contracts_path)?;
    contracts.save_with_base_path(shell, base_path)?;
    Ok(())
}
