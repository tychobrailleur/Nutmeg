/* config.rs
 *
 * Copyright 2026 sebastien
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
pub static GETTEXT_PACKAGE: &str = "hoctane";
pub static LOCALEDIR: &str = "/app/share/locale";
pub static PKGDATADIR: &str = "/app/share/hoctane";

pub fn consumer_key() -> String {
    env::var("HT_CONSUMER_KEY").unwrap_or_default()
}

pub fn consumer_secret() -> String {
    env::var("HT_CONSUMER_SECRET").unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_consumer_keys() {
        let key = "TEST_KEY_123";
        let secret = "TEST_SECRET_456";
        unsafe {
            env::set_var("HT_CONSUMER_KEY", key);
            env::set_var("HT_CONSUMER_SECRET", secret);
        }

        assert_eq!(consumer_key(), key);
        assert_eq!(consumer_secret(), secret);

        unsafe {
            env::remove_var("HT_CONSUMER_KEY");
            env::remove_var("HT_CONSUMER_SECRET");
        }
    }
}
