use crate::value_objects::price::Price;
use serde::{Deserialize, Serialize};

/// A struct representing a price range with a lower and upper price bound.
///
/// The `PriceRange` struct is used to define a range of prices within a
/// specific lower and upper limit. This can be helpful in scenarios such
/// as filtering items, setting bounds for pricing algorithms, or defining
/// ranges for user-input limits.
///
/// ## Fields
///
/// * `lower_price` - The minimum price in the range. Represents the lower bound value.
/// * `upper_price` - The maximum price in the range. Represents the upper bound value.
///
/// ## Traits
///
/// The `PriceRange` struct derives several useful traits:
///
/// * `Debug` - Enables formatting the structure using the `{:?}` formatter for debugging purposes.
/// * `Clone` - Allows creating a duplicate of a `PriceRange` instance.
/// * `Serialize` - Supports serializing the struct into formats such as JSON, typically for storage or communication.
/// * `Deserialize` - Supports constructing a `PriceRange` instance from serialized data (e.g., JSON).
///
///
/// Note: The `Price` type must be predefined with your desired implementation
/// in order to use this struct effectively.
/// Represents a price range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceRange {
    /// The lower bound price.
    pub lower_price: Price,
    /// The upper bound price.
    pub upper_price: Price,
}

impl PriceRange {
    /// Creates a new PriceRange.
    ///
    /// # Parameters
    /// - `lower`: The lower bound of the price. This parameter defines the minimum price value.
    /// - `upper`: The upper bound of the price. This parameter defines the maximum price value.
    ///
    /// # Returns
    /// A new instance of the type containing the specified lower and upper price bounds.
    ///
    pub fn new(lower: Price, upper: Price) -> Self {
        Self {
            lower_price: lower,
            upper_price: upper,
        }
    }

    /// Checks whether a given price falls within the range defined by the current instance.
    ///
    /// # Parameters
    /// - `price`: A `Price` instance representing the value to check.
    ///
    /// # Returns
    /// - `true` if the price is greater than or equal to the lower bound (`self.lower_price`)
    ///   and less than or equal to the upper bound (`self.upper_price`).
    /// - `false` otherwise.
    ///
    pub fn contains(&self, price: Price) -> bool {
        price.value >= self.lower_price.value && price.value <= self.upper_price.value
    }
}
