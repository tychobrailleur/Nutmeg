/* team.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#[derive(Debug, Clone, Default)]
pub struct Team {
    pub id: u32,
    pub name: String,
    pub short_name: Option<String>,
    pub league_id: Option<u32>,
    pub league_name: Option<String>,
    pub country_id: Option<u32>,
    pub country_name: Option<String>,
}
