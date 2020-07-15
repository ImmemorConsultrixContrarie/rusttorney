pub trait Command {
    fn from_protocol(name: String, args: Vec<String>) -> Result<Self, anyhow::Error>;
}