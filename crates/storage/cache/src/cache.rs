pub trait Cache: Send + Sync {
    fn commit(&self) -> eyre::Result<()>;
}
