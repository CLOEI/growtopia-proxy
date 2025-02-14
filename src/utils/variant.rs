use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use serde::Deserialize;
use serde::Serialize;
use std::io::prelude::*;
use std::io::Cursor;

#[derive(Debug, Clone, Copy, PartialEq)]
enum VariantType {
    Unknown,
    Float,
    String,
    Vec2,
    Vec3,
    Unsigned,
    Signed,
}

impl From<u8> for VariantType {
    fn from(value: u8) -> Self {
        match value {
            0 => VariantType::Unknown,
            1 => VariantType::Float,
            2 => VariantType::String,
            3 => VariantType::Vec2,
            4 => VariantType::Vec3,
            5 => VariantType::Unsigned,
            9 => VariantType::Signed,
            _ => VariantType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Variant {
    Float(f32),
    String(String),
    Vec2((f32, f32)),
    Vec3((f32, f32, f32)),
    Unsigned(u32),
    Signed(i32),
    Unknown,
}

impl Variant {
    pub fn as_string(&self) -> String {
        match self {
            Variant::Float(value) => value.to_string(),
            Variant::String(value) => value.clone(),
            Variant::Vec2((x, y)) => format!("{}, {}", x, y),
            Variant::Vec3((x, y, z)) => format!("{}, {}, {}", x, y, z),
            Variant::Unsigned(value) => value.to_string(),
            Variant::Signed(value) => value.to_string(),
            Variant::Unknown => "Unknown".to_string(),
        }
    }

    pub fn as_int32(&self) -> i32 {
        match self {
            Variant::Signed(value) => *value,
            _ => 0,
        }
    }

    pub fn as_vec2(&self) -> (f32, f32) {
        match self {
            Variant::Vec2(value) => *value,
            _ => (0.0, 0.0),
        }
    }

    pub fn as_uint32(&self) -> u32 {
        match self {
            Variant::Unsigned(value) => *value,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantList {
    variants: Vec<Variant>,
}

impl VariantList {
    pub fn deserialize(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut cursor = Cursor::new(data);
        let size = cursor.read_u8()?;
        let mut variants = Vec::with_capacity(size as usize);

        for _ in 0..size {
            let _index = cursor.read_u8()?;
            let var_type: VariantType = cursor.read_u8()?.into();

            let variant = match var_type {
                VariantType::Float => {
                    let value = cursor.read_f32::<LittleEndian>()?;
                    Variant::Float(value)
                }
                VariantType::String => {
                    let len = cursor.read_u32::<LittleEndian>()? as usize;
                    let mut buffer = vec![0; len];
                    cursor.read_exact(&mut buffer)?;
                    let value = String::from_utf8(buffer).unwrap();
                    Variant::String(value)
                }
                VariantType::Vec2 => {
                    let x = cursor.read_f32::<LittleEndian>()?;
                    let y = cursor.read_f32::<LittleEndian>()?;
                    Variant::Vec2((x, y))
                }
                VariantType::Vec3 => {
                    let x = cursor.read_f32::<LittleEndian>()?;
                    let y = cursor.read_f32::<LittleEndian>()?;
                    let z = cursor.read_f32::<LittleEndian>()?;
                    Variant::Vec3((x, y, z))
                }
                VariantType::Unsigned => {
                    let value = cursor.read_u32::<LittleEndian>()?;
                    Variant::Unsigned(value)
                }
                VariantType::Signed => {
                    let value = cursor.read_i32::<LittleEndian>()?;
                    Variant::Signed(value)
                }
                VariantType::Unknown => Variant::Unknown,
            };

            variants.push(variant);
        }

        Ok(Self { variants })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut mem_needed = 0;
        let mut vars_used = 0;

        for variant in &self.variants {
            let var_size = match variant {
                Variant::String(s) => s.len() + 4,
                Variant::Float(_) => 4,
                Variant::Vec2(_) => 8,
                Variant::Vec3(_) => 12,
                Variant::Unsigned(_) => 4,
                Variant::Signed(_) => 4,
                Variant::Unknown => 0,
            };
            if var_size > 0 {
                vars_used += 1;
                mem_needed += var_size;
            }
        }

        let total = mem_needed + 1 + (vars_used * 2);
        let mut data = vec![0u8; total];
        let mut p = &mut data[..];

        p[0] = vars_used as u8;
        p = &mut p[1..];

        for (idx, variant) in self.variants.iter().enumerate() {
            let var_size = match variant {
                Variant::String(s) => s.len() + 4,
                Variant::Float(_) => 4,
                Variant::Vec2(_) => 8,
                Variant::Vec3(_) => 12,
                Variant::Unsigned(_) => 4,
                Variant::Signed(_) => 4,
                Variant::Unknown => 0,
            };
            if var_size > 0 {
                p[0] = idx as u8;
                p[1] = match variant {
                    Variant::Float(_) => 1,
                    Variant::String(_) => 2,
                    Variant::Vec2(_) => 3,
                    Variant::Vec3(_) => 4,
                    Variant::Unsigned(_) => 5,
                    Variant::Signed(_) => 9,
                    Variant::Unknown => 0,
                };
                p = &mut p[2..];

                match variant {
                    Variant::Float(value) => {
                        LittleEndian::write_f32(&mut p[..4], *value);
                        p = &mut p[4..];
                    }
                    Variant::String(value) => {
                        let len = value.len() as u32;
                        LittleEndian::write_u32(&mut p[..4], len);
                        p = &mut p[4..];
                        p[..len as usize].copy_from_slice(value.as_bytes());
                        p = &mut p[len as usize..];
                    }
                    Variant::Vec2((x, y)) => {
                        LittleEndian::write_f32(&mut p[..4], *x);
                        LittleEndian::write_f32(&mut p[4..8], *y);
                        p = &mut p[8..];
                    }
                    Variant::Vec3((x, y, z)) => {
                        LittleEndian::write_f32(&mut p[..4], *x);
                        LittleEndian::write_f32(&mut p[4..8], *y);
                        LittleEndian::write_f32(&mut p[8..12], *z);
                        p = &mut p[12..];
                    }
                    Variant::Unsigned(value) => {
                        LittleEndian::write_u32(&mut p[..4], *value);
                        p = &mut p[4..];
                    }
                    Variant::Signed(value) => {
                        LittleEndian::write_i32(&mut p[..4], *value);
                        p = &mut p[4..];
                    }
                    Variant::Unknown => {}
                }
            }
        }

        data
    }

    pub fn get(&self, index: usize) -> Option<&Variant> {
        self.variants.get(index)
    }
}