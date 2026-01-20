//! Bitcoin blockchain support.
//!
//! This module provides types for:
//! - Address formats (Legacy, SegWit, Taproot)
//! - UTXO management
//! - Fee estimation
//! - PSBT creation and signing
//! - Deterministic UTXO selection

use crate::core::{Base58String, Bech32String, BigUint, Hash32, HexString};
use crate::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ============================================================================
// Network Configuration
// ============================================================================

/// Bitcoin network type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitcoinNetwork {
    /// Bitcoin mainnet
    Mainnet,
    /// Bitcoin testnet3
    Testnet,
    /// Bitcoin signet
    Signet,
    /// Bitcoin regtest
    Regtest,
}

impl BitcoinNetwork {
    /// Get the Bech32 human-readable part for this network.
    pub fn bech32_hrp(&self) -> &'static str {
        match self {
            Self::Mainnet => "bc",
            Self::Testnet | Self::Signet => "tb",
            Self::Regtest => "bcrt",
        }
    }

    /// Get the P2PKH version byte.
    pub fn p2pkh_version(&self) -> u8 {
        match self {
            Self::Mainnet => 0x00,
            Self::Testnet | Self::Signet | Self::Regtest => 0x6f,
        }
    }

    /// Get the P2SH version byte.
    pub fn p2sh_version(&self) -> u8 {
        match self {
            Self::Mainnet => 0x05,
            Self::Testnet | Self::Signet | Self::Regtest => 0xc4,
        }
    }

    /// Get the WIF version byte.
    pub fn wif_version(&self) -> u8 {
        match self {
            Self::Mainnet => 0x80,
            Self::Testnet | Self::Signet | Self::Regtest => 0xef,
        }
    }
}

impl Default for BitcoinNetwork {
    fn default() -> Self {
        Self::Mainnet
    }
}

impl fmt::Display for BitcoinNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mainnet => write!(f, "mainnet"),
            Self::Testnet => write!(f, "testnet"),
            Self::Signet => write!(f, "signet"),
            Self::Regtest => write!(f, "regtest"),
        }
    }
}

// ============================================================================
// Address Types
// ============================================================================

/// Bitcoin address format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressFormat {
    /// P2PKH (Pay to Public Key Hash) - Legacy, starts with 1/m/n
    P2pkh,
    /// P2SH (Pay to Script Hash) - starts with 3/2
    P2sh,
    /// P2WPKH (Pay to Witness Public Key Hash) - Native SegWit, starts with bc1q
    P2wpkh,
    /// P2WSH (Pay to Witness Script Hash) - Native SegWit, starts with bc1q
    P2wsh,
    /// P2TR (Pay to Taproot) - Taproot, starts with bc1p
    P2tr,
    /// P2SH-P2WPKH (Nested SegWit) - starts with 3/2
    P2shP2wpkh,
}

impl AddressFormat {
    /// Get the witness version (None for legacy).
    pub fn witness_version(&self) -> Option<u8> {
        match self {
            Self::P2pkh | Self::P2sh | Self::P2shP2wpkh => None,
            Self::P2wpkh | Self::P2wsh => Some(0),
            Self::P2tr => Some(1),
        }
    }

    /// Check if this is a SegWit address.
    pub fn is_segwit(&self) -> bool {
        matches!(self, Self::P2wpkh | Self::P2wsh | Self::P2tr | Self::P2shP2wpkh)
    }

    /// Check if this is a native SegWit address.
    pub fn is_native_segwit(&self) -> bool {
        matches!(self, Self::P2wpkh | Self::P2wsh | Self::P2tr)
    }
}

impl fmt::Display for AddressFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::P2pkh => write!(f, "P2PKH"),
            Self::P2sh => write!(f, "P2SH"),
            Self::P2wpkh => write!(f, "P2WPKH"),
            Self::P2wsh => write!(f, "P2WSH"),
            Self::P2tr => write!(f, "P2TR"),
            Self::P2shP2wpkh => write!(f, "P2SH-P2WPKH"),
        }
    }
}

/// A Bitcoin address.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BitcoinAddress {
    /// Address format
    pub format: AddressFormat,
    /// Network
    pub network: BitcoinNetwork,
    /// The address payload (hash)
    payload: Vec<u8>,
    /// Original string representation
    string: String,
}

impl BitcoinAddress {
    /// Parse a Bitcoin address string.
    pub fn parse(s: &str, network: BitcoinNetwork) -> CryptoResult<Self> {
        let s = s.trim();

        // Try Bech32 first (bc1...)
        if s.to_lowercase().starts_with(network.bech32_hrp()) {
            return Self::parse_bech32(s, network);
        }

        // Try Base58
        Self::parse_base58(s, network)
    }

    /// Parse a Bech32 address.
    fn parse_bech32(s: &str, network: BitcoinNetwork) -> CryptoResult<Self> {
        let bech32 = Bech32String::parse(s)?;

        if bech32.hrp() != network.bech32_hrp() {
            return Err(CryptoError::InvalidAddress(format!(
                "wrong network: expected {}, got {}",
                network.bech32_hrp(),
                bech32.hrp()
            )));
        }

        let data = bech32.data();
        if data.is_empty() {
            return Err(CryptoError::InvalidAddress("empty data".to_string()));
        }

        let witness_version = data[0];
        let payload: Vec<u8> = data[1..].to_vec();

        let format = match (witness_version, payload.len()) {
            (0, 20) => AddressFormat::P2wpkh,
            (0, 32) => AddressFormat::P2wsh,
            (1, 32) => AddressFormat::P2tr,
            _ => {
                return Err(CryptoError::InvalidAddress(format!(
                    "unknown witness program: version={}, len={}",
                    witness_version,
                    payload.len()
                )));
            }
        };

        Ok(Self {
            format,
            network,
            payload,
            string: s.to_lowercase(),
        })
    }

    /// Parse a Base58 address.
    fn parse_base58(s: &str, network: BitcoinNetwork) -> CryptoResult<Self> {
        let decoded = Base58String::parse(s)?.to_bytes()?;

        if decoded.len() < 5 {
            return Err(CryptoError::InvalidAddress("too short".to_string()));
        }

        let version = decoded[0];
        let payload = decoded[1..decoded.len() - 4].to_vec();
        let _checksum = &decoded[decoded.len() - 4..];

        // TODO: Verify checksum

        let format = if version == network.p2pkh_version() {
            AddressFormat::P2pkh
        } else if version == network.p2sh_version() {
            AddressFormat::P2sh
        } else {
            return Err(CryptoError::InvalidAddress(format!(
                "unknown version byte: 0x{:02x}",
                version
            )));
        };

        Ok(Self {
            format,
            network,
            payload,
            string: s.to_string(),
        })
    }

    /// Get the address string.
    pub fn as_str(&self) -> &str {
        &self.string
    }

    /// Get the payload (hash).
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Create a P2WPKH address from a public key hash.
    pub fn p2wpkh(pubkey_hash: [u8; 20], network: BitcoinNetwork) -> Self {
        let bech32 = format!(
            "{}1q{}",
            network.bech32_hrp(),
            // Simplified: real implementation would properly encode
            HexString::from_bytes(&pubkey_hash)
        );

        Self {
            format: AddressFormat::P2wpkh,
            network,
            payload: pubkey_hash.to_vec(),
            string: bech32,
        }
    }

    /// Create a P2TR address from a tweaked public key.
    pub fn p2tr(output_key: [u8; 32], network: BitcoinNetwork) -> Self {
        let bech32 = format!(
            "{}1p{}",
            network.bech32_hrp(),
            // Simplified: real implementation would properly encode
            HexString::from_bytes(&output_key)
        );

        Self {
            format: AddressFormat::P2tr,
            network,
            payload: output_key.to_vec(),
            string: bech32,
        }
    }
}

impl fmt::Display for BitcoinAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl FromStr for BitcoinAddress {
    type Err = CryptoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try mainnet first, then testnet
        Self::parse(s, BitcoinNetwork::Mainnet)
            .or_else(|_| Self::parse(s, BitcoinNetwork::Testnet))
    }
}

// ============================================================================
// UTXO Types
// ============================================================================

/// An unspent transaction output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    /// Transaction ID
    pub txid: Hash32,
    /// Output index
    pub vout: u32,
    /// Value in satoshis
    pub value: u64,
    /// Script pubkey (hex)
    pub script_pubkey: HexString,
    /// Address (if known)
    pub address: Option<BitcoinAddress>,
    /// Block height (None if unconfirmed)
    pub block_height: Option<u64>,
    /// Confirmations
    pub confirmations: u32,
}

impl Utxo {
    /// Create a new UTXO.
    pub fn new(txid: Hash32, vout: u32, value: u64, script_pubkey: HexString) -> Self {
        Self {
            txid,
            vout,
            value,
            script_pubkey,
            address: None,
            block_height: None,
            confirmations: 0,
        }
    }

    /// Get the outpoint as a string (txid:vout).
    pub fn outpoint(&self) -> String {
        format!("{}:{}", self.txid, self.vout)
    }

    /// Check if the UTXO is confirmed.
    pub fn is_confirmed(&self) -> bool {
        self.confirmations > 0
    }

    /// Check if the UTXO has sufficient confirmations for spending.
    pub fn is_spendable(&self, min_confirmations: u32) -> bool {
        self.confirmations >= min_confirmations
    }
}

/// A set of UTXOs for a wallet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UtxoSet {
    /// The UTXOs
    utxos: Vec<Utxo>,
}

impl UtxoSet {
    /// Create a new empty UTXO set.
    pub fn new() -> Self {
        Self { utxos: Vec::new() }
    }

    /// Add a UTXO.
    pub fn add(&mut self, utxo: Utxo) {
        self.utxos.push(utxo);
    }

    /// Remove a UTXO by outpoint.
    pub fn remove(&mut self, txid: &Hash32, vout: u32) -> Option<Utxo> {
        if let Some(idx) = self.utxos.iter().position(|u| &u.txid == txid && u.vout == vout) {
            Some(self.utxos.remove(idx))
        } else {
            None
        }
    }

    /// Get all UTXOs.
    pub fn all(&self) -> &[Utxo] {
        &self.utxos
    }

    /// Get confirmed UTXOs.
    pub fn confirmed(&self, min_confirmations: u32) -> Vec<&Utxo> {
        self.utxos
            .iter()
            .filter(|u| u.is_spendable(min_confirmations))
            .collect()
    }

    /// Get total balance in satoshis.
    pub fn balance(&self) -> u64 {
        self.utxos.iter().map(|u| u.value).sum()
    }

    /// Get confirmed balance in satoshis.
    pub fn confirmed_balance(&self, min_confirmations: u32) -> u64 {
        self.utxos
            .iter()
            .filter(|u| u.is_spendable(min_confirmations))
            .map(|u| u.value)
            .sum()
    }

    /// Get the number of UTXOs.
    pub fn len(&self) -> usize {
        self.utxos.len()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.utxos.is_empty()
    }
}

// ============================================================================
// UTXO Selection
// ============================================================================

/// UTXO selection algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtxoSelectionStrategy {
    /// Select largest UTXOs first
    LargestFirst,
    /// Select smallest UTXOs first (consolidation)
    SmallestFirst,
    /// Select oldest UTXOs first (FIFO)
    OldestFirst,
    /// Branch and bound optimal selection
    BranchAndBound,
    /// Random selection (for privacy)
    Random,
}

/// Configuration for UTXO selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoSelectionConfig {
    /// Selection strategy
    pub strategy: UtxoSelectionStrategy,
    /// Minimum confirmations required
    pub min_confirmations: u32,
    /// Whether to allow unconfirmed UTXOs
    pub allow_unconfirmed: bool,
    /// Maximum number of inputs
    pub max_inputs: Option<usize>,
    /// Dust threshold in satoshis
    pub dust_threshold: u64,
}

impl Default for UtxoSelectionConfig {
    fn default() -> Self {
        Self {
            strategy: UtxoSelectionStrategy::BranchAndBound,
            min_confirmations: 1,
            allow_unconfirmed: false,
            max_inputs: Some(100),
            dust_threshold: 546, // P2WPKH dust limit
        }
    }
}

/// Result of UTXO selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoSelection {
    /// Selected UTXOs
    pub selected: Vec<Utxo>,
    /// Total value of selected UTXOs
    pub total_value: u64,
    /// Target amount
    pub target: u64,
    /// Change amount (if any)
    pub change: u64,
    /// Estimated fee
    pub fee: u64,
}

impl UtxoSelection {
    /// Check if selection is sufficient.
    pub fn is_sufficient(&self) -> bool {
        self.total_value >= self.target + self.fee
    }

    /// Get the number of inputs.
    pub fn input_count(&self) -> usize {
        self.selected.len()
    }
}

// ============================================================================
// Fee Estimation
// ============================================================================

/// Fee rate in satoshis per virtual byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeeRate(pub u64);

impl FeeRate {
    /// Create from sat/vB.
    pub fn from_sat_per_vb(sat_per_vb: u64) -> Self {
        Self(sat_per_vb)
    }

    /// Create from sat/kWU (sat per 1000 weight units).
    pub fn from_sat_per_kwu(sat_per_kwu: u64) -> Self {
        Self(sat_per_kwu / 4)
    }

    /// Get the rate in sat/vB.
    pub fn sat_per_vb(&self) -> u64 {
        self.0
    }

    /// Calculate fee for a given virtual size.
    pub fn fee_for_vsize(&self, vsize: u64) -> u64 {
        self.0 * vsize
    }

    /// Calculate fee for a given weight.
    pub fn fee_for_weight(&self, weight: u64) -> u64 {
        // vsize = ceil(weight / 4)
        let vsize = (weight + 3) / 4;
        self.fee_for_vsize(vsize)
    }
}

impl Default for FeeRate {
    fn default() -> Self {
        Self(1) // 1 sat/vB minimum
    }
}

/// Fee estimation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeeEstimateTarget {
    /// High priority (next block)
    High,
    /// Medium priority (~3 blocks)
    Medium,
    /// Low priority (~6 blocks)
    Low,
    /// Economy (~12 blocks)
    Economy,
    /// Specific number of blocks
    Blocks(u32),
}

impl FeeEstimateTarget {
    /// Get the target block count.
    pub fn blocks(&self) -> u32 {
        match self {
            Self::High => 1,
            Self::Medium => 3,
            Self::Low => 6,
            Self::Economy => 12,
            Self::Blocks(n) => *n,
        }
    }
}

/// Fee estimates from the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimates {
    /// High priority fee rate
    pub high: FeeRate,
    /// Medium priority fee rate
    pub medium: FeeRate,
    /// Low priority fee rate
    pub low: FeeRate,
    /// Economy fee rate
    pub economy: FeeRate,
    /// Minimum relay fee
    pub minimum: FeeRate,
}

impl Default for FeeEstimates {
    fn default() -> Self {
        Self {
            high: FeeRate(50),
            medium: FeeRate(20),
            low: FeeRate(10),
            economy: FeeRate(5),
            minimum: FeeRate(1),
        }
    }
}

// ============================================================================
// PSBT Types
// ============================================================================

/// A Partially Signed Bitcoin Transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Psbt {
    /// Version
    pub version: u32,
    /// Unsigned transaction
    pub unsigned_tx: UnsignedTransaction,
    /// Inputs
    pub inputs: Vec<PsbtInput>,
    /// Outputs
    pub outputs: Vec<PsbtOutput>,
    /// Global proprietary data
    pub proprietary: Vec<ProprietaryData>,
}

impl Psbt {
    /// Create a new PSBT from an unsigned transaction.
    pub fn new(unsigned_tx: UnsignedTransaction) -> Self {
        let input_count = unsigned_tx.inputs.len();
        let output_count = unsigned_tx.outputs.len();

        Self {
            version: 0,
            unsigned_tx,
            inputs: vec![PsbtInput::default(); input_count],
            outputs: vec![PsbtOutput::default(); output_count],
            proprietary: vec![],
        }
    }

    /// Serialize to base64.
    pub fn to_base64(&self) -> String {
        // Placeholder: real implementation would serialize to PSBT format
        String::new()
    }

    /// Parse from base64.
    pub fn from_base64(_s: &str) -> CryptoResult<Self> {
        // Placeholder: real implementation would parse PSBT format
        Err(CryptoError::FeatureNotAvailable(
            "PSBT parsing not implemented".to_string(),
        ))
    }

    /// Check if all inputs are signed.
    pub fn is_fully_signed(&self) -> bool {
        self.inputs.iter().all(|i| i.is_signed())
    }

    /// Check if the PSBT is finalizable.
    pub fn is_finalizable(&self) -> bool {
        self.is_fully_signed()
    }

    /// Finalize the PSBT and extract the signed transaction.
    pub fn finalize(self) -> CryptoResult<SignedBitcoinTransaction> {
        if !self.is_fully_signed() {
            return Err(CryptoError::SigningFailed);
        }

        // Placeholder: real implementation would finalize the PSBT
        Ok(SignedBitcoinTransaction {
            raw: vec![],
            txid: Hash32::zero(),
            wtxid: Hash32::zero(),
            size: 0,
            vsize: 0,
            weight: 0,
        })
    }
}

/// An unsigned Bitcoin transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    /// Transaction version
    pub version: i32,
    /// Lock time
    pub lock_time: u32,
    /// Inputs
    pub inputs: Vec<TxInput>,
    /// Outputs
    pub outputs: Vec<TxOutput>,
}

/// A transaction input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    /// Previous output transaction ID
    pub previous_output_txid: Hash32,
    /// Previous output index
    pub previous_output_vout: u32,
    /// Sequence number
    pub sequence: u32,
}

/// A transaction output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    /// Value in satoshis
    pub value: u64,
    /// Script pubkey
    pub script_pubkey: HexString,
}

/// PSBT input data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PsbtInput {
    /// Non-witness UTXO
    pub non_witness_utxo: Option<Vec<u8>>,
    /// Witness UTXO
    pub witness_utxo: Option<TxOutput>,
    /// Partial signatures (pubkey -> signature)
    pub partial_sigs: Vec<(HexString, HexString)>,
    /// Sighash type
    pub sighash_type: Option<u32>,
    /// Redeem script
    pub redeem_script: Option<HexString>,
    /// Witness script
    pub witness_script: Option<HexString>,
    /// BIP32 derivation paths
    pub bip32_derivation: Vec<Bip32Derivation>,
    /// Final scriptsig
    pub final_script_sig: Option<HexString>,
    /// Final scriptwitness
    pub final_script_witness: Option<Vec<HexString>>,
}

impl PsbtInput {
    /// Check if this input is signed.
    pub fn is_signed(&self) -> bool {
        !self.partial_sigs.is_empty()
            || self.final_script_sig.is_some()
            || self.final_script_witness.is_some()
    }
}

/// PSBT output data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PsbtOutput {
    /// Redeem script
    pub redeem_script: Option<HexString>,
    /// Witness script
    pub witness_script: Option<HexString>,
    /// BIP32 derivation paths
    pub bip32_derivation: Vec<Bip32Derivation>,
}

/// BIP32 derivation information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bip32Derivation {
    /// Public key
    pub pubkey: HexString,
    /// Master key fingerprint
    pub master_fingerprint: [u8; 4],
    /// Derivation path
    pub path: Vec<u32>,
}

/// Proprietary data in PSBT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProprietaryData {
    /// Identifier
    pub identifier: Vec<u8>,
    /// Subtype
    pub subtype: u64,
    /// Key
    pub key: Vec<u8>,
    /// Value
    pub value: Vec<u8>,
}

/// A signed Bitcoin transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBitcoinTransaction {
    /// Raw transaction bytes
    pub raw: Vec<u8>,
    /// Transaction ID
    pub txid: Hash32,
    /// Witness transaction ID
    pub wtxid: Hash32,
    /// Size in bytes
    pub size: usize,
    /// Virtual size
    pub vsize: usize,
    /// Weight units
    pub weight: usize,
}

impl SignedBitcoinTransaction {
    /// Get the raw transaction as hex.
    pub fn to_hex(&self) -> HexString {
        HexString::from_bytes(&self.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config() {
        assert_eq!(BitcoinNetwork::Mainnet.bech32_hrp(), "bc");
        assert_eq!(BitcoinNetwork::Testnet.bech32_hrp(), "tb");
        assert_eq!(BitcoinNetwork::Mainnet.p2pkh_version(), 0x00);
        assert_eq!(BitcoinNetwork::Testnet.p2pkh_version(), 0x6f);
    }

    #[test]
    fn test_address_format() {
        assert!(AddressFormat::P2wpkh.is_segwit());
        assert!(AddressFormat::P2wpkh.is_native_segwit());
        assert!(AddressFormat::P2shP2wpkh.is_segwit());
        assert!(!AddressFormat::P2shP2wpkh.is_native_segwit());
        assert!(!AddressFormat::P2pkh.is_segwit());
    }

    #[test]
    fn test_utxo_set() {
        let mut set = UtxoSet::new();
        assert!(set.is_empty());

        let utxo = Utxo {
            txid: Hash32::zero(),
            vout: 0,
            value: 100_000,
            script_pubkey: HexString::from_bytes(&[]),
            address: None,
            block_height: Some(100),
            confirmations: 6,
        };

        set.add(utxo);
        assert_eq!(set.len(), 1);
        assert_eq!(set.balance(), 100_000);
        assert_eq!(set.confirmed_balance(1), 100_000);
        assert_eq!(set.confirmed_balance(10), 0);
    }

    #[test]
    fn test_fee_rate() {
        let rate = FeeRate::from_sat_per_vb(10);
        assert_eq!(rate.sat_per_vb(), 10);
        assert_eq!(rate.fee_for_vsize(100), 1000);
        assert_eq!(rate.fee_for_weight(400), 1000);
    }

    #[test]
    fn test_fee_estimate_target() {
        assert_eq!(FeeEstimateTarget::High.blocks(), 1);
        assert_eq!(FeeEstimateTarget::Medium.blocks(), 3);
        assert_eq!(FeeEstimateTarget::Blocks(10).blocks(), 10);
    }

    #[test]
    fn test_utxo_selection_config() {
        let config = UtxoSelectionConfig::default();
        assert_eq!(config.strategy, UtxoSelectionStrategy::BranchAndBound);
        assert!(!config.allow_unconfirmed);
    }

    #[test]
    fn test_psbt_creation() {
        let tx = UnsignedTransaction {
            version: 2,
            lock_time: 0,
            inputs: vec![TxInput {
                previous_output_txid: Hash32::zero(),
                previous_output_vout: 0,
                sequence: 0xffffffff,
            }],
            outputs: vec![TxOutput {
                value: 50000,
                script_pubkey: HexString::from_bytes(&[0x00, 0x14]),
            }],
        };

        let psbt = Psbt::new(tx);
        assert_eq!(psbt.inputs.len(), 1);
        assert_eq!(psbt.outputs.len(), 1);
        assert!(!psbt.is_fully_signed());
    }
}
