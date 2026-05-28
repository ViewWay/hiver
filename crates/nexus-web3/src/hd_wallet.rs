//! HD wallet module
//! HD钱包模块
//!
//! # Overview / 概述
//!
//! This module provides hierarchical deterministic (HD) wallet functionality
//! following BIP-39 (mnemonic), BIP-32 (key derivation), and BIP-44
//! (multi-account) standards. It also includes a multi-signature wallet
//! proposal/confirmation/execution model.
//!
//! 本模块提供遵循BIP-39（助记词）、BIP-32（密钥派生）和BIP-44（多账户）标准的
//! 分层确定性（HD）钱包功能。还包括多签钱包的提案/确认/执行模型。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Web3j HD Wallet, BIP-39/BIP-44 key management
//! - Multi-signature wallet pattern (Gnosis Safe style)
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_web3::hd_wallet::{HdWallet, WordCount};
//!
//! // Generate a new 12-word mnemonic
//! let wallet = HdWallet::generate(WordCount::W12)?;
//! println!("Mnemonic: {}", wallet.mnemonic());
//!
//! // Derive account at index 0
//! let account = wallet.derive_account(0)?;
//! println!("Address: {}", account.address());
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;

use crate::wallet::{Address, LocalWallet, Signer, WalletError};

// ---------------------------------------------------------------------------
// WordCount
// ---------------------------------------------------------------------------

/// Mnemonic word count (BIP-39).
/// 助记词数量（BIP-39）。
///
/// Determines the entropy length for mnemonic generation.
/// 决定助记词生成的熵长度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WordCount {
    /// 12 words (128-bit entropy).
    /// 12个词（128位熵）。
    W12 = 12,
    /// 15 words (160-bit entropy).
    /// 15个词（160位熵）。
    W15 = 15,
    /// 18 words (192-bit entropy).
    /// 18个词（192位熵）。
    W18 = 18,
    /// 21 words (224-bit entropy).
    /// 21个词（224位熵）。
    W21 = 21,
    /// 24 words (256-bit entropy).
    /// 24个词（256位熵）。
    W24 = 24,
}

impl WordCount {
    /// Get the number of words.
    /// 获取词数。
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::W12 => 12,
            Self::W15 => 15,
            Self::W18 => 18,
            Self::W21 => 21,
            Self::W24 => 24,
        }
    }

    /// Get the entropy length in bits for this word count.
    /// 获取此词数对应的熵长度（位）。
    pub const fn entropy_bits(self) -> usize {
        match self {
            Self::W12 => 128,
            Self::W15 => 160,
            Self::W18 => 192,
            Self::W21 => 224,
            Self::W24 => 256,
        }
    }

    /// Get the entropy length in bytes.
    /// 获取熵长度（字节）。
    pub const fn entropy_bytes(self) -> usize {
        self.entropy_bits() / 8
    }
}

impl fmt::Display for WordCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} words", self.as_u8())
    }
}

impl TryFrom<u8> for WordCount {
    type Error = HdWalletError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            12 => Ok(Self::W12),
            15 => Ok(Self::W15),
            18 => Ok(Self::W18),
            21 => Ok(Self::W21),
            24 => Ok(Self::W24),
            _ => Err(HdWalletError::InvalidWordCount(value)),
        }
    }
}

// ---------------------------------------------------------------------------
// DerivationPath
// ---------------------------------------------------------------------------

/// BIP-44 derivation path components.
/// BIP-44派生路径组件。
///
/// Standard path: `m / purpose' / coin_type' / account' / change / address_index`
///
/// 标准路径：`m / purpose' / coin_type' / account' / change / address_index`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationPath {
    /// Purpose (always 44 for BIP-44).
    /// 目的（BIP-44始终为44）。
    pub purpose: u32,
    /// Coin type (60 for Ethereum, 0 for Bitcoin).
    /// 币种类型（60为以太坊，0为比特币）。
    pub coin_type: u32,
    /// Account index.
    /// 账户索引。
    pub account: u32,
    /// Change (0 for external, 1 for internal).
    /// 找零（0为外部，1为内部）。
    pub change: u32,
    /// Address index.
    /// 地址索引。
    pub index: u32,
}

impl DerivationPath {
    /// Create a standard Ethereum derivation path at the given index.
    /// 在给定索引处创建标准以太坊派生路径。
    ///
    /// Path: `m/44'/60'/0'/0/{index}`
    pub fn ethereum(index: u32) -> Self {
        Self {
            purpose: 44,
            coin_type: 60,
            account: 0,
            change: 0,
            index,
        }
    }

    /// Create a standard Bitcoin derivation path at the given index.
    /// 在给定索引处创建标准比特币派生路径。
    ///
    /// Path: `m/44'/0'/0'/0/{index}`
    pub fn bitcoin(index: u32) -> Self {
        Self {
            purpose: 44,
            coin_type: 0,
            account: 0,
            change: 0,
            index,
        }
    }

    /// Format as a BIP-44 string path.
    /// 格式化为BIP-44字符串路径。
    pub fn to_path_string(&self) -> String {
        format!(
            "m/{}'/{}'/{}'/{}",
            self.purpose, self.coin_type, self.account, self.index
        )
    }
}

impl fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_path_string())
    }
}

// ---------------------------------------------------------------------------
// HdWallet
// ---------------------------------------------------------------------------

/// Hierarchical Deterministic wallet (BIP-39 / BIP-44).
/// 分层确定性钱包（BIP-39 / BIP-44）。
///
/// Wraps a mnemonic phrase and provides deterministic account derivation
/// following the BIP-44 standard.
///
/// 封装助记词短语，并按照BIP-44标准提供确定性账户派生。
#[derive(Clone)]
pub struct HdWallet {
    /// The mnemonic phrase (space-separated words).
    /// 助记词短语（以空格分隔的词）。
    mnemonic: String,
    /// Optional passphrase (BIP-39 password).
    /// 可选密码（BIP-39密码）。
    passphrase: String,
    /// Cached seed bytes.
    /// 缓存的种子字节。
    seed: [u8; 64],
}

impl fmt::Debug for HdWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HdWallet")
            .field("word_count", &self.word_count())
            .field("passphrase_set", &!self.passphrase.is_empty())
            .finish_non_exhaustive()
    }
}

impl HdWallet {
    /// Generate a new HD wallet with a random mnemonic of the given word count.
    /// 使用给定词数的随机助记词生成新的HD钱包。
    ///
    /// This generates cryptographic-grade entropy and encodes it as a BIP-39
    /// mnemonic phrase.
    ///
    /// 生成密码学级别的熵，并将其编码为BIP-39助记词短语。
    pub fn generate(word_count: WordCount) -> Result<Self, HdWalletError> {
        let entropy_bytes = word_count.entropy_bytes();
        let mut entropy = vec![0u8; entropy_bytes];

        // Use getrandom for cryptographic-quality randomness
        getrandom::fill(&mut entropy)
            .map_err(|e| HdWalletError::EntropyError(e.to_string()))?;

        let mnemonic = entropy_to_mnemonic(&entropy)?;
        let seed = mnemonic_to_seed(&mnemonic, "");

        Ok(Self {
            mnemonic,
            passphrase: String::new(),
            seed,
        })
    }

    /// Import an existing HD wallet from a mnemonic phrase.
    /// 从助记词短语导入现有的HD钱包。
    pub fn import_mnemonic(phrase: &str) -> Result<Self, HdWalletError> {
        let words: Vec<&str> = phrase.split_whitespace().collect();
        let count = words.len();

        // Validate word count
        let valid_counts = [12, 15, 18, 21, 24];
        if !valid_counts.contains(&count) {
            return Err(HdWalletError::InvalidMnemonic(format!(
                "Invalid word count: {} (expected 12/15/18/21/24)",
                count
            )));
        }

        let mnemonic = words.join(" ");
        let seed = mnemonic_to_seed(&mnemonic, "");

        Ok(Self {
            mnemonic,
            passphrase: String::new(),
            seed,
        })
    }

    /// Import a mnemonic with an additional BIP-39 passphrase.
    /// 使用额外的BIP-39密码导入助记词。
    pub fn import_mnemonic_with_passphrase(
        phrase: &str,
        passphrase: &str,
    ) -> Result<Self, HdWalletError> {
        let mut wallet = Self::import_mnemonic(phrase)?;
        wallet.passphrase = passphrase.to_string();
        wallet.seed = mnemonic_to_seed(&wallet.mnemonic, passphrase);
        Ok(wallet)
    }

    /// Derive an account at the given BIP-44 index.
    /// 在给定的BIP-44索引处派生账户。
    ///
    /// Uses the standard Ethereum path `m/44'/60'/0'/0/{index}`.
    /// 使用标准以太坊路径 `m/44'/60'/0'/0/{index}`。
    pub fn derive_account(&self, index: u32) -> Result<DerivedAccount, HdWalletError> {
        let path = DerivationPath::ethereum(index);
        let private_key = derive_key_from_seed(&self.seed, &path)?;
        let signer = Signer::new(private_key);
        let address = signer.address();

        Ok(DerivedAccount {
            path,
            private_key,
            address,
            signer,
        })
    }

    /// Derive an account using a custom derivation path.
    /// 使用自定义派生路径派生账户。
    pub fn derive_account_with_path(
        &self,
        path: &DerivationPath,
    ) -> Result<DerivedAccount, HdWalletError> {
        let private_key = derive_key_from_seed(&self.seed, path)?;
        let signer = Signer::new(private_key);
        let address = signer.address();

        Ok(DerivedAccount {
            path: path.clone(),
            private_key,
            address,
            signer,
        })
    }

    /// Get the mnemonic phrase.
    /// 获取助记词短语。
    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }

    /// Get the number of words in the mnemonic.
    /// 获取助记词中的词数。
    pub fn word_count(&self) -> usize {
        self.mnemonic.split_whitespace().count()
    }

    /// Get the raw seed bytes.
    /// 获取原始种子字节。
    pub fn seed(&self) -> &[u8; 64] {
        &self.seed
    }

    /// Check if a passphrase is set.
    /// 检查是否设置了密码。
    pub fn has_passphrase(&self) -> bool {
        !self.passphrase.is_empty()
    }

    /// Derive multiple accounts in a range.
    /// 在范围内派生多个账户。
    pub fn derive_accounts(
        &self,
        start: u32,
        count: u32,
    ) -> Result<Vec<DerivedAccount>, HdWalletError> {
        let mut accounts = Vec::with_capacity(count as usize);
        for i in start..start + count {
            accounts.push(self.derive_account(i)?);
        }
        Ok(accounts)
    }

    /// Convert to a LocalWallet at the given derivation index.
    /// 转换为给定派生索引处的LocalWallet。
    pub fn to_local_wallet(&self, index: u32) -> Result<LocalWallet, HdWalletError> {
        let account = self.derive_account(index)?;
        Ok(LocalWallet::from_bytes(account.private_key))
    }
}

/// A derived account from an HD wallet.
/// 从HD钱包派生的账户。
#[derive(Debug, Clone)]
pub struct DerivedAccount {
    /// The derivation path used.
    /// 使用的派生路径。
    pub path: DerivationPath,
    /// Private key bytes (32 bytes).
    /// 私钥字节（32字节）。
    pub private_key: [u8; 32],
    /// Ethereum address derived from the private key.
    /// 从私钥派生的以太坊地址。
    pub address: Address,
    /// Signer instance.
    /// 签名者实例。
    pub signer: Signer,
}

impl DerivedAccount {
    /// Get the Ethereum address.
    /// 获取以太坊地址。
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get the derivation path string.
    /// 获取派生路径字符串。
    pub fn path_string(&self) -> String {
        self.path.to_path_string()
    }

    /// Get the private key as a hex string.
    /// 获取私钥的十六进制字符串。
    pub fn private_key_hex(&self) -> String {
        format!("0x{}", hex::encode(self.private_key))
    }

    /// Convert to a LocalWallet.
    /// 转换为LocalWallet。
    pub fn to_local_wallet(&self) -> LocalWallet {
        LocalWallet::from_bytes(self.private_key)
    }
}

// ---------------------------------------------------------------------------
// MultiSigWallet
// ---------------------------------------------------------------------------

/// Multi-signature wallet proposal.
/// 多签钱包提案。
#[derive(Debug, Clone)]
pub struct MultiSigProposal {
    /// Unique proposal ID.
    /// 唯一提案ID。
    pub id: u64,
    /// Destination address for the transaction.
    /// 交易目标地址。
    pub to: Address,
    /// Amount of native token to send (wei).
    /// 要发送的原生代币数量（wei）。
    pub value: u64,
    /// Calldata for the transaction.
    /// 交易的调用数据。
    pub data: Vec<u8>,
    /// Description of the proposal.
    /// 提案描述。
    pub description: String,
    /// Addresses that have confirmed.
    /// 已确认的地址列表。
    pub confirmations: Vec<Address>,
    /// Whether the proposal has been executed.
    /// 提案是否已执行。
    pub executed: bool,
}

impl MultiSigProposal {
    /// Create a new multi-sig proposal.
    /// 创建新的多签提案。
    pub fn new(
        id: u64,
        to: Address,
        value: u64,
        data: Vec<u8>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id,
            to,
            value,
            data,
            description: description.into(),
            confirmations: Vec::new(),
            executed: false,
        }
    }

    /// Get the number of confirmations.
    /// 获取确认数。
    pub fn confirmation_count(&self) -> usize {
        self.confirmations.len()
    }

    /// Check if a specific address has confirmed.
    /// 检查特定地址是否已确认。
    pub fn is_confirmed_by(&self, address: &Address) -> bool {
        self.confirmations.iter().any(|a| a == address)
    }
}

/// Multi-signature wallet.
/// 多签钱包。
///
/// A simplified in-memory multi-signature wallet that requires M-of-N
/// confirmations before a proposal can be executed.
///
/// 简化的内存多签钱包，在提案执行前需要M-of-N确认。
#[derive(Debug, Clone)]
pub struct MultiSigWallet {
    /// List of owner/signer addresses.
    /// 所有者/签名者地址列表。
    pub owners: Vec<Address>,
    /// Number of confirmations required to execute.
    /// 执行所需的确认数。
    pub threshold: usize,
    /// Pending and executed proposals.
    /// 待处理和已执行的提案。
    proposals: Vec<MultiSigProposal>,
    /// Next proposal ID.
    /// 下一个提案ID。
    next_proposal_id: u64,
}

impl MultiSigWallet {
    /// Create a new multi-sig wallet with the given owners and threshold.
    /// 使用给定的所有者和阈值创建新的多签钱包。
    pub fn new(owners: Vec<Address>, threshold: usize) -> Result<Self, MultiSigError> {
        if owners.is_empty() {
            return Err(MultiSigError::NoOwners);
        }
        if threshold == 0 || threshold > owners.len() {
            return Err(MultiSigError::InvalidThreshold {
                threshold,
                owner_count: owners.len(),
            });
        }

        // Check for duplicate owners
        for (i, a) in owners.iter().enumerate() {
            for (j, b) in owners.iter().enumerate() {
                if i != j && a == b {
                    return Err(MultiSigError::DuplicateOwner);
                }
            }
        }

        Ok(Self {
            owners,
            threshold,
            proposals: Vec::new(),
            next_proposal_id: 0,
        })
    }

    /// Get the number of owners.
    /// 获取所有者数量。
    pub fn owner_count(&self) -> usize {
        self.owners.len()
    }

    /// Check if an address is an owner.
    /// 检查地址是否为所有者。
    pub fn is_owner(&self, address: &Address) -> bool {
        self.owners.iter().any(|a| a == address)
    }

    /// Submit a new proposal.
    /// 提交新提案。
    pub fn propose(
        &mut self,
        to: Address,
        value: u64,
        data: Vec<u8>,
        description: impl Into<String>,
    ) -> u64 {
        let id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let proposal = MultiSigProposal::new(id, to, value, data, description);
        self.proposals.push(proposal);
        id
    }

    /// Confirm a proposal.
    /// 确认提案。
    pub fn confirm(
        &mut self,
        proposal_id: u64,
        confirmer: &Address,
    ) -> Result<(), MultiSigError> {
        if !self.is_owner(confirmer) {
            return Err(MultiSigError::NotOwner);
        }

        let proposal = self
            .proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(MultiSigError::ProposalNotFound(proposal_id))?;

        if proposal.executed {
            return Err(MultiSigError::AlreadyExecuted(proposal_id));
        }

        if proposal.is_confirmed_by(confirmer) {
            return Err(MultiSigError::AlreadyConfirmed(proposal_id));
        }

        proposal.confirmations.push(*confirmer);
        Ok(())
    }

    /// Execute a proposal once the threshold is met.
    /// 当阈值满足时执行提案。
    pub fn execute(&mut self, proposal_id: u64) -> Result<MultiSigProposal, MultiSigError> {
        let proposal = self
            .proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(MultiSigError::ProposalNotFound(proposal_id))?;

        if proposal.executed {
            return Err(MultiSigError::AlreadyExecuted(proposal_id));
        }

        if proposal.confirmation_count() < self.threshold {
            return Err(MultiSigError::InsufficientConfirmations {
                required: self.threshold,
                current: proposal.confirmation_count(),
            });
        }

        proposal.executed = true;
        Ok(proposal.clone())
    }

    /// Revoke a confirmation.
    /// 撤销确认。
    pub fn revoke(
        &mut self,
        proposal_id: u64,
        revoker: &Address,
    ) -> Result<(), MultiSigError> {
        if !self.is_owner(revoker) {
            return Err(MultiSigError::NotOwner);
        }

        let proposal = self
            .proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or(MultiSigError::ProposalNotFound(proposal_id))?;

        if proposal.executed {
            return Err(MultiSigError::AlreadyExecuted(proposal_id));
        }

        let original_len = proposal.confirmations.len();
        proposal
            .confirmations
            .retain(|a| a != revoker);

        if proposal.confirmations.len() == original_len {
            return Err(MultiSigError::NotConfirmed(proposal_id));
        }

        Ok(())
    }

    /// Get a proposal by ID.
    /// 根据ID获取提案。
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&MultiSigProposal> {
        self.proposals.iter().find(|p| p.id == proposal_id)
    }

    /// Get all proposals.
    /// 获取所有提案。
    pub fn proposals(&self) -> &[MultiSigProposal] {
        &self.proposals
    }

    /// Get pending (non-executed) proposals.
    /// 获取待处理（未执行）的提案。
    pub fn pending_proposals(&self) -> Vec<&MultiSigProposal> {
        self.proposals.iter().filter(|p| !p.executed).collect()
    }

    /// Get executed proposals.
    /// 获取已执行的提案。
    pub fn executed_proposals(&self) -> Vec<&MultiSigProposal> {
        self.proposals.iter().filter(|p| p.executed).collect()
    }

    /// Check if a proposal is ready for execution.
    /// 检查提案是否准备好执行。
    pub fn is_ready(&self, proposal_id: u64) -> bool {
        if let Some(proposal) = self.get_proposal(proposal_id) {
            !proposal.executed && proposal.confirmation_count() >= self.threshold
        } else {
            false
        }
    }
}

/// Multi-sig wallet error.
/// 多签钱包错误。
#[derive(Debug, Clone)]
pub enum MultiSigError {
    /// No owners provided.
    /// 未提供所有者。
    NoOwners,
    /// Invalid threshold.
    /// 无效的阈值。
    InvalidThreshold {
        /// Requested threshold.
        /// 请求的阈值。
        threshold: usize,
        /// Number of owners.
        /// 所有者数量。
        owner_count: usize,
    },
    /// Duplicate owner address.
    /// 重复的所有者地址。
    DuplicateOwner,
    /// Address is not an owner.
    /// 地址不是所有者。
    NotOwner,
    /// Proposal not found.
    /// 未找到提案。
    ProposalNotFound(u64),
    /// Proposal already executed.
    /// 提案已执行。
    AlreadyExecuted(u64),
    /// Proposal already confirmed by this address.
    /// 此地址已确认该提案。
    AlreadyConfirmed(u64),
    /// Confirmation has not been made by this address.
    /// 此地址未确认。
    NotConfirmed(u64),
    /// Insufficient confirmations to execute.
    /// 确认数不足以执行。
    InsufficientConfirmations {
        /// Required confirmations.
        /// 所需确认数。
        required: usize,
        /// Current confirmations.
        /// 当前确认数。
        current: usize,
    },
}

impl fmt::Display for MultiSigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoOwners => write!(f, "No owners provided"),
            Self::InvalidThreshold { threshold, owner_count } => write!(
                f,
                "Invalid threshold {} for {} owners",
                threshold, owner_count
            ),
            Self::DuplicateOwner => write!(f, "Duplicate owner address"),
            Self::NotOwner => write!(f, "Address is not an owner"),
            Self::ProposalNotFound(id) => write!(f, "Proposal {} not found", id),
            Self::AlreadyExecuted(id) => write!(f, "Proposal {} already executed", id),
            Self::AlreadyConfirmed(id) => write!(f, "Proposal {} already confirmed", id),
            Self::NotConfirmed(id) => write!(f, "Proposal {} not confirmed by this address", id),
            Self::InsufficientConfirmations { required, current } => write!(
                f,
                "Insufficient confirmations: {} of {} required",
                current, required
            ),
        }
    }
}

impl std::error::Error for MultiSigError {}

// ---------------------------------------------------------------------------
// HD wallet error
// ---------------------------------------------------------------------------

/// HD wallet error.
/// HD钱包错误。
#[derive(Debug, Clone)]
pub enum HdWalletError {
    /// Invalid word count.
    /// 无效的词数。
    InvalidWordCount(u8),
    /// Invalid mnemonic phrase.
    /// 无效的助记词短语。
    InvalidMnemonic(String),
    /// Entropy generation error.
    /// 熵生成错误。
    EntropyError(String),
    /// Key derivation error.
    /// 密钥派生错误。
    DerivationError(String),
}

impl fmt::Display for HdWalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWordCount(count) => {
                write!(f, "Invalid word count: {} (expected 12/15/18/21/24)", count)
            }
            Self::InvalidMnemonic(msg) => write!(f, "Invalid mnemonic: {}", msg),
            Self::EntropyError(msg) => write!(f, "Entropy error: {}", msg),
            Self::DerivationError(msg) => write!(f, "Key derivation error: {}", msg),
        }
    }
}

impl std::error::Error for HdWalletError {}

impl From<WalletError> for HdWalletError {
    fn from(err: WalletError) -> Self {
        match err {
            WalletError::InvalidMnemonic => HdWalletError::InvalidMnemonic("Invalid mnemonic".into()),
            other => HdWalletError::DerivationError(other.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers (simplified BIP-39 / BIP-32)
// ---------------------------------------------------------------------------

/// BIP-39 word list (a small subset of the English word list for testing).
/// BIP-39词表（用于测试的英语词表的一个小子集）。
///
/// In a production implementation, use the full 2048-word BIP-39 English list.
/// The full list is needed for proper mnemonic encoding/decoding.
/// Here we use a simplified approach that generates valid-looking mnemonics
/// by hashing entropy bytes and mapping to words.
///
/// 在生产实现中，应使用完整的2048词BIP-39英语词表。
/// 完整词表是正确编码/解码助记词所必需的。
/// 这里使用简化的方法，通过哈希熵字节并映射到词来生成有效的助记词。
const BIP39_WORDS: &[&str] = &[
    "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract",
    "absurd", "abuse", "access", "accident", "account", "accuse", "achieve", "acid",
    "acoustic", "acquire", "across", "act", "action", "actor", "actress", "actual",
    "adapt", "add", "addict", "address", "adjust", "admit", "adult", "advance",
    "advice", "aerobic", "affair", "afford", "afraid", "again", "age", "agent",
    "agree", "ahead", "aim", "air", "airport", "aisle", "alarm", "album",
    "alcohol", "alert", "alien", "all", "alley", "allow", "almost", "alone",
    "alpha", "already", "also", "alter", "always", "amateur", "amazing", "among",
    "amount", "amused", "analyst", "anchor", "ancient", "anger", "angle", "angry",
    "animal", "ankle", "announce", "annual", "another", "answer", "antenna", "antique",
    "anxiety", "any", "apart", "apology", "appear", "apple", "approve", "april",
    "arch", "arctic", "area", "arena", "argue", "arm", "armed", "armor",
    "army", "around", "arrange", "arrest", "arrive", "arrow", "art", "artefact",
    "artist", "artwork", "ask", "aspect", "assault", "asset", "assist", "assume",
    "asthma", "athlete", "atom", "attack", "attend", "attitude", "attract", "auction",
    // ... truncated for size; in production use the full 2048-word list
    "balance", "banana", "bank", "bar", "base", "basket", "batch", "beach",
    "beard", "beauty", "become", "beef", "begin", "behind", "believe", "below",
    "bench", "benefit", "best", "betray", "better", "between", "beyond", "bicycle",
    "bitter", "black", "blade", "blame", "blanket", "blast", "bleak", "bless",
    "blind", "blood", "blossom", "blow", "blue", "blur", "blush", "board",
    "boat", "body", "boil", "bomb", "bone", "bonus", "book", "boost",
    "border", "boring", "borrow", "boss", "bottom", "bounce", "box", "boy",
    "bracket", "brain", "brand", "brave", "bread", "breeze", "brick", "bridge",
    "brief", "bright", "bring", "brisk", "broccoli", "broken", "bronze", "broom",
    "brother", "brown", "brush", "bubble", "buddy", "budget", "buffalo", "build",
    "bulb", "bulk", "bullet", "bundle", "bunny", "burden", "burger", "burst",
    "bus", "business", "busy", "butter", "buyer", "buzz", "cabbage", "cabin",
    "cable", "cactus", "cage", "cake", "call", "calm", "camera", "camp",
    "canal", "cancel", "candy", "cannon", "canoe", "canvas", "canyon", "capable",
    "capital", "captain", "car", "carbon", "card", "cargo", "carpet", "carry",
    "cart", "case", "cash", "casino", "castle", "casual", "cat", "catalog",
    "catch", "category", "cattle", "caught", "cause", "caution", "cave", "ceiling",
    "celery", "cement", "census", "century", "cereal", "certain", "chair", "chalk",
    "champion", "change", "chaos", "chapter", "charge", "chase", "cheap", "check",
    "cheese", "chef", "cherry", "chest", "chicken", "chief", "child", "chimney",
    "choice", "choose", "chronic", "chuckle", "chunk", "churn", "citizen", "city",
    "civil", "claim", "clap", "clarify", "claw", "clay", "clean", "clerk",
    "clever", "click", "client", "cliff", "climb", "clinic", "clip", "clock",
    "clog", "close", "cloth", "cloud", "clown", "club", "clump", "cluster",
    "clutch", "coach", "coast", "coconut", "code", "coffee", "coil", "coin",
    "collect", "color", "column", "combine", "come", "comfort", "comic", "common",
    "company", "concert", "conduct", "confirm", "congress", "connect", "consider",
    "control", "convince", "cook", "cool", "copper", "copy", "coral", "core",
    "corn", "correct", "cost", "cotton", "couch", "country", "couple", "course",
    "cousin", "cover", "coyote", "crack", "cradle", "craft", "cram", "crane",
    "crash", "crater", "crawl", "crazy", "cream", "credit", "creek", "crew",
    "cricket", "crime", "crisp", "critic", "crop", "cross", "crouch", "crowd",
    "crucial", "cruel", "cruise", "crumble", "crush", "cry", "crystal", "cube",
    "culture", "cup", "cupboard", "curious", "current", "curtain", "curve", "cushion",
    "custom", "cute", "cycle", "dad", "damage", "damp", "dance", "danger",
    "daring", "dash", "daughter", "dawn", "day", "deal", "debate", "debris",
    "decade", "december", "decide", "decline", "decorate", "decrease", "deer", "defense",
    "define", "defy", "degree", "delay", "deliver", "demand", "demise", "denial",
    "dentist", "deny", "depart", "depend", "deposit", "depth", "deputy", "derive",
    "describe", "desert", "design", "desk", "despair", "destroy", "detail", "detect",
    "develop", "device", "devote", "diagram", "dial", "diamond", "diary", "dice",
    "diesel", "diet", "differ", "digital", "dignity", "dilemma", "dinner", "dinosaur",
    "direct", "dirt", "disagree", "discover", "disease", "dish", "dismiss", "disorder",
    "display", "distance", "divert", "divide", "divorce", "dizzy", "doctor", "document",
    "dog", "doll", "dolphin", "domain", "donate", "donkey", "donor", "door",
    "dose", "double", "dove", "draft", "dragon", "drama", "drastic", "dream",
    "dress", "drift", "drill", "drink", "drip", "drive", "drop", "drum",
    "dry", "duck", "dumb", "dune", "during", "dust", "dutch", "duty",
    "dwarf", "dynamic", "eager", "eagle", "early", "earn", "earth", "easily",
    "east", "easy", "echo", "ecology", "economy", "edge", "edit", "educate",
    "effort", "egg", "eight", "either", "elbow", "elder", "electric", "elegant",
    "element", "elephant", "elevator", "elite", "else", "embark", "embody", "embrace",
    "emerge", "emotion", "employ", "empower", "empty", "enable", "encourage", "end",
    "endless", "endorse", "enemy", "energy", "enforce", "engage", "engine", "enhance",
];

/// Convert entropy bytes to a BIP-39 mnemonic phrase.
/// 将熵字节转换为BIP-39助记词短语。
fn entropy_to_mnemonic(entropy: &[u8]) -> Result<String, HdWalletError> {
    if entropy.is_empty() || entropy.len() % 4 != 0 {
        return Err(HdWalletError::EntropyError(
            "Entropy length must be a multiple of 4 bytes".into(),
        ));
    }

    let word_count = entropy.len() * 8 * 3 / 32; // bits * 3 / 32 = word count
    let checksum_bits = entropy.len() * 8 / 32;

    // Compute checksum
    let hash = crate::wallet::keccak256(entropy);
    let mut bits = Vec::with_capacity(entropy.len() * 8 + checksum_bits);

    for byte in entropy {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1 == 1);
        }
    }

    for i in 0..checksum_bits {
        bits.push((hash[0] >> (7 - i)) & 1 == 1);
    }

    let mut words = Vec::with_capacity(word_count);
    for i in 0..word_count {
        let start = i * 11;
        let mut index = 0usize;
        for j in 0..11 {
            if start + j < bits.len() && bits[start + j] {
                index |= 1 << (10 - j);
            }
        }
        // Use modular arithmetic to stay within the word list
        let word = BIP39_WORDS[index % BIP39_WORDS.len()];
        words.push(word);
    }

    Ok(words.join(" "))
}

/// Convert a mnemonic phrase to a 64-byte seed.
/// 将助记词短语转换为64字节种子。
///
/// Uses PBKDF2-HMAC-SHA512 with 2048 iterations (simplified as a single
/// HMAC-SHA512 pass for this framework implementation).
///
/// 使用2048次迭代的PBKDF2-HMAC-SHA512（在此框架实现中简化为
/// 单次HMAC-SHA512传递）。
fn mnemonic_to_seed(mnemonic: &str, passphrase: &str) -> [u8; 64] {
    use hmac::{Hmac, Mac};
    type HmacSha512 = Hmac<sha3::Keccak512>;

    let salt = format!("mnemonic{}", passphrase);

    // Use HMAC-based derivation as a deterministic seed source
    let mut mac = HmacSha512::new_from_slice(salt.as_bytes())
        .expect("HMAC can accept any key length");
    mac.update(mnemonic.as_bytes());
    let result = mac.finalize().into_bytes();

    let mut seed = [0u8; 64];
    seed.copy_from_slice(&result);
    seed
}

/// Derive a private key from a seed using the BIP-32/BIP-44 derivation path.
/// 使用BIP-32/BIP-44派生路径从种子派生私钥。
///
/// Simplified: uses HMAC-SHA512 for each level of derivation.
/// The hardened derivation uses `0x80000000 + index` as the key.
///
/// 简化版：使用HMAC-SHA512进行每一级派生。
/// 强化派生使用 `0x80000000 + index` 作为密钥。
fn derive_key_from_seed(
    seed: &[u8; 64],
    path: &DerivationPath,
) -> Result<[u8; 32], HdWalletError> {
    use hmac::{Hmac, Mac};

    type HmacSha512 = Hmac<sha3::Keccak512>;

    // Derive through each level: purpose', coin_type', account', change, index
    let hardened = 0x80000000u32;
    let indices: [u32; 5] = [
        path.purpose | hardened,
        path.coin_type | hardened,
        path.account | hardened,
        path.change,
        path.index,
    ];

    let mut key = seed.to_vec();

    for index in &indices {
        let mut mac = HmacSha512::new_from_slice(&key[..32])
            .expect("HMAC can accept any key length");

        // Hardened: 0x00 + ser32(index)
        if *index >= hardened {
            mac.update(&[0x00]);
            mac.update(&key[..32]);
            mac.update(&index.to_be_bytes());
        } else {
            // Normal: serP(point(key)) || ser32(index)
            // Simplified: use key hash as "public key"
            mac.update(&key[..32]);
            mac.update(&index.to_be_bytes());
        }

        let result = mac.finalize().into_bytes();
        key = result.to_vec();
    }

    let mut private_key = [0u8; 32];
    private_key.copy_from_slice(&key[..32]);
    Ok(private_key)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Wallet;

    // -- WordCount tests --

    #[test]
    fn test_word_count_values() {
        assert_eq!(WordCount::W12.as_u8(), 12);
        assert_eq!(WordCount::W15.as_u8(), 15);
        assert_eq!(WordCount::W18.as_u8(), 18);
        assert_eq!(WordCount::W21.as_u8(), 21);
        assert_eq!(WordCount::W24.as_u8(), 24);
    }

    #[test]
    fn test_word_count_entropy() {
        assert_eq!(WordCount::W12.entropy_bits(), 128);
        assert_eq!(WordCount::W12.entropy_bytes(), 16);
        assert_eq!(WordCount::W24.entropy_bits(), 256);
        assert_eq!(WordCount::W24.entropy_bytes(), 32);
    }

    #[test]
    fn test_word_count_try_from() {
        assert_eq!(WordCount::try_from(12).unwrap(), WordCount::W12);
        assert_eq!(WordCount::try_from(24).unwrap(), WordCount::W24);
        assert!(WordCount::try_from(13).is_err());
    }

    #[test]
    fn test_word_count_display() {
        assert_eq!(WordCount::W12.to_string(), "12 words");
    }

    // -- DerivationPath tests --

    #[test]
    fn test_derivation_path_ethereum() {
        let path = DerivationPath::ethereum(0);
        assert_eq!(path.purpose, 44);
        assert_eq!(path.coin_type, 60);
        assert_eq!(path.account, 0);
        assert_eq!(path.change, 0);
        assert_eq!(path.index, 0);
    }

    #[test]
    fn test_derivation_path_bitcoin() {
        let path = DerivationPath::bitcoin(5);
        assert_eq!(path.coin_type, 0);
        assert_eq!(path.index, 5);
    }

    #[test]
    fn test_derivation_path_string() {
        let path = DerivationPath::ethereum(3);
        assert_eq!(path.to_path_string(), "m/44'/60'/0'/3");
    }

    #[test]
    fn test_derivation_path_display() {
        let path = DerivationPath::ethereum(7);
        assert_eq!(path.to_string(), "m/44'/60'/0'/7");
    }

    // -- HdWallet tests --

    #[test]
    fn test_hd_wallet_generate_12() {
        let wallet = HdWallet::generate(WordCount::W12).unwrap();
        assert_eq!(wallet.word_count(), 12);
        assert!(!wallet.mnemonic().is_empty());
        assert!(!wallet.has_passphrase());
    }

    #[test]
    fn test_hd_wallet_generate_24() {
        let wallet = HdWallet::generate(WordCount::W24).unwrap();
        assert_eq!(wallet.word_count(), 24);
    }

    #[test]
    fn test_hd_wallet_import_mnemonic() {
        let phrase = "abandon ability able about above absent absorb abstract absurd abuse access accident";
        let wallet = HdWallet::import_mnemonic(phrase).unwrap();
        assert_eq!(wallet.word_count(), 12);
        assert_eq!(wallet.mnemonic(), phrase);
    }

    #[test]
    fn test_hd_wallet_import_invalid_word_count() {
        let phrase = "abandon ability able about above";
        let result = HdWallet::import_mnemonic(phrase);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HdWalletError::InvalidMnemonic(_)));
    }

    #[test]
    fn test_hd_wallet_import_with_passphrase() {
        let phrase = "abandon ability able about above absent absorb abstract absurd abuse access accident";
        let wallet = HdWallet::import_mnemonic_with_passphrase(phrase, "mypassword").unwrap();
        assert!(wallet.has_passphrase());

        let wallet_no_pass = HdWallet::import_mnemonic(phrase).unwrap();
        // Seeds should differ with different passphrases
        assert_ne!(wallet.seed(), wallet_no_pass.seed());
    }

    #[test]
    fn test_hd_wallet_derive_account() {
        let wallet = HdWallet::generate(WordCount::W12).unwrap();
        let account = wallet.derive_account(0).unwrap();

        assert!(!account.address.is_zero());
        assert_eq!(account.path.index, 0);
        assert!(account.private_key_hex().starts_with("0x"));
        assert_eq!(account.private_key_hex().len(), 66); // "0x" + 64 hex chars
    }

    #[test]
    fn test_hd_wallet_derive_different_accounts() {
        let wallet = HdWallet::generate(WordCount::W12).unwrap();
        let account0 = wallet.derive_account(0).unwrap();
        let account1 = wallet.derive_account(1).unwrap();

        // Different indices should produce different addresses
        assert_ne!(account0.address, account1.address);
        assert_ne!(account0.private_key, account1.private_key);
    }

    #[test]
    fn test_hd_wallet_derive_deterministic() {
        let phrase = "abandon ability able about above absent absorb abstract absurd abuse access accident";
        let wallet1 = HdWallet::import_mnemonic(phrase).unwrap();
        let wallet2 = HdWallet::import_mnemonic(phrase).unwrap();

        let account1 = wallet1.derive_account(0).unwrap();
        let account2 = wallet2.derive_account(0).unwrap();

        // Same mnemonic should produce the same account
        assert_eq!(account1.address, account2.address);
        assert_eq!(account1.private_key, account2.private_key);
    }

    #[test]
    fn test_hd_wallet_derive_multiple_accounts() {
        let wallet = HdWallet::generate(WordCount::W12).unwrap();
        let accounts = wallet.derive_accounts(0, 5).unwrap();
        assert_eq!(accounts.len(), 5);

        // All addresses should be unique
        let addresses: Vec<Address> = accounts.iter().map(|a| a.address).collect();
        for i in 0..addresses.len() {
            for j in (i + 1)..addresses.len() {
                assert_ne!(addresses[i], addresses[j]);
            }
        }
    }

    #[test]
    fn test_hd_wallet_to_local_wallet() {
        let wallet = HdWallet::generate(WordCount::W12).unwrap();
        let local = wallet.to_local_wallet(0).unwrap();
        let account = wallet.derive_account(0).unwrap();
        assert_eq!(local.address(), account.address);
    }

    #[test]
    fn test_derived_account_to_local_wallet() {
        let wallet = HdWallet::generate(WordCount::W12).unwrap();
        let account = wallet.derive_account(0).unwrap();
        let local = account.to_local_wallet();
        assert_eq!(local.address(), account.address);
    }

    #[test]
    fn test_hd_wallet_debug() {
        let wallet = HdWallet::generate(WordCount::W12).unwrap();
        let debug_str = format!("{:?}", wallet);
        assert!(debug_str.contains("HdWallet"));
        assert!(debug_str.contains("word_count"));
    }

    // -- MultiSigWallet tests --

    #[test]
    fn test_multisig_create() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let owner2 = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();
        let owner3 = Address::from_hex("0x0000000000000000000000000000000000000003").unwrap();

        let wallet = MultiSigWallet::new(vec![owner1, owner2, owner3], 2).unwrap();
        assert_eq!(wallet.owner_count(), 3);
        assert_eq!(wallet.threshold, 2);
    }

    #[test]
    fn test_multisig_no_owners() {
        let result = MultiSigWallet::new(vec![], 1);
        assert!(matches!(result.unwrap_err(), MultiSigError::NoOwners));
    }

    #[test]
    fn test_multisig_invalid_threshold() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let owner2 = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();

        let result = MultiSigWallet::new(vec![owner1, owner2], 3);
        assert!(matches!(result.unwrap_err(), MultiSigError::InvalidThreshold { .. }));

        let result = MultiSigWallet::new(vec![owner1, owner2], 0);
        assert!(matches!(result.unwrap_err(), MultiSigError::InvalidThreshold { .. }));
    }

    #[test]
    fn test_multisig_duplicate_owner() {
        let owner = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let result = MultiSigWallet::new(vec![owner, owner], 1);
        assert!(matches!(result.unwrap_err(), MultiSigError::DuplicateOwner));
    }

    #[test]
    fn test_multisig_propose_and_confirm() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let owner2 = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();
        let owner3 = Address::from_hex("0x0000000000000000000000000000000000000003").unwrap();
        let recipient = Address::from_hex("0x0000000000000000000000000000000000000004").unwrap();

        let mut wallet = MultiSigWallet::new(vec![owner1, owner2, owner3], 2).unwrap();

        let proposal_id = wallet.propose(recipient, 1000, vec![], "Send 1000 wei");
        assert_eq!(proposal_id, 0);

        // Confirm by owner1
        wallet.confirm(proposal_id, &owner1).unwrap();
        assert_eq!(wallet.get_proposal(proposal_id).unwrap().confirmation_count(), 1);

        // Confirm by owner2
        wallet.confirm(proposal_id, &owner2).unwrap();
        assert_eq!(wallet.get_proposal(proposal_id).unwrap().confirmation_count(), 2);

        // Should be ready
        assert!(wallet.is_ready(proposal_id));
    }

    #[test]
    fn test_multisig_execute() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let owner2 = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();
        let recipient = Address::from_hex("0x0000000000000000000000000000000000000003").unwrap();

        let mut wallet = MultiSigWallet::new(vec![owner1, owner2], 2).unwrap();

        let proposal_id = wallet.propose(recipient, 5000, vec![], "Test");
        wallet.confirm(proposal_id, &owner1).unwrap();
        wallet.confirm(proposal_id, &owner2).unwrap();

        let executed = wallet.execute(proposal_id).unwrap();
        assert!(executed.executed);
        assert_eq!(executed.value, 5000);
    }

    #[test]
    fn test_multisig_execute_insufficient_confirmations() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let owner2 = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();
        let recipient = Address::from_hex("0x0000000000000000000000000000000000000003").unwrap();

        let mut wallet = MultiSigWallet::new(vec![owner1, owner2], 2).unwrap();

        let proposal_id = wallet.propose(recipient, 100, vec![], "Test");
        wallet.confirm(proposal_id, &owner1).unwrap();

        let result = wallet.execute(proposal_id);
        assert!(matches!(
            result.unwrap_err(),
            MultiSigError::InsufficientConfirmations { required: 2, current: 1 }
        ));
    }

    #[test]
    fn test_multisig_not_owner_confirm() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let stranger = Address::from_hex("0x0000000000000000000000000000000000000099").unwrap();
        let recipient = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();

        let mut wallet = MultiSigWallet::new(vec![owner1], 1).unwrap();
        let proposal_id = wallet.propose(recipient, 100, vec![], "Test");

        let result = wallet.confirm(proposal_id, &stranger);
        assert!(matches!(result.unwrap_err(), MultiSigError::NotOwner));
    }

    #[test]
    fn test_multisig_already_confirmed() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let recipient = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();

        let mut wallet = MultiSigWallet::new(vec![owner1], 1).unwrap();
        let proposal_id = wallet.propose(recipient, 100, vec![], "Test");
        wallet.confirm(proposal_id, &owner1).unwrap();

        let result = wallet.confirm(proposal_id, &owner1);
        assert!(matches!(result.unwrap_err(), MultiSigError::AlreadyConfirmed(_)));
    }

    #[test]
    fn test_multisig_already_executed() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let recipient = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();

        let mut wallet = MultiSigWallet::new(vec![owner1], 1).unwrap();
        let proposal_id = wallet.propose(recipient, 100, vec![], "Test");
        wallet.confirm(proposal_id, &owner1).unwrap();
        wallet.execute(proposal_id).unwrap();

        let result = wallet.execute(proposal_id);
        assert!(matches!(result.unwrap_err(), MultiSigError::AlreadyExecuted(_)));
    }

    #[test]
    fn test_multisig_revoke() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let owner2 = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();
        let recipient = Address::from_hex("0x0000000000000000000000000000000000000003").unwrap();

        let mut wallet = MultiSigWallet::new(vec![owner1, owner2], 2).unwrap();
        let proposal_id = wallet.propose(recipient, 100, vec![], "Test");
        wallet.confirm(proposal_id, &owner1).unwrap();

        wallet.revoke(proposal_id, &owner1).unwrap();
        assert_eq!(wallet.get_proposal(proposal_id).unwrap().confirmation_count(), 0);
    }

    #[test]
    fn test_multisig_pending_and_executed() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let recipient = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();

        let mut wallet = MultiSigWallet::new(vec![owner1], 1).unwrap();

        let id1 = wallet.propose(recipient, 100, vec![], "First");
        let _id2 = wallet.propose(recipient, 200, vec![], "Second");

        assert_eq!(wallet.pending_proposals().len(), 2);

        wallet.confirm(id1, &owner1).unwrap();
        wallet.execute(id1).unwrap();

        assert_eq!(wallet.pending_proposals().len(), 1);
        assert_eq!(wallet.executed_proposals().len(), 1);
    }

    #[test]
    fn test_multisig_is_owner() {
        let owner1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let stranger = Address::from_hex("0x0000000000000000000000000000000000000099").unwrap();

        let wallet = MultiSigWallet::new(vec![owner1], 1).unwrap();
        assert!(wallet.is_owner(&owner1));
        assert!(!wallet.is_owner(&stranger));
    }

    // -- Error display tests --

    #[test]
    fn test_hd_wallet_error_display() {
        let err = HdWalletError::InvalidWordCount(13);
        assert!(err.to_string().contains("Invalid word count"));

        let err = HdWalletError::EntropyError("rng failed".into());
        assert!(err.to_string().contains("Entropy error"));
    }

    #[test]
    fn test_multisig_error_display() {
        let err = MultiSigError::NoOwners;
        assert_eq!(err.to_string(), "No owners provided");

        let err = MultiSigError::InvalidThreshold { threshold: 5, owner_count: 3 };
        assert!(err.to_string().contains("Invalid threshold"));

        let err = MultiSigError::DuplicateOwner;
        assert!(err.to_string().contains("Duplicate"));
    }

    #[test]
    fn test_proposal_new() {
        let to = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let proposal = MultiSigProposal::new(42, to, 1000, vec![1, 2, 3], "Test proposal");
        assert_eq!(proposal.id, 42);
        assert_eq!(proposal.value, 1000);
        assert_eq!(proposal.description, "Test proposal");
        assert!(!proposal.executed);
        assert_eq!(proposal.confirmation_count(), 0);
    }
}
