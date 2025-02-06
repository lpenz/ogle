// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use mockall::automock;

use crate::time_wrapper::Instant;

#[automock]
pub trait SysApi: std::fmt::Debug + Default {
    fn now(&self) -> Instant;
}
