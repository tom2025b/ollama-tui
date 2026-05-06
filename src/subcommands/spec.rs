pub trait Subcommand {
    fn name(&self) -> &'static str;
}
