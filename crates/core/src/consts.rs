// Anchor global namespace
pub const SIGHASH_GLOBAL_NAMESPACE: &str = "global";
pub const HASH_BYTES: usize = 32;

pub const STD_LIB_NAMES: &[&str] = &[
    "core",
    "alloc",
    "std",
    "compiler_builtins",
    "rustc_std_workspace_core",
    "rustc_std_workspace_alloc",
    "solana_program",
    "anchor_lang",
    "anchor_spl",
    "spl_token",
    "spl_associated_token_account",
    "spl_memo",
    "bytemuck",
    "borsh",
    "num_derive",
    "num_traits",
    "thiserror",
];

pub const NATIVE_INSTRUCTIONS: &[&str] = &[
    "SystemInstruction",
    "LoaderInstruction",
    "VoteInstruction",
    "AddressLookupTableInstruction",
    "CreateIdempotent",
];

pub const PROTECTED_INSTRUCTIONS: &[&str] = &[
    "IdlCreateAccount",
    "IdlCloseAccount",
    "IdlWrite",
    "IdlSetAuthority",
    "IdlResizeAccount",
];

pub const REMOVABLE_KEYWORDS: &[&str] = &["Instruction", "anchor", "idl", "space", "index"];

pub const FALSE_POSITIVES: &[&str] = &["anchor", "idl", "space", "index", "rs"];

pub const COMMON_ACCOUNT_NAME_CHUNKS: &[&str] = &[
    "system_program",
    "token_program",
    "token_2022_program",
    "token22_program",
    "token_22_program",
    "associated_token_program",
    "token_metadata_program",
    "mpl_token_metadata",
    "mpl_token_metadata_program",
    "address_lookup_table",
    "mpl_core",
    "mpl_core_program",
    "rent",
    "payer",
    "signer",
    "authority",
    "owner",
    "fee_payer",
    "admin",
    "state",
    "pool",
    "vault",
    "escrow",
    "token",
    "token_account",
    "ata",
    "lp",
    "amm",
    "clmm",
    "dlmm",
    "emission",
    "treasury",
    "position",
    "source",
    "from",
    "to",
    "withdrawer",
    "admin",
    "user",
    "yield",
    "farm",
    "market",
    "oracle",
    "program",
];
