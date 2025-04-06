//! ## `wares`
//! 
//! This module holds traits for [`StartingBytesware`], [`Middleware`], [`EndingBytesware`] and their default implementations.
//! Because they are async traits, they will look dumb, but look at the examples and everything would be more understandable.
//! 
//! Also, it holds [`AfterConnect`] and [`BeforeConnect`]
pub mod starting_bytesware;
pub mod middleware;
pub mod ending_bytesware;
pub mod before_connect;
pub mod after_connect;

pub use starting_bytesware::StartingBytesware;
pub use middleware::Middleware;
pub use ending_bytesware::EndingBytesware;
pub use before_connect::BeforeConnect;
pub use after_connect::AfterConnect;

/// I know thats just the definition, and not a real type, but still easier to carry around.
pub type Route = (Box<dyn Middleware>, Box<dyn EndingBytesware>);

use std::sync::atomic::{AtomicUsize, Ordering};

// Also the thing, for the AfterConnect and BeforeConnect.
static TOTAL_CONNECTED: AtomicUsize = AtomicUsize::new(0);

/// Thingy for the default implementation of the [`AfterConnect`] and [`BeforeConnect`]
pub fn increment_total_connected() {
    TOTAL_CONNECTED.fetch_add(1, Ordering::SeqCst);
}

/// Thingy for the default implementation of the [`AfterConnect`] and [`BeforeConnect`]
pub fn decrement_total_connected() {
    TOTAL_CONNECTED.fetch_sub(1, Ordering::SeqCst);
}

/// Thingy for the default implementation of the [`AfterConnect`] and [`BeforeConnect`]
pub fn get_total_connected() -> usize {
    TOTAL_CONNECTED.load(Ordering::SeqCst)
}