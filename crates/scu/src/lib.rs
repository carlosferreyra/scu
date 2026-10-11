//! Public library for `scu`.

/// Returns a starter message.
#[must_use]
pub const fn hello() -> &'static str {
    "Hello from scu!"
}

#[cfg(test)]
mod tests {
    #[test]
    fn hello_returns_expected_message() {
        assert_eq!(super::hello(), "Hello from scu!");
    }
}
