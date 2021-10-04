pub mod types;
pub mod entrypoint;
pub mod instruction;
pub mod processor;

solana_program::declare_id!("FZiJXJ5ZhMvYDE5JjRs9P6vQP5TxbKVP63y3WgDVQUQb");
// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;