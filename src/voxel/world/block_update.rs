pub mod rand_block_update;
mod simulations;

use super::World;
use crate::{
    gfx::ChunkTables,
    voxel::{
        is_valid::get_check_valid_fn, world_to_chunk_position, wrap_coord, Block, CHUNK_SIZE_I32,
        EMPTY_BLOCK,
    },
};
pub use simulations::run_test_simulations;
use std::collections::{HashMap, HashSet};

pub const BLOCK_UPDATE_INTERVAL: f32 = 0.2;
const ADJ: [(i32, i32, i32); 4] = [(1, 0, 0), (0, 0, 1), (-1, 0, 0), (0, 0, -1)];

type UpdateList = HashMap<(i32, i32, i32), Block>;

pub fn get_chunktable_updates(x: i32, y: i32, z: i32, update_mesh: &mut HashSet<(i32, i32, i32)>) {
    let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
    let ix = wrap_coord(x);
    let iy = wrap_coord(y);
    let iz = wrap_coord(z);
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                if (dx == -1 && ix != 0) || (dx == 1 && ix != CHUNK_SIZE_I32 - 1) {
                    continue;
                }

                if (dy == -1 && iy != 0) || (dy == 1 && iy != CHUNK_SIZE_I32 - 1) {
                    continue;
                }

                if (dz == -1 && iz != 0) || (dz == 1 && iz != CHUNK_SIZE_I32 - 1) {
                    continue;
                }

                update_mesh.insert((chunkx + dx, chunky + dy, chunkz + dz));
            }
        }
    }
}

fn add_water_tile(x: i32, y: i32, z: i32, level: u8, id: u8, to_update: &mut UpdateList) {
    let mut water = Block::new_fluid(id);
    water.geometry = level;

    if water.geometry == 0 || water.geometry > 8 {
        water.id = EMPTY_BLOCK;
    }

    if let Some(tile) = to_update.get(&(x, y, z)) {
        if !tile.is_fluid() {
            return;
        }

        if tile.id == water.id && tile.geometry < water.geometry && tile.geometry != 7 {
            to_update.insert((x, y, z), water);
        }
    } else {
        if water.geometry == 0 {
            to_update.insert((x, y, z), Block::new());
        }
        to_update.insert((x, y, z), water);
    }
}

//Returns true if updated
fn update_fluid(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList, decrease: u8) {
    let block = world.get_block(x, y, z);
    let below = world.get_block(x, y - 1, z);
    let level = block.geometry.min(7);

    //Check for adjacent tiles and see if they allow for this block to exist
    if block.geometry < 7 {
        let mut count = 0;
        let mut maxval = 0;
        let mut next_to_fall = false;
        for (dx, dy, dz) in ADJ {
            let (posx, posy, posz) = (x + dx, y + dy, z + dz);
            let block2 = world.get_block(posx, posy, posz);
            if block2.id != block.id {
                continue;
            }

            let underblock = world.get_block(posx, posy - 1, posz);
            if (underblock.id == EMPTY_BLOCK || underblock.id == block.id) && block2.geometry != 7 {
                continue;
            }

            if block2.geometry == 8 {
                next_to_fall = true;
                continue;
            }

            if block2.geometry > maxval {
                maxval = block2.geometry;
                count = 0;
            }

            if block2.geometry == maxval {
                count += 1;
            }
        }

        if maxval > 1 && (below.id == EMPTY_BLOCK || below.id == block.id) {
            add_water_tile(x, y, z, 1, block.id, to_update);
            if below.geometry != 7 {
                add_water_tile(x, y - 1, z, 8, block.id, to_update);
            }
            return;
        } else if maxval == 7 && count > 1 && decrease == 1 {
            add_water_tile(x, y, z, 7, block.id, to_update);
            return;
        } else if next_to_fall && maxval < 7 {
            add_water_tile(x, y, z, 7 - decrease, block.id, to_update);
        } else if maxval <= 1 {
            add_water_tile(x, y, z, 0, block.id, to_update);
            return;
        } else if maxval <= level {
            add_water_tile(x, y, z, maxval - decrease, block.id, to_update);
            return;
        }
    } else if block.geometry == 8 && world.get_block(x, y + 1, z).id != block.id {
        add_water_tile(x, y, z, 7 - decrease, block.id, to_update);
        return;
    }

    //Flow down
    if (below.id == EMPTY_BLOCK || below.id == block.id || below.fluid_destructibe()) && level > 0 {
        if below.geometry != 7 {
            add_water_tile(x, y - 1, z, 8, block.id, to_update);
        }
        if block.geometry != 7 {
            return;
        }
    }

    if level <= decrease {
        return;
    }

    //Flow to the sides
    for (dx, dy, dz) in ADJ {
        let (posx, posy, posz) = (x + dx, y + dy, z + dz);
        let adjacent = world.get_block(posx, posy, posz);
        if adjacent.id == EMPTY_BLOCK
            || (adjacent.id == block.id && adjacent.geometry < block.geometry - 1)
            || adjacent.fluid_destructibe()
        {
            let underblock = world.get_block(posx, posy - 1, posz);
            let blocklevel = if underblock.id == block.id || underblock.id == EMPTY_BLOCK {
                1.min(level)
            } else if level <= 7 {
                level - decrease
            } else {
                0
            };

            if level == 0 {
                continue;
            }

            add_water_tile(posx, posy, posz, blocklevel, block.id, to_update);
        }
    }
}

//Returns true if updated
fn water_to_stone(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) -> bool {
    //Is the block above lava?
    if world.get_block(x, y + 1, z).id == 13 {
        //then turn to stone
        to_update.insert((x, y, z), Block::new_id(2));
        return true;
    }
    false
}

//Returns true if updated
fn freeze_lava(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) -> bool {
    let block = world.get_block(x, y, z);
    let new_block = if block.geometry == 7 {
        //Obsidian
        Block::new_id(14)
    } else {
        //Stone
        Block::new_id(2)
    };

    //If water is above or to the side, then freeze the lava
    if world.get_block(x, y + 1, z).id == 12 {
        to_update.insert((x, y, z), new_block);
        return true;
    }

    for (dx, dy, dz) in ADJ {
        if world.get_block(x + dx, y + dy, z + dz).id == 12 {
            to_update.insert((x, y, z), new_block);
            return true;
        }
    }

    false
}

fn update_water(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    let updated = water_to_stone(world, x, y, z, to_update);
    if updated {
        return;
    }
    update_fluid(world, x, y, z, to_update, 1);
}

fn update_lava(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    let updated = freeze_lava(world, x, y, z, to_update);
    if updated {
        return;
    }
    let mut update_list = UpdateList::new();
    update_fluid(world, x, y, z, &mut update_list, 2);
    if world.ticks % 5 == 0 {
        for (pos, block) in update_list {
            let (px, py, pz) = pos;
            add_water_tile(px, py, pz, block.geometry, block.id, to_update);
        }
    } else if !update_list.is_empty() {
        let mut updated = false;
        for ((x, y, z), block) in update_list {
            if world.get_block(x, y, z) != block {
                updated = true;
            }
        }

        if !updated {
            return;
        }

        let mut block2 = world.get_block(x, y, z);
        block2.geometry |= 1 << 7;
        to_update.insert((x, y, z), block2);
    }
}

fn update_farmland(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    let above = world.get_block(x, y + 1, z);
    if above.transparent() || above.id == EMPTY_BLOCK {
        return;
    }
    to_update.insert((x, y, z), Block::new_id(4));
}

fn update_plant(world: &World, x: i32, y: i32, z: i32, id: u8, to_update: &mut UpdateList) {
    if let Some(check_valid) = get_check_valid_fn(id) {
        if check_valid(world, x, y, z) {
            return;
        }
        to_update.insert((x, y, z), Block::new_id(EMPTY_BLOCK));
    }
}

//Connect fences
fn update_fence(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    let mut block = world.get_block(x, y, z);

    let geometry = block.geometry;
    block.geometry = 0;
    ADJ.iter()
        .map(|(dx, dy, dz)| (x + dx, y + dy, z + dz))
        .map(|(x, y, z)| world.get_block(x, y, z))
        .enumerate()
        .for_each(|(i, b)| {
            if b.id == EMPTY_BLOCK {
                return;
            }
            if b.shape() != 0 {
                return;
            }
            if b.transparent() && b.id != block.id && b.id != 78 {
                return;
            }
            block.geometry |= 1 << i;
        });

    if block.geometry == geometry {
        return;
    }

    to_update.insert((x, y, z), block);
}

impl World {
    //Returns true if at least one block updated, otherwise false
    fn update_chunk(&mut self, chunkx: i32, chunky: i32, chunkz: i32, to_update: &mut UpdateList) {
        if let Some(chunk) = self.chunks.get(&(chunkx, chunky, chunkz)) {
            if chunk.is_empty() {
                return;
            }
        }

        let startx = chunkx * CHUNK_SIZE_I32;
        let starty = chunky * CHUNK_SIZE_I32;
        let startz = chunkz * CHUNK_SIZE_I32;

        for x in startx..(startx + CHUNK_SIZE_I32) {
            for y in starty..(starty + CHUNK_SIZE_I32) {
                for z in startz..(startz + CHUNK_SIZE_I32) {
                    let block = self.get_block(x, y, z);
                    if block.shape() != 0 {
                        continue;
                    }
                    match block.id {
                        //Water
                        12 => update_water(self, x, y, z, to_update),
                        //Lava
                        13 => update_lava(self, x, y, z, to_update),
                        //Farmland
                        43 | 45 => update_farmland(self, x, y, z, to_update),
                        //Plants, torches, ladders
                        47..=56 | 69 | 71..=75 | 77 | 88 | 90 => {
                            update_plant(self, x, y, z, block.id, to_update)
                        }
                        //Fence
                        76 => update_fence(self, x, y, z, to_update),
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn update_blocks(&mut self, dt: f32, chunktables: &mut ChunkTables, chunk_sim_dist: i32) {
        self.block_update_timer += dt;
        if self.block_update_timer <= BLOCK_UPDATE_INTERVAL {
            return;
        }

        self.ticks += 1;
        self.block_update_timer = 0.0;

        let mut to_update = UpdateList::new();
        let mut update_mesh = HashSet::<(i32, i32, i32)>::new();
        for x in (self.centerx - chunk_sim_dist)..=(self.centerx + chunk_sim_dist) {
            for y in (self.centery - chunk_sim_dist)..=(self.centery + chunk_sim_dist) {
                for z in (self.centerz - chunk_sim_dist)..=(self.centerz + chunk_sim_dist) {
                    if !self.updating.contains(&(x, y, z)) {
                        continue;
                    }

                    self.update_chunk(x, y, z, &mut to_update);
                }
            }
        }

        self.updating.clear();

        let mut light_updates = vec![];
        for ((x, y, z), block) in to_update {
            if self.get_block(x, y, z) == block {
                continue;
            }

            if block.geometry & (1 << 7) != 0 && block.is_fluid() {
                let mut block2 = block;
                block2.geometry &= !(1 << 7);
                self.set_block(x, y, z, block2);
                continue;
            }

            get_chunktable_updates(x, y, z, &mut update_mesh);
            self.set_block(x, y, z, block);
            light_updates.push((x, y, z));
        }

        update_mesh.extend(self.update_block_light(&light_updates));

        for (x, y, z) in update_mesh {
            chunktables.update_table(self, x, y, z);
        }
    }

    //Add all chunks to the updating list
    pub fn update_all_chunks(&mut self) {
        for chunkpos in self.chunks.keys() {
            self.updating.insert(*chunkpos);
            self.in_update_range.insert(*chunkpos);
        }
    }

    //Determine which chunks are in update range
    pub fn update_sim_range(&mut self, chunk_sim_dist: i32) {
        let mut update_range = HashSet::new();
        for x in (self.centerx - chunk_sim_dist)..=(self.centerx + chunk_sim_dist) {
            for y in (self.centery - chunk_sim_dist)..=(self.centery + chunk_sim_dist) {
                for z in (self.centerz - chunk_sim_dist)..=(self.centerz + chunk_sim_dist) {
                    update_range.insert((x, y, z));
                    if !self.in_update_range.contains(&(x, y, z)) {
                        self.updating.insert((x, y, z));
                        continue;
                    }
                }
            }
        }
        self.in_update_range = update_range;
    }
}
