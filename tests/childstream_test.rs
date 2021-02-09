// Copyright (C) 2021 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use ogle::childstream::*;

use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;
use tokio_stream::StreamExt;

#[tokio::test]
async fn basicout() -> Result<()> {
    let mut cmd = Command::new("/bin/sh");
    cmd.args(&["-c", "printf 'test1\ntest2'"]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let child = cmd.spawn()?;
    let mut childstream = ChildStream::from_child(child)?;
    assert_eq!(
        childstream.next().await,
        Some(Item::Stdout("test1".to_owned()))
    );
    assert_eq!(
        childstream.next().await,
        Some(Item::Stdout("test2".to_owned()))
    );
    let exitstatus = childstream.next().await;
    if let Some(Item::Done(sts)) = exitstatus {
        assert!(sts.success());
    } else {
        panic!("invalid exit status {:?}", exitstatus);
    }
    assert_eq!(childstream.next().await, None);
    Ok(())
}

#[tokio::test]
async fn basicerr() -> Result<()> {
    let mut cmd = Command::new("/bin/sh");
    cmd.args(&["-c", "printf 'test1\ntest2' >&2"]);
    let mut childstream = ChildStream::from_command(cmd)?;
    assert_eq!(
        childstream.next().await,
        Some(Item::Stderr("test1".to_owned()))
    );
    assert_eq!(
        childstream.next().await,
        Some(Item::Stderr("test2".to_owned()))
    );
    let exitstatus = childstream.next().await;
    if let Some(Item::Done(sts)) = exitstatus {
        assert!(sts.success());
    } else {
        panic!("invalid exit status {:?}", exitstatus);
    }
    assert_eq!(childstream.next().await, None);
    Ok(())
}
