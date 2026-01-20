//! OxideKit Loading States and Progress Indicators
//!
//! This crate provides comprehensive loading states and progress indicators for OxideKit applications:
//!
//! - **Progress Indicators**: Linear and circular progress bars with determinate, indeterminate, and buffer modes
//! - **Skeleton Loaders**: Placeholder components with shimmer and wave animations
//! - **Loading Overlays**: Full-screen, container, and inline loaders with backdrop options
//! - **Loading States**: State management with LoadingBoundary for async content
//! - **Pull to Refresh**: Touch-based refresh triggers with customizable thresholds
//! - **Infinite Scroll**: Scroll-based pagination with loading and end indicators
//! - **Button Loading**: Loading states for buttons with spinner integration
//!
//! # Accessibility
//!
//! All components include ARIA attributes for screen reader support:
//! - `aria-busy` to indicate loading state
//! - `aria-live` regions for dynamic content updates
//! - `aria-valuenow/min/max` for progress indicators
//! - Screen reader announcements for state changes
//!
//! # Example
//!
//! ```
//! use oxide_loading::prelude::*;
//!
//! // Create a determinate linear progress bar
//! let progress = LinearProgress::new()
//!     .mode(ProgressMode::Determinate)
//!     .value(0.65);
//!
//! // Create a skeleton card loader
//! let skeleton = SkeletonCard::new()
//!     .animation(SkeletonAnimation::Shimmer);
//!
//! // Use loading boundary for async content
//! let state = LoadingState::Loading;
//! let boundary = LoadingBoundary::new(state)
//!     .loader(LoaderType::Spinner);
//! ```

pub mod button;
pub mod infinite;
pub mod overlay;
pub mod progress;
pub mod refresh;
pub mod skeleton;
pub mod state;

pub use button::*;
pub use infinite::*;
pub use overlay::*;
pub use progress::*;
pub use refresh::*;
pub use skeleton::*;
pub use state::*;

/// Prelude module for convenient imports
pub mod prelude {
    // Progress indicators
    pub use crate::progress::{
        CircularProgress, CircularProgressSize, LinearProgress, ProgressMode,
    };

    // Skeleton loaders
    pub use crate::skeleton::{
        SkeletonAnimation, SkeletonAvatar, SkeletonCard, SkeletonImage, SkeletonTable,
        SkeletonText,
    };

    // Loading overlays
    pub use crate::overlay::{BackdropStyle, ContainerLoader, FullScreenLoader, InlineLoader};

    // Loading state management
    pub use crate::state::{LoadingBoundary, LoadingState, LoaderType};

    // Pull to refresh
    pub use crate::refresh::{PullIndicator, PullToRefresh, RefreshState};

    // Infinite scroll
    pub use crate::infinite::{EndIndicator, InfiniteScroll, ScrollDirection};

    // Button loading
    pub use crate::button::{ButtonLoadingState, LoadingButton};

    // Accessibility
    pub use crate::state::AccessibilityAnnouncer;
}
