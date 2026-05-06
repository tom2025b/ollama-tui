#[cfg(test)]
mod tests {
    use crate::subcommands::spec::{SubcommandId, SwarmArgs};

    #[test]
    fn swarm_with_task_carries_the_task() {
        let args = SwarmArgs {
            task: Some("explain ownership".to_string()),
        };
        let id = SubcommandId::Swarm(args.clone());
        assert_eq!(id, SubcommandId::Swarm(args));
    }

    #[test]
    fn swarm_without_task_is_default_args() {
        let id = SubcommandId::Swarm(SwarmArgs::default());
        let SubcommandId::Swarm(args) = id else {
            panic!("expected Swarm");
        };
        assert!(args.task.is_none());
    }
}
