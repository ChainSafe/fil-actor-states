// Copyright 2019-2025 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_bitfield::UnvalidatedBitField;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UnvalidatedBitFieldWrapper {
    inner: UnvalidatedBitField,
}

impl Clone for UnvalidatedBitFieldWrapper {
    fn clone(&self) -> Self {
        UnvalidatedBitFieldWrapper {
            inner: match &self.inner {
                UnvalidatedBitField::Validated(bitfield) => {
                    UnvalidatedBitField::Validated(bitfield.clone())
                }
                UnvalidatedBitField::Unvalidated(bytes) => {
                    UnvalidatedBitField::Unvalidated(bytes.clone())
                }
            },
        }
    }
}

impl PartialEq for UnvalidatedBitFieldWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (&self.inner, &other.inner) {
            (UnvalidatedBitField::Validated(a), UnvalidatedBitField::Validated(b)) => a == b,
            (UnvalidatedBitField::Unvalidated(a), UnvalidatedBitField::Unvalidated(b)) => a == b,
            _ => false,
        }
    }
}

impl From<UnvalidatedBitField> for UnvalidatedBitFieldWrapper {
    fn from(value: UnvalidatedBitField) -> Self {
        UnvalidatedBitFieldWrapper { inner: value }
    }
}

impl From<UnvalidatedBitFieldWrapper> for UnvalidatedBitField {
    fn from(wrapper: UnvalidatedBitFieldWrapper) -> Self {
        wrapper.inner
    }
}
