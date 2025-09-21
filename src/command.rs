// This file is part of Ortie, a CLI to manage OAuth 2.0 access
// tokens.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

#[cfg(not(feature = "command"))]
pub mod missing_feature {
    pub const ERR: &'static str = "missing `command` cargo feature";

    pub fn serialize<S: serde::Serializer>(_: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(ERR))
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(_: D) -> Result<(), D::Error> {
        Err(serde::de::Error::custom(ERR))
    }
}
