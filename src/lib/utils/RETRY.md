# GnostrRetry

`gnostr::utils::retry` is a Rust library that provides utilities for retrying operations with different strategies.

This library provides several retry strategies, including linear, exponential, and their asynchronous versions. You can choose the strategy that best fits your needs.

The library is designed to be simple and easy to use. It provides a single enum, `GnostrRetry`, that represents different retry strategies. You can create a new retry strategy by calling one of the `new_*` methods on the `GnostrRetry` enum.

The library provides a `run` method that takes a closure and runs the operation with the specified retry strategy. The `run` method returns the result of the operation, or an error if the operation fails after all retries.

The run method expects the closure to return a `Result` type. The `Ok` variant should contain the result of the operation, and the `Err` variant should contain the error that occurred during the operation.

# Features

* **Linear Retry**: In this strategy, the delay between retries is constant.
* **Exponential Retry**: In this strategy, the delay between retries doubles after each retry.
* **Linear Async Retry**: This is an asynchronous version of the linear retry strategy. This feature is only available when the `async` feature is enabled.
* **Exponential Async Retry**: This is an asynchronous version of the exponential retry strategy. This feature is only available when the `async` feature is enabled.

# Examples

```
use gnostr::utils::retry::GnostrRetry;

fn my_sync_fn(_n: &str) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "generic error"))
}

// Retry the operation with a linear strategy (1 second delay, 2 retries)
let retry_strategy = GnostrRetry::new_linear(1, 2);
let result = retry_strategy.run(|| my_sync_fn("Hi"));
assert!(result.is_err());

```

# Asynchronous Example

```rust
use gnostr::utils::retry::GnostrRetry;

async fn my_async_fn(_n: u64) -> Result<(), std::io::Error> {
   Err(std::io::Error::new(std::io::ErrorKind::Other, "generic error"))
}

#[tokio::main]
async fn main() {
    // Retry the operation with an exponential strategy (1 second delay, 2 retries)
    let retry_strategy = GnostrRetry::new_exponential_async(1, 2);
    let result = retry_strategy.run_async(|| my_async_fn(42)).await;
    assert!(result.is_err());

}
```
# Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
gnostr = "*"
```

Then, add this to your crate root (`main.rs` or `lib.rs`):

```rust
use gnostr::utils::retry;
```

Now, you can use the `GnostrRetry` enum to create a retry strategy:

```rust
use gnostr::utils::retry::GnostrRetry;

let retry_strategy = GnostrRetry::new_linear(100, 5);
```

# License

This project is licensed under the MIT License.

#![deny(missing_docs)]
use std::fmt::Debug;
//#[cfg(feature = "async")]
use std::future::Future;
#[derive(Debug, Copy, Clone)]

/// `GnostrRetry` is an enum representing different kinds of retry strategies.
pub enum GnostrRetry {
    /// Represents a linear retry strategy.
    Linear {
        #[doc(hidden)]
        /// The delay between retries in seconds.
        delay: u64,
        #[doc(hidden)]
        /// The number of retries.
        retries: u64,
    },
    /// Represents an exponential retry strategy.
    Exponential {
        /// The delay between retries in seconds.
        #[doc(hidden)]
        delay: u64,
        /// The number of retries.
        #[doc(hidden)]
        retries: u64,
    },
    /// Represents an asynchronous version of the linear retry strategy.
    ///
    /// This variant is only available when the `async` feature is enabled.
    //#[cfg(feature = "async")]
    LinearAsync {
        /// The delay between retries in seconds.
        #[doc(hidden)]
        delay: u64,
        /// The number of retries.
        #[doc(hidden)]
        retries: u64,
    },
    /// Represents an asynchronous version of the exponential retry strategy.
    ///
    /// This variant is only available when the `async` feature is enabled.
    //#[cfg(feature = "async")]
    ExponentialAsync {
        /// The delay between retries in seconds.
        #[doc(hidden)]
        delay: u64,
        /// The number of retries.
        #[doc(hidden)]
        retries: u64,
    },
}

impl GnostrRetry {
    /// Creates a new `GnostrRetry::Linear` variant with the specified delay and number of retries.
    ///
    /// # Arguments
    ///
    /// * `delay` - The delay between retries in seconds.
    /// * `retries` - The number of retries.
    ///
    /// # Examples
    ///
    /// ```
    /// use gnostr::utils::retry::GnostrRetry;
    ///
    /// let retry_strategy = GnostrRetry::new_linear(100, 5);
    /// ```
    pub fn new_linear(delay: u64, retries: u64) -> Self {
        GnostrRetry::Linear { delay, retries }
    }

    /// Creates a new `GnostrRetry::Exponential` variant with the specified initial delay and number of retries.
    ///
    /// # Arguments
    ///
    /// * `delay` - The delay between retries in . The delay doubles after each retry.
    /// * `retries` - The number of retries.
    ///
    /// # Examples
    ///
    /// ```
    /// use gnostr::utils::retry::GnostrRetry;
    ///
    /// let retry_strategy = GnostrRetry::new_exponential(100, 5);
    /// ```
    pub fn new_exponential(delay: u64, retries: u64) -> Self {
        GnostrRetry::Exponential { delay, retries }
    }

    /// Creates a new `GnostrRetry::LinearAsync` variant with the specified delay and number of retries.
    ///
    /// # Arguments
    ///
    /// * `delay` - The delay between retries in seconds.
    /// * `retries` - The number of retries.
    ///
    /// # Examples
    ///
    /// ```
    /// use gnostr::utils::retry::GnostrRetry;
    ///
    /// let retry_strategy = GnostrRetry::new_linear_async(100, 5);
    /// ```
    //#[cfg(feature = "async")]
    pub fn new_linear_async(delay: u64, retries: u64) -> Self {
        GnostrRetry::LinearAsync { delay, retries }
    }

    /// Creates a new `GnostrRetry::ExponentialAsync` variant with the specified initial delay and number of retries.
    ///
    /// # Arguments
    ///
    /// * `delay` - The delay between retries in seconds. The delay doubles after each retry.
    /// * `retries` - The number of retries.
    ///
    /// # Examples
    ///
    /// ```
    /// use gnostr::utils::retry::GnostrRetry;
    ///
    /// let retry_strategy = GnostrRetry::new_exponential_async(100, 5);
    /// ```
    //#[cfg(feature = "async")]
    pub fn new_exponential_async(delay: u64, retries: u64) -> Self {
        GnostrRetry::ExponentialAsync { delay, retries }
    }

    /// Runs the provided function `f` with a retry strategy.
    ///
    /// This function takes a function `f` that implements the `SyncReturn` trait and runs it with a retry strategy. The `SyncReturn` trait is implemented for `FnMut` closures, which can mutate their captured variables and can be called multiple times.
    ///
    /// The function `f` should return a `Result` with the operation's result or error. The types of the result and error are determined by the `SyncReturn` trait's associated types `Item` and `Error`.
    ///
    /// # Errors
    ///
    /// Will return an error if the operation fails after all retries.
    pub fn run<T>(&self, f: T) -> Result<<T as SyncReturn>::Item, <T as SyncReturn>::Error>
    where
        T: SyncReturn,
    {
        Retry::run(f, *self)
    }

    /// Runs the provided function `f` with a retry strategy.
    ///
    /// This function takes a function `f` that implements the `AsyncReturn` trait and runs it with a retry strategy. The `AsyncReturn` trait is implemented for `FnMut` closures, which can mutate their captured variables and can be called multiple times. This function is only available when the `async` feature is enabled.
    ///
    /// The function `f` should return a `Result` with the operation's result or error. The types of the result and error are determined by the `SyncReturn` trait's associated types `Item` and `Error`.
    /// # Errors
    ///
    /// Will return an error if the operation fails after all retries.
    //#[cfg(feature = "async")]
    pub async fn run_async<'a, T>(
        &'a self,
        f: T,
    ) -> Result<<T as AsyncReturn>::Item, <T as AsyncReturn>::Error>
    where
        T: AsyncReturn + 'a + 'static,
    {
        Retry::run_async(f, *self).await
    }

    fn get_retries(&self) -> u64 {
        match self {
            GnostrRetry::Linear { retries, .. } => *retries,
            GnostrRetry::Exponential { retries, .. } => *retries,
            //#[cfg(feature = "async")]
            GnostrRetry::LinearAsync { retries, .. } => *retries,
            //#[cfg(feature = "async")]
            GnostrRetry::ExponentialAsync { retries, .. } => *retries,
        }
    }

    fn get_delay(&self) -> u64 {
        match self {
            GnostrRetry::Linear { delay, .. } => *delay,
            GnostrRetry::Exponential { delay, .. } => *delay,
            //#[cfg(feature = "async")]
            GnostrRetry::LinearAsync { delay, .. } => *delay,
            //#[cfg(feature = "async")]
            GnostrRetry::ExponentialAsync { delay, .. } => *delay,
        }
    }

    fn linear(x: u64) -> u64 {
        x
    }

    fn exponential(x: u64) -> u64 {
        2u64.pow(x.try_into().unwrap_or_default())
    }

    fn retry_fn(&self) -> fn(u64) -> u64 {
        match self {
            GnostrRetry::Linear { .. } => Self::linear,
            GnostrRetry::Exponential { .. } => Self::exponential,
            //#[cfg(feature = "async")]
            GnostrRetry::LinearAsync { .. } => Self::linear,
            //#[cfg(feature = "async")]
            GnostrRetry::ExponentialAsync { .. } => Self::exponential,
        }
    }
}

fn do_retry<F, T, E>(mut f: F, t: GnostrRetry) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut retries: u64 = 0;
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                if retries >= t.get_retries() {
                    return Err(e);
                }
                retries += 1;
                std::thread::sleep(std::time::Duration::from_secs((t.retry_fn())( 
                    t.get_delay(),
                )));
            }
        }
    }
}

//#[cfg(feature = "async")]
async fn do_retry_async<F, T, E>(mut f: F, t: GnostrRetry) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn Future<Output = Result<T, E>>>>,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if retries >= t.get_retries() {
                    return Err(e);
                }
                retries += 1;
                tokio::time::sleep(std::time::Duration::from_secs((t.retry_fn())(
                    t.get_delay(),
                )))
                .await;
            }
        }
    }
}
struct Retry;
impl Retry {
    //#[cfg(feature = "async")]
    async fn run_async<T>(
        mut f: T,
        t: GnostrRetry,
    ) -> Result<<T as AsyncReturn>::Item, <T as AsyncReturn>::Error>
    where
        T: AsyncReturn + 'static,
    {
        do_retry_async(move || Box::pin(f.run()), t).await
    }

    fn run<T>(mut f: T, t: GnostrRetry) -> Result<<T as SyncReturn>::Item, <T as SyncReturn>::Error>
    where
        T: SyncReturn,
    {
        do_retry(move || f.run(), t)
    }
}
/// The `AsyncReturn` trait is used for operations that need to return a value asynchronously.
///
/// This trait provides a single method, `run`, which takes no arguments and returns a `Future` that resolves to a `Result` with the operation's result or error. Both the result and error types must implement the `Debug` trait.
///
/// This trait is only available when the `async` feature is enabled.
///
/// # Associated Types
///
/// * `Item`: The type of the value returned by the `run` method. This type must implement the `Debug` trait.
/// * `Error`: The type of the error returned by the `run` method. This type must also implement the `Debug` trait.
/// * `Future`: The type of the `Future` returned by the `run` method. This `Future` should resolve to a `Result<Item, Error>`.
///
/// # Methods
///
/// * `run`: Performs the operation and returns a `Future` that resolves to the result.
///
/// # Examples
///
/// ```
/// use gnostr::utils::retry::AsyncReturn;
/// use std::fmt::Debug;
/// use futures::future::ready;
///
/// struct MyOperation;
///
/// impl AsyncReturn for MyOperation {
///     type Item = i32;
///     type Error = &'static str;
///     type Future = futures::future::Ready<Result<Self::Item, Self::Error>>;
///
///     fn run(&mut self) -> Self::Future {
///         // Perform the operation and return the result...
///         ready(Ok(42))
///     }
/// }
///
/// let mut operation = MyOperation;
/// let future = operation.run();
/// ```
//#[cfg(feature = "async")]
pub trait AsyncReturn {
    /// The type of the value returned by the `run` method.
    type Item: Debug;
    /// The type of the error returned by the `run` method.
    type Error: Debug;
    /// The type of the `Future` returned by the `run` method.
    type Future: Future<Output = Result<Self::Item, Self::Error>>;

    /// Performs the operation and returns a `Future` that resolves to the result.
    fn run(&mut self) -> Self::Future;
}

//#[cfg(feature = "async")]
impl<I: Debug, E: Debug, T: Future<Output = Result<I, E>>, F: FnMut() -> T> AsyncReturn for F {
    type Item = I;
    type Error = E;
    type Future = T;
    fn run(&mut self) -> Self::Future {
        self()
    }
}

/// The `SyncReturn` trait is used for operations that need to return a value synchronously.
///
/// This trait provides a single method, `run`, which takes no arguments and returns a `Result` with the operation's result or error. Both the result and error types must implement the `Debug` trait.
///
/// # Type Parameters
///
/// * `Item`: The type of the value returned by the `run` method. This type must implement the `Debug` trait.
/// * `Error`: The type of the error returned by the `run` method. This type must also implement the `Debug` trait.
///
/// # Methods
///
/// * `run`: Performs the operation and returns the result.
///
/// # Errors
///
/// If the operation fails, this method returns `Err` containing the error. The type of the error is defined by the `Error` associated type.
///
/// # Examples
///
/// ```
/// use gnostr::utils::retry::SyncReturn;
/// use std::fmt::Debug;
///
/// struct MyOperation;
///
/// impl SyncReturn for MyOperation {
///     type Item = i32;
///     type Error = &'static str;
///
///     fn run(&mut self) -> Result<Self::Item, Self::Error> {
///         // Perform the operation and return the result...
///         Ok(42)
///     }
/// }
///
/// let mut operation = MyOperation;
/// assert_eq!(operation.run(), Ok(42));
/// ```
pub trait SyncReturn {
    /// The type of the value returned by the `run` method.
    type Item: Debug;
    /// The type of the error returned by the `run` method.
    type Error: Debug;

    /// Performs the operation and returns the result.
    fn run(&mut self) -> Result<Self::Item, Self::Error>;
}

impl<I: Debug, E: Debug, F: FnMut() -> Result<I, E>> SyncReturn for F {
    type Item = I;
    type Error = E;
    fn run(&mut self) -> Result<Self::Item, Self::Error> {
        self()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Clone)]
    struct NotCopy {
        pub _n: usize,
    }

    fn to_retry_not_copy(n: &NotCopy) -> Result<(), std::io::Error> {
        let _r = n.clone();

        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "generic error",
        ))
    }

    fn to_retry(_n: usize) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "generic error",
        ))
    }

    //#[cfg(feature = "async")]
    async fn to_retry_async(n: usize) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "generic error",
        ))
    }

    #[test]
    fn test_linear() {
        let retries = 2;
        let delay = 1;
        let instant = std::time::Instant::now();
        let s = GnostrRetry::Linear { retries, delay }.run(|| to_retry(1));
        assert!(s.is_err());
        let elapsed = instant.elapsed();
        assert!(elapsed.as_secs() >= retries * delay);
    }

    #[test]
    fn test_expontential() {
        let retries = 2;
        let delay = 1;
        let instant = std::time::Instant::now();
        let s = GnostrRetry::Exponential { retries, delay }.run(|| to_retry(1));
        assert!(s.is_err());
        let elapsed = instant.elapsed();
        assert!(elapsed.as_secs() >= retries * delay);
    }

    #[test]
    fn test_not_copy() {
        let retries = 2;
        let delay = 1;
        let not_copy = NotCopy { _n: 1 };
        let instant = std::time::Instant::now();

        let s = GnostrRetry::Linear { retries, delay }.run(|| to_retry_not_copy(&not_copy));
        assert!(s.is_err());
        let elapsed = instant.elapsed();
        assert!(elapsed.as_secs() >= retries * delay);
    }

    //#[cfg(feature = "async")]
    #[tokio::test]
    async fn test_linear_async() {
        let retries = 2;
        let delay = 1;
        let instant = std::time::Instant::now();
        let s = GnostrRetry::LinearAsync { retries, delay }
            .run_async(|| to_retry_async(1))
            .await;
        assert!(s.is_err());
        let elapsed = instant.elapsed();
        assert!(elapsed.as_secs() >= retries * delay);
    }

    //#[cfg(feature = "async")]
    #[tokio::test]
    async fn test_expontential_async() {
        let retries = 2;
        let delay = 1;
        let instant = std::time::Instant::now();
        let s = GnostrRetry::ExponentialAsync { retries, delay }
            .run_async(|| to_retry_async(1))
            .await;
        assert!(s.is_err());
        let elapsed = instant.elapsed();
        assert!(elapsed.as_secs() >= retries * delay);
    }
}
