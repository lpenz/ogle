#[cfg(test)]
mod tests {
    use super::*;

    use crate::sys_input::SysInputReal;
    use crate::sys_virtual::SysVirtual;
    use crate::view_sequence::ViewSequence;

    use clap::Parser;

    #[tokio::test]
    async fn test_true() -> Result<()> {
        let o = ViewSequence::default();
        let cli = Cli::try_parse_from(["ogle", "-z", "--", "true"])?;
        let sys_input = SysInputReal::default();
        let mut sys = SysVirtual::default().into();
        run(sys_input, &mut sys, &cli, View::from(o)).await?;
        let Sys::SysVirtual(sys) = sys else {
            unreachable!()
        };
        assert_eq!(
            sys.log,
            vec!["<O> 1970-01-01 00:00:00 first execution", "+ "]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_false() -> Result<()> {
        let o = ViewSequence::default();
        let cli = Cli::try_parse_from(["ogle", "-e", "--", "false"])?;
        let sys_input = SysInputReal::default();
        let mut sys = SysVirtual::default().into();
        run(sys_input, &mut sys, &cli, View::from(o)).await?;
        let Sys::SysVirtual(sys) = sys else {
            unreachable!()
        };
        assert_eq!(
            sys.log,
            vec!["<O> 1970-01-01 00:00:00 first execution", "+ "]
        );
        Ok(())
    }
}
