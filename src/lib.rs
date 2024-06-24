#[cfg(feature = "client")]
pub mod client;
pub mod protocol;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
