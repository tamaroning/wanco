pub mod stackmap;

use anyhow::Result;
use inkwell::{
    types::BasicTypeEnum,
    values::{BasicMetadataValueEnum, BasicValue, PointerValue},
};

use crate::context::Context;

use super::cr::{gen_compare_migration_state, MIGRATION_STATE_CHECKPOINT_START};
