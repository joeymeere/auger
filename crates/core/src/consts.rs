// Anchor global namespace
pub const SIGHASH_GLOBAL_NAMESPACE: &str = "global";
pub const HASH_BYTES: usize = 32;

pub const STD_LIB_NAMES: &[&str] = &[
    "core",
    "alloc",
    "std",
    "test",
    "proc_macro",
    "error",
    "future",
    "fmt",
    "env",
    "any",
    "arch",
    "array",
    "ascii",
    "backtrace",
    "borrow",
    "boxed",
    "cell",
    "char",
    "clone",
    "cmp",
    "collections",
    "convert",
    "default",
    "env",
    "error",
    "f32",
    "f64",
    "ffi",
    "fmt",
    "fs",
    "future",
    "hash",
    "hint",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "io",
    "isize",
    "iter",
    "marker",
    "mem",
    "net",
    "num",
    "ops",
    "option",
    "os",
    "panic",
    "path",
    "pin",
    "prelude",
    "primitive",
    "process",
    "ptr",
    "rc",
    "result",
    "slice",
    "str",
    "string",
    "sync",
    "task",
    "thread",
    "time",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "usize",
    "vec",
    "assert_matches",
    "async_iter",
    "autodiff",
    "f16",
    "f128",
    "intrinsics",
    "pat",
    "pipe",
    "random",
    "simd",
    "unsafe_binder",
    "compiler_builtins",
    "rustc_std_workspace_core",
    "rustc_std_workspace_alloc",
];

pub const ANCILLARY_LIB_NAMES: &[&str] = &[
    "anchor_lang",
    "anchor_spl",
    "spl_token",
    "spl_associated_token_account",
    "spl_memo",
    "pinnochio",
    "solana_program",
    "borsh",
    "num_derive",
    "num_traits",
    "thiserror",
    "bytemuck",
    "ruint",
    "sokoban",
    "fixed",
    "arrayref",
    "arrayvec",
    "itertools"
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

pub const REMOVABLE_KEYWORDS: &[&str] = &["Instruction", "anchor", "idl", "space", "invalid", "value", "index"];

pub const FALSE_POSITIVES: &[&str] = &["anchor", "idl", "space", "index", "rs", "invalid", "Invalid", "value"];

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
