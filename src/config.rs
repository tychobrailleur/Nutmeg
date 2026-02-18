/* config.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::env;

pub static VERSION: &str = "0.1.0";
pub static GETTEXT_PACKAGE: &str = "nutmeg";
pub static LOCALEDIR: &str = "/app/share/locale";
pub static PKGDATADIR: &str = "/app/share/nutmeg";

pub fn consumer_key() -> String {
    env::var("HT_CONSUMER_KEY").unwrap_or_default()
}

pub fn consumer_secret() -> String {
    env::var("HT_CONSUMER_SECRET").unwrap_or_default()
}
