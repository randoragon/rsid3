// rsid3 - a simple, command line ID3v2 tag editor designed for scripting
// Copyright (C) 2024  Randoragon
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
use anyhow::Result;
use vergen::EmitBuilder;

pub fn main() -> Result<()> {
    // NOTE: This will output only a build timestamp and long SHA from git.
    // NOTE: This set requires the build and git features.
    // NOTE: See the EmitBuilder documentation for configuration options.
    EmitBuilder::builder()
        .build_timestamp()
        .git_sha(false)
        .emit()?;
    Ok(())
}
