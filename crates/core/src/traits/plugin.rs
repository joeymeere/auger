pub trait AugerPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
}