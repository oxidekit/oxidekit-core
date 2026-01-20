//! Ethereum blockchain support.
//!
//! This module provides types for:
//! - Chain configuration (chainId, EIPs)
//! - EIP-1559 fee handling
//! - Transaction building and signing
//! - EIP-712 typed data signing
//! - ABI encoding/decoding
//!
//! Network access is handled via the `rpc` module.

use crate::core::{BigUint, Hash32, HexString};
use crate::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ============================================================================
// Chain Configuration
// ============================================================================

/// Ethereum chain configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Chain ID (EIP-155)
    pub chain_id: u64,
    /// Human-readable name
    pub name: String,
    /// Short name (e.g., "eth", "arb")
    pub short_name: String,
    /// Native currency
    pub native_currency: NativeCurrency,
    /// RPC endpoints
    pub rpc_urls: Vec<String>,
    /// Block explorer URLs
    pub explorers: Vec<ExplorerConfig>,
    /// EIP support
    pub eip_support: EipSupport,
    /// Whether this is a testnet
    pub is_testnet: bool,
}

impl ChainConfig {
    /// Create Ethereum mainnet configuration.
    pub fn ethereum_mainnet() -> Self {
        Self {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            short_name: "eth".to_string(),
            native_currency: NativeCurrency {
                name: "Ether".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            rpc_urls: vec![
                "https://eth.llamarpc.com".to_string(),
                "https://cloudflare-eth.com".to_string(),
            ],
            explorers: vec![ExplorerConfig {
                name: "Etherscan".to_string(),
                url: "https://etherscan.io".to_string(),
                api_url: Some("https://api.etherscan.io/api".to_string()),
            }],
            eip_support: EipSupport::full(),
            is_testnet: false,
        }
    }

    /// Create Ethereum Sepolia testnet configuration.
    pub fn ethereum_sepolia() -> Self {
        Self {
            chain_id: 11155111,
            name: "Sepolia Testnet".to_string(),
            short_name: "sep".to_string(),
            native_currency: NativeCurrency {
                name: "Sepolia Ether".to_string(),
                symbol: "SEP".to_string(),
                decimals: 18,
            },
            rpc_urls: vec!["https://rpc.sepolia.org".to_string()],
            explorers: vec![ExplorerConfig {
                name: "Sepolia Etherscan".to_string(),
                url: "https://sepolia.etherscan.io".to_string(),
                api_url: None,
            }],
            eip_support: EipSupport::full(),
            is_testnet: true,
        }
    }

    /// Create Arbitrum One configuration.
    pub fn arbitrum_one() -> Self {
        Self {
            chain_id: 42161,
            name: "Arbitrum One".to_string(),
            short_name: "arb1".to_string(),
            native_currency: NativeCurrency {
                name: "Ether".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            rpc_urls: vec!["https://arb1.arbitrum.io/rpc".to_string()],
            explorers: vec![ExplorerConfig {
                name: "Arbiscan".to_string(),
                url: "https://arbiscan.io".to_string(),
                api_url: Some("https://api.arbiscan.io/api".to_string()),
            }],
            eip_support: EipSupport::full(),
            is_testnet: false,
        }
    }

    /// Create Optimism configuration.
    pub fn optimism() -> Self {
        Self {
            chain_id: 10,
            name: "Optimism".to_string(),
            short_name: "oeth".to_string(),
            native_currency: NativeCurrency {
                name: "Ether".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            rpc_urls: vec!["https://mainnet.optimism.io".to_string()],
            explorers: vec![ExplorerConfig {
                name: "Optimism Explorer".to_string(),
                url: "https://optimistic.etherscan.io".to_string(),
                api_url: None,
            }],
            eip_support: EipSupport::full(),
            is_testnet: false,
        }
    }

    /// Create Base configuration.
    pub fn base() -> Self {
        Self {
            chain_id: 8453,
            name: "Base".to_string(),
            short_name: "base".to_string(),
            native_currency: NativeCurrency {
                name: "Ether".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            rpc_urls: vec!["https://mainnet.base.org".to_string()],
            explorers: vec![ExplorerConfig {
                name: "BaseScan".to_string(),
                url: "https://basescan.org".to_string(),
                api_url: None,
            }],
            eip_support: EipSupport::full(),
            is_testnet: false,
        }
    }
}

/// Native currency configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeCurrency {
    /// Currency name
    pub name: String,
    /// Currency symbol
    pub symbol: String,
    /// Number of decimals
    pub decimals: u8,
}

/// Block explorer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerConfig {
    /// Explorer name
    pub name: String,
    /// Base URL
    pub url: String,
    /// API URL (if available)
    pub api_url: Option<String>,
}

/// EIP support flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EipSupport {
    /// EIP-155: Replay attack protection
    pub eip155: bool,
    /// EIP-1559: Fee market
    pub eip1559: bool,
    /// EIP-2718: Typed transactions
    pub eip2718: bool,
    /// EIP-2930: Access lists
    pub eip2930: bool,
    /// EIP-4844: Blob transactions
    pub eip4844: bool,
}

impl EipSupport {
    /// Full EIP support (modern Ethereum).
    pub fn full() -> Self {
        Self {
            eip155: true,
            eip1559: true,
            eip2718: true,
            eip2930: true,
            eip4844: true,
        }
    }

    /// Legacy support only.
    pub fn legacy() -> Self {
        Self {
            eip155: true,
            eip1559: false,
            eip2718: false,
            eip2930: false,
            eip4844: false,
        }
    }
}

// ============================================================================
// Addresses
// ============================================================================

/// An Ethereum address (20 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EthereumAddress(#[serde(with = "address_serde")] [u8; 20]);

impl EthereumAddress {
    /// Create from bytes.
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Parse from hex string (with or without 0x prefix).
    pub fn parse(s: &str) -> CryptoResult<Self> {
        let hex = HexString::parse(s)?;
        let bytes = hex.to_bytes()?;

        if bytes.len() != 20 {
            return Err(CryptoError::InvalidAddress(format!(
                "expected 20 bytes, got {}",
                bytes.len()
            )));
        }

        let mut arr = [0u8; 20];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Get as bytes.
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Convert to hex string with checksum (EIP-55).
    pub fn to_checksum_string(&self) -> String {
        // Simple hex for now; real implementation would add EIP-55 checksum
        format!("0x{}", HexString::from_bytes(&self.0))
    }

    /// Zero address.
    pub fn zero() -> Self {
        Self([0u8; 20])
    }

    /// Check if zero address.
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl fmt::Display for EthereumAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_checksum_string())
    }
}

impl FromStr for EthereumAddress {
    type Err = CryptoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

mod address_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 20], s: S) -> Result<S::Ok, S::Error> {
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        s.serialize_str(&format!("0x{}", hex))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 20], D::Error> {
        let s = String::deserialize(d)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);

        if s.len() != 40 {
            return Err(serde::de::Error::custom("expected 40 hex characters"));
        }

        let mut result = [0u8; 20];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let s = std::str::from_utf8(chunk).map_err(serde::de::Error::custom)?;
            result[i] = u8::from_str_radix(s, 16).map_err(serde::de::Error::custom)?;
        }
        Ok(result)
    }
}

// ============================================================================
// EIP-1559 Transactions
// ============================================================================

/// Transaction type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Legacy (pre-EIP-2718)
    Legacy = 0,
    /// EIP-2930 access list
    AccessList = 1,
    /// EIP-1559 dynamic fee
    DynamicFee = 2,
    /// EIP-4844 blob transaction
    Blob = 3,
}

/// EIP-1559 fee configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Eip1559Fees {
    /// Maximum fee per gas (in wei)
    pub max_fee_per_gas: BigUint,
    /// Maximum priority fee per gas (in wei)
    pub max_priority_fee_per_gas: BigUint,
}

impl Eip1559Fees {
    /// Create with values in gwei.
    pub fn from_gwei(max_fee: u64, priority_fee: u64) -> Self {
        Self {
            max_fee_per_gas: BigUint::from_u64(max_fee * 1_000_000_000),
            max_priority_fee_per_gas: BigUint::from_u64(priority_fee * 1_000_000_000),
        }
    }
}

/// An EIP-1559 transaction request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Eip1559Transaction {
    /// Chain ID
    pub chain_id: u64,
    /// Nonce
    pub nonce: u64,
    /// Recipient address (None for contract creation)
    pub to: Option<EthereumAddress>,
    /// Value in wei
    pub value: BigUint,
    /// Transaction data
    pub data: Vec<u8>,
    /// Gas limit
    pub gas_limit: u64,
    /// EIP-1559 fees
    pub fees: Eip1559Fees,
    /// Access list (EIP-2930)
    pub access_list: Vec<AccessListEntry>,
}

impl Eip1559Transaction {
    /// Create a simple ETH transfer.
    pub fn transfer(
        chain_id: u64,
        to: EthereumAddress,
        value: BigUint,
        nonce: u64,
        fees: Eip1559Fees,
    ) -> Self {
        Self {
            chain_id,
            nonce,
            to: Some(to),
            value,
            data: vec![],
            gas_limit: 21000, // Standard ETH transfer
            fees,
            access_list: vec![],
        }
    }

    /// Create a contract call.
    pub fn call(
        chain_id: u64,
        to: EthereumAddress,
        data: Vec<u8>,
        value: BigUint,
        nonce: u64,
        gas_limit: u64,
        fees: Eip1559Fees,
    ) -> Self {
        Self {
            chain_id,
            nonce,
            to: Some(to),
            value,
            data,
            gas_limit,
            fees,
            access_list: vec![],
        }
    }

    /// Get the transaction type.
    pub fn tx_type(&self) -> TransactionType {
        TransactionType::DynamicFee
    }
}

/// Access list entry (EIP-2930).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessListEntry {
    /// Address
    pub address: EthereumAddress,
    /// Storage keys
    pub storage_keys: Vec<Hash32>,
}

/// A signed transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The transaction request
    pub transaction: Eip1559Transaction,
    /// Signature r value
    pub r: BigUint,
    /// Signature s value
    pub s: BigUint,
    /// Recovery ID (0 or 1)
    pub v: u8,
    /// Transaction hash
    pub hash: Hash32,
}

impl SignedTransaction {
    /// Get the sender address (recovered from signature).
    pub fn sender(&self) -> CryptoResult<EthereumAddress> {
        // In a real implementation, this would recover the address from the signature
        Err(CryptoError::FeatureNotAvailable(
            "address recovery not implemented".to_string(),
        ))
    }

    /// Serialize to RLP-encoded bytes.
    pub fn to_rlp(&self) -> Vec<u8> {
        // Placeholder: real implementation would RLP-encode the transaction
        vec![]
    }

    /// Get the transaction hash.
    pub fn tx_hash(&self) -> &Hash32 {
        &self.hash
    }
}

// ============================================================================
// EIP-712 Typed Data
// ============================================================================

/// EIP-712 domain separator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataDomain {
    /// Name of the signing domain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Current version of the signing domain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Chain ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
    /// Verifying contract address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifying_contract: Option<EthereumAddress>,
    /// Salt (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<Hash32>,
}

impl TypedDataDomain {
    /// Create a new domain.
    pub fn new(name: &str) -> Self {
        Self {
            name: Some(name.to_string()),
            version: None,
            chain_id: None,
            verifying_contract: None,
            salt: None,
        }
    }

    /// Set version.
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    /// Set chain ID.
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Set verifying contract.
    pub fn with_contract(mut self, contract: EthereumAddress) -> Self {
        self.verifying_contract = Some(contract);
        self
    }
}

/// An EIP-712 type definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedDataType {
    /// Type name
    pub name: String,
    /// Type definition
    #[serde(rename = "type")]
    pub type_name: String,
}

/// EIP-712 typed data for signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedData {
    /// Type definitions
    pub types: std::collections::HashMap<String, Vec<TypedDataType>>,
    /// Primary type name
    pub primary_type: String,
    /// Domain separator
    pub domain: TypedDataDomain,
    /// Message data
    pub message: serde_json::Value,
}

/// Trait for types that support EIP-712 typed data signing.
pub trait TypedDataSigning {
    /// Get the EIP-712 typed data representation.
    fn to_typed_data(&self, domain: TypedDataDomain) -> TypedData;

    /// Get the type hash.
    fn type_hash() -> Hash32;
}

// ============================================================================
// ABI Encoding
// ============================================================================

/// An ABI type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbiType {
    /// Address
    Address,
    /// Boolean
    Bool,
    /// Unsigned integer (bits must be 8-256, multiple of 8)
    Uint(u16),
    /// Signed integer (bits must be 8-256, multiple of 8)
    Int(u16),
    /// Fixed-size bytes (1-32)
    FixedBytes(u8),
    /// Dynamic bytes
    Bytes,
    /// String
    String,
    /// Fixed-size array
    FixedArray(Box<AbiType>, usize),
    /// Dynamic array
    Array(Box<AbiType>),
    /// Tuple
    Tuple(Vec<AbiType>),
}

impl AbiType {
    /// uint256
    pub fn uint256() -> Self {
        Self::Uint(256)
    }

    /// uint128
    pub fn uint128() -> Self {
        Self::Uint(128)
    }

    /// uint64
    pub fn uint64() -> Self {
        Self::Uint(64)
    }

    /// bytes32
    pub fn bytes32() -> Self {
        Self::FixedBytes(32)
    }

    /// Check if this type is dynamic.
    pub fn is_dynamic(&self) -> bool {
        match self {
            Self::Bytes | Self::String | Self::Array(_) => true,
            Self::FixedArray(inner, _) => inner.is_dynamic(),
            Self::Tuple(types) => types.iter().any(|t| t.is_dynamic()),
            _ => false,
        }
    }
}

impl fmt::Display for AbiType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Address => write!(f, "address"),
            Self::Bool => write!(f, "bool"),
            Self::Uint(bits) => write!(f, "uint{}", bits),
            Self::Int(bits) => write!(f, "int{}", bits),
            Self::FixedBytes(size) => write!(f, "bytes{}", size),
            Self::Bytes => write!(f, "bytes"),
            Self::String => write!(f, "string"),
            Self::FixedArray(inner, size) => write!(f, "{}[{}]", inner, size),
            Self::Array(inner) => write!(f, "{}[]", inner),
            Self::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
        }
    }
}

/// An ABI value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbiValue {
    /// Address
    Address(EthereumAddress),
    /// Boolean
    Bool(bool),
    /// Unsigned integer (256 bits max)
    Uint(BigUint),
    /// Signed integer (256 bits max)
    Int(BigUint, bool), // (magnitude, is_negative)
    /// Fixed-size bytes
    FixedBytes(Vec<u8>),
    /// Dynamic bytes
    Bytes(Vec<u8>),
    /// String
    String(String),
    /// Array
    Array(Vec<AbiValue>),
    /// Tuple
    Tuple(Vec<AbiValue>),
}

/// ABI function definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiFunction {
    /// Function name
    pub name: String,
    /// Input types
    pub inputs: Vec<AbiParam>,
    /// Output types
    pub outputs: Vec<AbiParam>,
    /// State mutability
    pub state_mutability: StateMutability,
}

impl AbiFunction {
    /// Get the function selector (first 4 bytes of keccak256 hash).
    pub fn selector(&self) -> [u8; 4] {
        // In a real implementation, this would compute the selector
        // keccak256("name(type1,type2,...)")[0..4]
        [0, 0, 0, 0]
    }

    /// Encode function call data.
    pub fn encode_call(&self, _args: &[AbiValue]) -> CryptoResult<Vec<u8>> {
        // Placeholder: real implementation would ABI-encode the call
        Ok(vec![])
    }

    /// Decode function return data.
    pub fn decode_output(&self, _data: &[u8]) -> CryptoResult<Vec<AbiValue>> {
        // Placeholder: real implementation would ABI-decode the output
        Ok(vec![])
    }
}

/// ABI function parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiParam {
    /// Parameter name
    pub name: String,
    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: AbiType,
    /// Indexed (for events)
    #[serde(default)]
    pub indexed: bool,
}

/// Function state mutability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StateMutability {
    /// Read-only, no state access
    Pure,
    /// Read-only, accesses state
    View,
    /// Modifies state, no ETH
    NonPayable,
    /// Modifies state, accepts ETH
    Payable,
}

/// ABI event definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiEvent {
    /// Event name
    pub name: String,
    /// Event parameters
    pub inputs: Vec<AbiParam>,
    /// Whether the event is anonymous
    #[serde(default)]
    pub anonymous: bool,
}

impl AbiEvent {
    /// Get the event topic (keccak256 hash of signature).
    pub fn topic(&self) -> Hash32 {
        // In a real implementation, this would compute the topic
        Hash32::zero()
    }
}

/// Full contract ABI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAbi {
    /// Contract name
    pub name: Option<String>,
    /// Functions
    pub functions: Vec<AbiFunction>,
    /// Events
    pub events: Vec<AbiEvent>,
    /// Constructor
    pub constructor: Option<AbiFunction>,
}

impl ContractAbi {
    /// Parse ABI from JSON.
    pub fn from_json(json: &str) -> CryptoResult<Self> {
        serde_json::from_str(json).map_err(|e| {
            CryptoError::DeserializationError(format!("failed to parse ABI: {}", e))
        })
    }

    /// Get a function by name.
    pub fn function(&self, name: &str) -> Option<&AbiFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Get an event by name.
    pub fn event(&self, name: &str) -> Option<&AbiEvent> {
        self.events.iter().find(|e| e.name == name)
    }
}

// ============================================================================
// Transaction Receipt
// ============================================================================

/// Transaction receipt status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction succeeded
    Success,
    /// Transaction reverted
    Reverted,
}

/// A transaction receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    /// Transaction hash
    pub transaction_hash: Hash32,
    /// Transaction index in block
    pub transaction_index: u64,
    /// Block hash
    pub block_hash: Hash32,
    /// Block number
    pub block_number: u64,
    /// Sender address
    pub from: EthereumAddress,
    /// Recipient address
    pub to: Option<EthereumAddress>,
    /// Contract address (if contract creation)
    pub contract_address: Option<EthereumAddress>,
    /// Cumulative gas used
    pub cumulative_gas_used: u64,
    /// Gas used by this transaction
    pub gas_used: u64,
    /// Effective gas price
    pub effective_gas_price: BigUint,
    /// Status
    pub status: TransactionStatus,
    /// Logs
    pub logs: Vec<Log>,
}

/// An event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    /// Contract address
    pub address: EthereumAddress,
    /// Topics
    pub topics: Vec<Hash32>,
    /// Data
    pub data: Vec<u8>,
    /// Block number
    pub block_number: u64,
    /// Transaction hash
    pub transaction_hash: Hash32,
    /// Log index
    pub log_index: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_config_mainnet() {
        let config = ChainConfig::ethereum_mainnet();
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.native_currency.symbol, "ETH");
        assert!(!config.is_testnet);
    }

    #[test]
    fn test_chain_config_sepolia() {
        let config = ChainConfig::ethereum_sepolia();
        assert_eq!(config.chain_id, 11155111);
        assert!(config.is_testnet);
    }

    #[test]
    fn test_ethereum_address_parsing() {
        let addr = EthereumAddress::parse("0xdead000000000000000000000000000000000000").unwrap();
        assert_eq!(addr.as_bytes()[0], 0xde);
        assert_eq!(addr.as_bytes()[1], 0xad);
    }

    #[test]
    fn test_ethereum_address_zero() {
        let addr = EthereumAddress::zero();
        assert!(addr.is_zero());
    }

    #[test]
    fn test_eip1559_fees() {
        let fees = Eip1559Fees::from_gwei(100, 2);
        assert!(!fees.max_fee_per_gas.is_zero());
        assert!(!fees.max_priority_fee_per_gas.is_zero());
    }

    #[test]
    fn test_abi_type_display() {
        assert_eq!(AbiType::Address.to_string(), "address");
        assert_eq!(AbiType::uint256().to_string(), "uint256");
        assert_eq!(AbiType::bytes32().to_string(), "bytes32");
        assert_eq!(AbiType::Array(Box::new(AbiType::Address)).to_string(), "address[]");
    }

    #[test]
    fn test_abi_type_is_dynamic() {
        assert!(!AbiType::Address.is_dynamic());
        assert!(!AbiType::uint256().is_dynamic());
        assert!(AbiType::String.is_dynamic());
        assert!(AbiType::Bytes.is_dynamic());
        assert!(AbiType::Array(Box::new(AbiType::Address)).is_dynamic());
    }

    #[test]
    fn test_typed_data_domain() {
        let domain = TypedDataDomain::new("MyApp")
            .with_version("1")
            .with_chain_id(1);

        assert_eq!(domain.name, Some("MyApp".to_string()));
        assert_eq!(domain.version, Some("1".to_string()));
        assert_eq!(domain.chain_id, Some(1));
    }

    #[test]
    fn test_transaction_transfer() {
        let to = EthereumAddress::parse("0x1234567890123456789012345678901234567890").unwrap();
        let value = BigUint::from_u64(1_000_000_000_000_000_000); // 1 ETH
        let fees = Eip1559Fees::from_gwei(50, 1);

        let tx = Eip1559Transaction::transfer(1, to, value, 0, fees);
        assert_eq!(tx.chain_id, 1);
        assert_eq!(tx.gas_limit, 21000);
        assert!(tx.data.is_empty());
    }
}
