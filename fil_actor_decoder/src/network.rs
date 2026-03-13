//! Network and epoch to actor version resolution.

use crate::ActorVersion;
use anyhow::{Result, bail};

/// Supported Filecoin networks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Calibnet,
}

impl std::str::FromStr for Network {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mainnet" | "main" => Ok(Network::Mainnet),
            "calibnet" | "calibration" | "calib" => Ok(Network::Calibnet),
            _ => bail!("Unknown network: {s}. Supported: mainnet, calibnet"),
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Calibnet => write!(f, "calibnet"),
        }
    }
}

/// Resolve an actor version from a network and epoch.
pub fn resolve_actor_version(network: Network, epoch: i64) -> Result<ActorVersion> {
    let nv = epoch_to_network_version(network, epoch)?;
    network_version_to_actor_version(nv)
}

type NetworkVersion = u32;

fn epoch_to_network_version(network: Network, epoch: i64) -> Result<NetworkVersion> {
    let upgrades = match network {
        Network::Mainnet => MAINNET_UPGRADES,
        Network::Calibnet => CALIBNET_UPGRADES,
    };

    let mut nv = 0u32;
    for &(upgrade_epoch, version) in upgrades {
        if epoch >= upgrade_epoch {
            nv = version;
        } else {
            break;
        }
    }

    if nv < 17 {
        bail!(
            "Epoch {epoch} on {network} is before NV17 (Shark). \
             Actor versions before v9 are not supported."
        );
    }

    Ok(nv)
}

fn network_version_to_actor_version(nv: NetworkVersion) -> Result<ActorVersion> {
    match nv {
        17 => Ok(ActorVersion::V9),
        18 => Ok(ActorVersion::V10),
        19 | 20 => Ok(ActorVersion::V11),
        21 => Ok(ActorVersion::V12),
        22 => Ok(ActorVersion::V13),
        23 => Ok(ActorVersion::V14),
        24 => Ok(ActorVersion::V15),
        25 | 26 => Ok(ActorVersion::V16),
        27 => Ok(ActorVersion::V17),
        _ => bail!("Unsupported network version: NV{nv}"),
    }
}

static MAINNET_UPGRADES: &[(i64, NetworkVersion)] = &[
    (0, 0),
    (41_280, 1),
    (51_000, 2),
    (94_000, 3),
    (138_720, 4),
    (140_760, 5),
    (170_000, 6),
    (265_200, 7),
    (272_400, 8),
    (336_458, 9),
    (550_321, 10),
    (665_280, 11),
    (712_320, 12),
    (892_800, 13),
    (1_231_620, 14),
    (1_594_680, 15),
    (1_960_320, 16),
    (2_383_680, 17),     // Shark
    (2_683_348, 18),     // Hygge
    (2_809_800, 19),     // Lightning
    (2_870_280, 20),     // Thunder
    (3_469_380, 21),     // Watermelon
    (3_855_360, 22),     // Dragon
    (4_154_640, 23),     // Waffle
    (4_461_240, 24),     // TukTuk
    (4_878_840, 25),     // Teep
    (4_900_440, 26),     // Tock
    (5_348_280, 27),     // GoldenWeek
];

static CALIBNET_UPGRADES: &[(i64, NetworkVersion)] = &[
    (0, 0),
    (30, 4),
    (60, 5),
    (90, 6),
    (120, 7),
    (240, 8),
    (270, 9),
    (330, 10),
    (360, 11),
    (390, 12),
    (420, 13),
    (450, 14),
    (480, 15),
    (510, 16),
    (16_800, 17),        // Shark
    (322_354, 18),       // Hygge
    (489_094, 19),       // Lightning
    (492_214, 20),       // Thunder
    (1_013_134, 21),     // Watermelon
    (1_427_974, 22),     // Dragon
    (1_779_094, 23),     // Waffle
    (2_078_794, 24),     // TukTuk
    (2_523_454, 25),     // Teep
    (2_543_614, 26),     // Tock
    (3_007_294, 27),     // GoldenWeek
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_current_epoch() {
        let v = resolve_actor_version(Network::Mainnet, 5_833_541).unwrap();
        assert_eq!(v, ActorVersion::V17);
    }

    #[test]
    fn test_mainnet_dragon() {
        let v = resolve_actor_version(Network::Mainnet, 3_900_000).unwrap();
        assert_eq!(v, ActorVersion::V13);
    }

    #[test]
    fn test_mainnet_watermelon() {
        let v = resolve_actor_version(Network::Mainnet, 3_469_380).unwrap();
        assert_eq!(v, ActorVersion::V12);
    }

    #[test]
    fn test_mainnet_shark() {
        let v = resolve_actor_version(Network::Mainnet, 2_383_680).unwrap();
        assert_eq!(v, ActorVersion::V9);
    }

    #[test]
    fn test_mainnet_before_shark_errors() {
        let r = resolve_actor_version(Network::Mainnet, 2_000_000);
        assert!(r.is_err());
    }

    #[test]
    fn test_calibnet_current() {
        let v = resolve_actor_version(Network::Calibnet, 3_100_000).unwrap();
        assert_eq!(v, ActorVersion::V17);
    }

    #[test]
    fn test_nv20_uses_v11() {
        let v = resolve_actor_version(Network::Mainnet, 2_870_280).unwrap();
        assert_eq!(v, ActorVersion::V11);
    }

    #[test]
    fn test_nv26_uses_v16() {
        let v = resolve_actor_version(Network::Mainnet, 4_900_440).unwrap();
        assert_eq!(v, ActorVersion::V16);
    }
}
