/* avatar.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::model::Layer;
// use image::{DynamicImage, GenericImage, ImageBuffer, Rgba};
use image::{DynamicImage, ImageFormat};
use log::{debug, error, warn};
use std::io::Cursor;

pub struct AvatarService;

impl AvatarService {
    pub async fn fetch_and_composite_avatar(player_id: u32, layers: &[Layer]) -> Option<Vec<u8>> {
        if layers.is_empty() {
            return None;
        }

        debug!("Compositing avatar for player {}", player_id);

        let mut base_image: Option<DynamicImage> = None;

        for layer in layers {
            let url = if layer.image.starts_with("/Img") {
                format!("https://www.hattrick.org{}", layer.image)
            } else {
                layer.image.clone()
            };

            debug!("Downloading layer for player {}: {}", player_id, url);

            match reqwest::get(&url).await {
                Ok(response) => {
                    match response.bytes().await {
                        Ok(bytes) => {
                            match image::load_from_memory(&bytes) {
                                Ok(img) => {
                                    if let Some(base) = &mut base_image {
                                        // Overlay current image on base
                                        image::imageops::overlay(
                                            base,
                                            &img,
                                            layer.x as i64,
                                            layer.y as i64,
                                        );
                                    } else {
                                        // First layer becomes base
                                        base_image = Some(img);
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to load image from memory for player {}: {}",
                                        player_id, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to get bytes for layer for player {}: {}",
                                player_id, e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to download layer for player {}: {}", player_id, e);
                }
            }
        }

        if let Some(img) = base_image {
            let mut bytes: Vec<u8> = Vec::new();
            let mut cursor = Cursor::new(&mut bytes);
            match img.write_to(&mut cursor, ImageFormat::Png) {
                Ok(_) => Some(bytes),
                Err(e) => {
                    error!(
                        "Failed to write composited avatar to PNG for player {}: {}",
                        player_id, e
                    );
                    None
                }
            }
        } else {
            warn!("No base image created for player {}", player_id);
            None
        }
    }
}
