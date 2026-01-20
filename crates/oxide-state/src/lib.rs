//! OxideKit State & Persistence System
//!
//! This crate provides a unified state management and persistence system for OxideKit applications.
//! It enforces a single canonical approach to state handling, ensuring consistency across plugins
//! and applications.
//!
//! ## Architecture
//!
//! The state system is built around two fundamental concepts:
//!
//! - **UI State**: Ephemeral, view-specific state (scroll position, focus, animations)
//! - **App State**: Persistent, application-wide state (user data, settings, business logic)
//!
//! ## Persistence Tiers
//!
//! State can be persisted at different tiers based on sensitivity and requirements:
//!
//! - **Volatile**: In-memory only, lost on restart (UI state, temporary data)
//! - **Local**: Persisted to local storage (user preferences, cached data)
//! - **Secure**: Encrypted local storage (sensitive settings)
//! - **Encrypted**: Full encryption with key derivation (wallet data, admin credentials)
//! - **Syncable**: Designed for cloud synchronization (user data, settings)
//!
//! ## Example
//!
//! ```rust,ignore
//! use oxide_state::prelude::*;
//!
//! #[derive(AppState, Default, serde::Serialize, serde::Deserialize)]
//! #[state(tier = "local", version = 1)]
//! pub struct Settings {
//!     theme: String,
//!     language: String,
//! }
//!
//! #[derive(AppState, Default, serde::Serialize, serde::Deserialize)]
//! #[state(tier = "encrypted", version = 1)]
//! pub struct WalletState {
//!     encrypted_seed: Vec<u8>,
//! }
//! ```

#![warn(missing_docs)]

mod error;
mod persistence;
mod snapshot;
mod state;
mod sync;
mod tier;

#[cfg(feature = "encryption")]
mod encryption;

#[cfg(feature = "debug")]
mod debug;

pub mod migration;

pub use error::*;
pub use persistence::*;
pub use snapshot::*;
pub use state::*;
pub use sync::*;
pub use tier::*;

#[cfg(feature = "encryption")]
pub use encryption::*;

#[cfg(feature = "debug")]
pub use debug::*;

/// Convenient re-exports for common usage patterns.
pub mod prelude {
    pub use crate::error::{StateError, StateResult};
    pub use crate::migration::{Migration, MigrationRegistry};
    pub use crate::persistence::{PersistenceBackend, FileBackend, StateStore};
    pub use crate::snapshot::{StateSnapshot, SnapshotManager};
    pub use crate::state::{AppState, StateContainer, StateId, UiState};
    pub use crate::sync::{StateSync, SyncStatus};
    pub use crate::tier::PersistenceTier;

    #[cfg(feature = "sqlite")]
    pub use crate::persistence::SqliteBackend;

    #[cfg(feature = "encryption")]
    pub use crate::encryption::{EncryptedState, StateEncryption};

    #[cfg(feature = "debug")]
    pub use crate::debug::{StateInspector, StateDebugger};
}
