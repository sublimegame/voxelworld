//Array of voxel flags
static mut VOXEL_FLAGS: [u8; 256] = [0; 256];

pub const TRANSPARENT_FLAG: u8 = 1 << 0;
pub const CONNECT_FLAG: u8 = 1 << 1;
pub const CAN_ROTATE_FLAG: u8 = 1 << 2;
pub const NO_HITBOX: u8 = 1 << 3;
pub const FLUID: u8 = 1 << 4;
pub const ROTATE_Y_ONLY: u8 = 1 << 5;
pub const FLAT_ITEM: u8 = 1 << 6;

unsafe fn set_plant_flags(voxel_id: usize) {
    VOXEL_FLAGS[voxel_id] |= TRANSPARENT_FLAG;
    VOXEL_FLAGS[voxel_id] |= NO_HITBOX;
    VOXEL_FLAGS[voxel_id] |= FLAT_ITEM;
}

//TODO: Have a better way of configuring block flags other than hardcoding
//This function should be called at the start of the game
pub fn init_voxel_flags() {
    unsafe {
        //Leaves
        VOXEL_FLAGS[7] |= TRANSPARENT_FLAG;
        //Log
        VOXEL_FLAGS[8] |= CAN_ROTATE_FLAG;
        //Glass
        VOXEL_FLAGS[9] |= TRANSPARENT_FLAG;
        VOXEL_FLAGS[9] |= CONNECT_FLAG;
        //Water
        VOXEL_FLAGS[12] |= TRANSPARENT_FLAG;
        VOXEL_FLAGS[12] |= CONNECT_FLAG;
        VOXEL_FLAGS[12] |= NO_HITBOX;
        VOXEL_FLAGS[12] |= FLUID;
        //Lava
        VOXEL_FLAGS[13] |= TRANSPARENT_FLAG;
        VOXEL_FLAGS[13] |= CONNECT_FLAG;
        VOXEL_FLAGS[13] |= NO_HITBOX;
        VOXEL_FLAGS[13] |= FLUID;
        //Chest
        VOXEL_FLAGS[37] |= CAN_ROTATE_FLAG;
        VOXEL_FLAGS[37] |= ROTATE_Y_ONLY;
        //Furnace
        VOXEL_FLAGS[40] |= CAN_ROTATE_FLAG;
        VOXEL_FLAGS[40] |= ROTATE_Y_ONLY;
        //Sapling
        set_plant_flags(47);
        //Mushroom
        set_plant_flags(48); //Yeah yeah mushrooms are fungi not plants,
                             //I know that, but they have the same voxel flags
                             //so I don't give a flying duck
                             //Tall grass
        set_plant_flags(49);
        //Wheat
        set_plant_flags(50);
        set_plant_flags(51);
        set_plant_flags(52);
        set_plant_flags(53);
        //Red flower
        set_plant_flags(54);
        //Yellow floer
        set_plant_flags(55);
        //Blue flower
        set_plant_flags(56);
    }
}

//Read only
pub fn get_flag(id: u8) -> u8 {
    unsafe { VOXEL_FLAGS[id as usize] }
}
