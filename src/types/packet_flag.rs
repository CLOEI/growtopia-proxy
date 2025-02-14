use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PacketFlag {
    pub none: bool,
    pub walk: bool,
    pub unk_2: bool,
    pub spawn_related: bool,
    pub extended: bool,
    pub facing_left: bool,
    pub standing: bool,
    pub fire_damage: bool,
    pub jump: bool,
    pub got_killed: bool,
    pub punch: bool,
    pub place: bool,
    pub tile_change: bool,
    pub got_punched: bool,
    pub respawn: bool,
    pub object_collect: bool,
    pub trampoline: bool,
    pub damage: bool,
    pub slide: bool,
    pub parasol: bool,
    pub unk_gravity_related: bool,
    pub swim: bool,
    pub wall_hang: bool,
    pub power_up_punch_start: bool,
    pub power_up_punch_end: bool,
    pub unk_tile_change: bool,
    pub hay_cart_related: bool,
    pub acid_related_damage: bool,
    pub unk_3: bool,
    pub acid_damage: bool,
}

impl PacketFlag {
    fn from_u32(value: u32) -> Self {
        Self {
            none: value & 0x0 != 0,
            walk: value & 0x1 != 0,
            unk_2: value & 0x2 != 0,
            spawn_related: value & 0x4 != 0,
            extended: value & 0x8 != 0,
            facing_left: value & 0x10 != 0,
            standing: value & 0x20 != 0,
            fire_damage: value & 0x40 != 0,
            jump: value & 0x80 != 0,
            got_killed: value & 0x100 != 0,
            punch: value & 0x200 != 0,
            place: value & 0x400 != 0,
            tile_change: value & 0x800 != 0,
            got_punched: value & 0x1000 != 0,
            respawn: value & 0x2000 != 0,
            object_collect: value & 0x4000 != 0,
            trampoline: value & 0x8000 != 0,
            damage: value & 0x10000 != 0,
            slide: value & 0x20000 != 0,
            parasol: value & 0x40000 != 0,
            unk_gravity_related: value & 0x80000 != 0,
            swim: value & 0x100000 != 0,
            wall_hang: value & 0x200000 != 0,
            power_up_punch_start: value & 0x400000 != 0,
            power_up_punch_end: value & 0x800000 != 0,
            unk_tile_change: value & 0x1000000 != 0,
            hay_cart_related: value & 0x2000000 != 0,
            acid_related_damage: value & 0x4000000 != 0,
            unk_3: value & 0x8000000 != 0,
            acid_damage: value & 0x10000000 != 0,
        }
    }

    fn to_u32(&self) -> u32 {
        let mut value = 0;
        if self.walk {
            value |= 0x1;
        }
        if self.unk_2 {
            value |= 0x2;
        }
        if self.spawn_related {
            value |= 0x4;
        }
        if self.extended {
            value |= 0x8;
        }
        if self.facing_left {
            value |= 0x10;
        }
        if self.standing {
            value |= 0x20;
        }
        if self.fire_damage {
            value |= 0x40;
        }
        if self.jump {
            value |= 0x80;
        }
        if self.got_killed {
            value |= 0x100;
        }
        if self.punch {
            value |= 0x200;
        }
        if self.place {
            value |= 0x400;
        }
        if self.tile_change {
            value |= 0x800;
        }
        if self.got_punched {
            value |= 0x1000;
        }
        if self.respawn {
            value |= 0x2000;
        }
        if self.object_collect {
            value |= 0x4000;
        }
        if self.trampoline {
            value |= 0x8000;
        }
        if self.damage {
            value |= 0x10000;
        }
        if self.slide {
            value |= 0x20000;
        }
        if self.parasol {
            value |= 0x40000;
        }
        if self.unk_gravity_related {
            value |= 0x80000;
        }
        if self.swim {
            value |= 0x100000;
        }
        if self.wall_hang {
            value |= 0x200000;
        }
        if self.power_up_punch_start {
            value |= 0x400000;
        }
        if self.power_up_punch_end {
            value |= 0x800000;
        }
        if self.unk_tile_change {
            value |= 0x1000000;
        }
        if self.hay_cart_related {
            value |= 0x2000000;
        }
        if self.acid_related_damage {
            value |= 0x4000000;
        }
        if self.unk_3 {
            value |= 0x8000000;
        }
        if self.acid_damage {
            value |= 0x10000000;
        }
        value
    }
}

impl Serialize for PacketFlag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.to_u32())
    }
}

impl<'de> Deserialize<'de> for PacketFlag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Ok(PacketFlag::from_u32(value))
    }
}