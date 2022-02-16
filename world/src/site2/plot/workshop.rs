use super::*;
use crate::{
    util::{RandomField, Sampler},
    Land,
};
use common::terrain::{Block, BlockKind, SpriteKind};
use rand::prelude::*;
use vek::*;

/// Represents house data generated by the `generate()` method
pub struct Workshop {
    /// Axis aligned bounding region for the house
    bounds: Aabr<i32>,
    /// Approximate altitude of the door tile
    pub(crate) alt: i32,
}

impl Workshop {
    pub fn generate(
        land: &Land,
        _rng: &mut impl Rng,
        site: &Site,
        door_tile: Vec2<i32>,
        door_dir: Vec2<i32>,
        tile_aabr: Aabr<i32>,
    ) -> Self {
        let bounds = Aabr {
            min: site.tile_wpos(tile_aabr.min),
            max: site.tile_wpos(tile_aabr.max),
        };

        Self {
            bounds,
            alt: land.get_alt_approx(site.tile_center_wpos(door_tile + door_dir)) as i32,
        }
    }
}

impl Structure for Workshop {
    fn render(&self, _site: &Site, _land: &Land, painter: &Painter) {
        let brick = Fill::Brick(BlockKind::Rock, Rgb::new(80, 75, 85), 24);

        let base = self.alt + 1;
        let center = self.bounds.center();

        // Base
        painter
            .aabb(Aabb {
                min: (self.bounds.min + 1).with_z(base - 16),
                max: self.bounds.max.with_z(base),
            })
            .fill(brick.clone());

        let roof = base + 5;

        painter
            .aabb(Aabb {
                min: (self.bounds.min + 2).with_z(base),
                max: (self.bounds.max - 1).with_z(roof),
            })
            .clear();

        // Supports
        for pos in [
            Vec2::new(self.bounds.min.x + 3, self.bounds.min.y + 3),
            Vec2::new(self.bounds.max.x - 3, self.bounds.min.y + 3),
            Vec2::new(self.bounds.min.x + 3, self.bounds.max.y - 3),
            Vec2::new(self.bounds.max.x - 3, self.bounds.max.y - 3),
        ] {
            painter
                .line(pos.with_z(base), pos.with_z(roof), 1.0)
                .fill(Fill::Block(Block::new(
                    BlockKind::Wood,
                    Rgb::new(55, 25, 8),
                )));
        }

        let roof_top = roof + 5;

        // Roof
        painter
            .pyramid(Aabb {
                min: (self.bounds.min + 2).with_z(roof),
                max: (self.bounds.max - 1).with_z(roof_top),
            })
            .fill(Fill::Brick(BlockKind::Rock, Rgb::new(45, 28, 21), 24));

        let chimney = roof_top + 2;

        // Chimney
        let chimney_radius = 3.0;
        painter
            .line(
                center.with_z(base + 4),
                center.with_z(chimney),
                chimney_radius,
            )
            .fill(brick);
        painter
            .line(
                center.with_z(base),
                center.with_z(chimney + 2),
                chimney_radius - 1.0,
            )
            .clear();
        for x in -1..2 {
            for y in -1..2 {
                painter.sprite(
                    (center + Vec2::new(x, y)).with_z(base - 1),
                    SpriteKind::Ember,
                );
            }
        }
        for dir in CARDINALS {
            for d in 0..3 {
                let position = center + dir * (3 + d * 2);
                let mut stations = vec![
                    SpriteKind::CraftingBench,
                    SpriteKind::Forge,
                    SpriteKind::SpinningWheel,
                    SpriteKind::TanningRack,
                    SpriteKind::CookingPot,
                    SpriteKind::Cauldron,
                    SpriteKind::Loom,
                    SpriteKind::Anvil,
                    SpriteKind::DismantlingBench,
                ];
                if !stations.is_empty() {
                    let cr_station = stations.swap_remove(
                        RandomField::new(0).get(position.with_z(base)) as usize % stations.len(),
                    );
                    painter.sprite(position.with_z(base), cr_station);
                }
            }
        }
    }
}