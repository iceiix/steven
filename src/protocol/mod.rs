// Copyright 2016 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code)]

use aes::Aes128;
use cfb8::Cfb8;
use cfb8::stream_cipher::{NewStreamCipher, StreamCipher};
use serde_json;
use reqwest;

pub mod mojang;

use crate::nbt;
use crate::format;
use std::fmt;
use std::default;
use std::net::TcpStream;
use std::io;
use std::io::{Write, Read};
use std::convert;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use flate2::read::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use std::time::{Instant, Duration};
use crate::shared::Position;

pub const SUPPORTED_PROTOCOL: i32 = 340;


/// Helper macro for defining packets
#[macro_export]
macro_rules! state_packets {
     ($($state:ident $stateName:ident {
        $($dir:ident $dirName:ident {
            $(
                $(#[$attr:meta])*
                packet $name:ident {
                    $($(#[$fattr:meta])*field $field:ident: $field_type:ty = $(when ($cond:expr))*, )+
                }
            )*
        })+
    })+) => {
        use crate::protocol::*;
        use std::io;

        #[derive(Debug)]
        pub enum Packet {
        $(
            $(
                $(
        $name($state::$dir::$name),
                )*
            )+
        )+
        }

        $(
        pub mod $state {

            $(
            pub mod $dir {
                #![allow(unused_imports)]
                use crate::protocol::*;
                use std::io;
                use crate::format;
                use crate::nbt;
                use crate::types;
                use crate::item;
                use crate::shared::Position;


                #[allow(non_upper_case_globals)]
                pub mod internal_ids {
                    create_ids!(i32, $($name),*);
                }

                $(
                    #[derive(Default, Debug)]
                    $(#[$attr])* pub struct $name {
                        $($(#[$fattr])* pub $field: $field_type),+,
                    }

                    impl PacketType for $name {

                        fn packet_id(&self) -> i32 { internal_ids::$name }

                        fn write<W: io::Write>(self, buf: &mut W) -> Result<(), Error> {
                            $(
                                if true $(&& ($cond(&self)))* {
                                    self.$field.write_to(buf)?;
                                }
                            )+

                            Result::Ok(())
                        }
                    }
                )*
            }
            )+
        }
        )+

        /// Returns the packet for the given state, direction and id after parsing the fields
        /// from the buffer.
        pub fn packet_by_id<R: io::Read>(state: State, dir: Direction, id: i32, mut buf: &mut R) -> Result<Option<Packet>, Error> {
            match state {
                $(
                    State::$stateName => {
                        match dir {
                            $(
                                Direction::$dirName => {
                                    let internal_id = packet::translate_packet_id(state, dir, id);
                                    match internal_id {
                                    $(
                                        self::$state::$dir::internal_ids::$name => {
                                            use self::$state::$dir::$name;
                                            let mut packet : $name = $name::default();
                                            $(
                                                if true $(&& ($cond(&packet)))* {
                                                    packet.$field = Serializable::read_from(&mut buf)?;
                                                }
                                            )+
                                            Result::Ok(Option::Some(Packet::$name(packet)))
                                        },
                                    )*
                                        _ => Result::Ok(Option::None)
                                    }
                                }
                            )+
                        }
                    }
                )+
            }
        }
    }
}

pub mod packet;

pub fn translate_packet_id(state: State, dir: Direction, id: i32) -> i32 {
    match state {
        State::Handshaking => match dir {
            Direction::Serverbound => match id {
                0x00 => crate::protocol::packet::handshake::serverbound::internal_ids::Handshake,
                _ => panic!("bad packet id in Handshake Serverbound"),
            }
            Direction::Clientbound => panic!("unexpected Clientbound in Handshake"),
        }
        State::Play => match dir {
            Direction::Serverbound => match id {
                0x00 => crate::protocol::packet::play::serverbound::internal_ids::TeleportConfirm,
                0x01 => crate::protocol::packet::play::serverbound::internal_ids::TabComplete,
                0x02 => crate::protocol::packet::play::serverbound::internal_ids::ChatMessage,
                0x03 => crate::protocol::packet::play::serverbound::internal_ids::ClientStatus,
                0x04 => crate::protocol::packet::play::serverbound::internal_ids::ClientSettings,
                0x05 => crate::protocol::packet::play::serverbound::internal_ids::ConfirmTransactionServerbound,
                0x06 => crate::protocol::packet::play::serverbound::internal_ids::EnchantItem,
                0x07 => crate::protocol::packet::play::serverbound::internal_ids::ClickWindow,
                0x08 => crate::protocol::packet::play::serverbound::internal_ids::CloseWindow,
                0x09 => crate::protocol::packet::play::serverbound::internal_ids::PluginMessageServerbound,
                0x0a => crate::protocol::packet::play::serverbound::internal_ids::UseEntity,
                0x0b => crate::protocol::packet::play::serverbound::internal_ids::KeepAliveServerbound,
                0x0c => crate::protocol::packet::play::serverbound::internal_ids::Player,
                0x0d => crate::protocol::packet::play::serverbound::internal_ids::PlayerPosition,
                0x0e => crate::protocol::packet::play::serverbound::internal_ids::PlayerPositionLook,
                0x0f => crate::protocol::packet::play::serverbound::internal_ids::PlayerLook,
                0x10 => crate::protocol::packet::play::serverbound::internal_ids::VehicleMove,
                0x11 => crate::protocol::packet::play::serverbound::internal_ids::SteerBoat,
                0x12 => crate::protocol::packet::play::serverbound::internal_ids::CraftRecipeRequest,
                0x13 => crate::protocol::packet::play::serverbound::internal_ids::ClientAbilities,
                0x14 => crate::protocol::packet::play::serverbound::internal_ids::PlayerDigging,
                0x15 => crate::protocol::packet::play::serverbound::internal_ids::PlayerAction,
                0x16 => crate::protocol::packet::play::serverbound::internal_ids::SteerVehicle,
                0x17 => crate::protocol::packet::play::serverbound::internal_ids::CraftingBookData,
                0x18 => crate::protocol::packet::play::serverbound::internal_ids::ResourcePackStatus,
                0x19 => crate::protocol::packet::play::serverbound::internal_ids::AdvancementTab,
                0x1a => crate::protocol::packet::play::serverbound::internal_ids::HeldItemChange,
                0x1b => crate::protocol::packet::play::serverbound::internal_ids::CreativeInventoryAction,
                0x1c => crate::protocol::packet::play::serverbound::internal_ids::SetSign,
                0x1d => crate::protocol::packet::play::serverbound::internal_ids::ArmSwing,
                0x1e => crate::protocol::packet::play::serverbound::internal_ids::SpectateTeleport,
                0x1f => crate::protocol::packet::play::serverbound::internal_ids::PlayerBlockPlacement,
                0x20 => crate::protocol::packet::play::serverbound::internal_ids::UseItem,
                _ => panic!("bad packet id in Play Serverbound"),
            }
            Direction::Clientbound => match id {
                0x00 => crate::protocol::packet::play::clientbound::internal_ids::SpawnObject,
                0x01 => crate::protocol::packet::play::clientbound::internal_ids::SpawnExperienceOrb,
                0x02 => crate::protocol::packet::play::clientbound::internal_ids::SpawnGlobalEntity,
                0x03 => crate::protocol::packet::play::clientbound::internal_ids::SpawnMob,
                0x04 => crate::protocol::packet::play::clientbound::internal_ids::SpawnPainting,
                0x05 => crate::protocol::packet::play::clientbound::internal_ids::SpawnPlayer,
                0x06 => crate::protocol::packet::play::clientbound::internal_ids::Animation,
                0x07 => crate::protocol::packet::play::clientbound::internal_ids::Statistics,
                0x08 => crate::protocol::packet::play::clientbound::internal_ids::BlockBreakAnimation,
                0x09 => crate::protocol::packet::play::clientbound::internal_ids::UpdateBlockEntity,
                0x0a => crate::protocol::packet::play::clientbound::internal_ids::BlockAction,
                0x0b => crate::protocol::packet::play::clientbound::internal_ids::BlockChange,
                0x0c => crate::protocol::packet::play::clientbound::internal_ids::BossBar,
                0x0d => crate::protocol::packet::play::clientbound::internal_ids::ServerDifficulty,
                0x0e => crate::protocol::packet::play::clientbound::internal_ids::TabCompleteReply,
                0x0f => crate::protocol::packet::play::clientbound::internal_ids::ServerMessage,
                0x10 => crate::protocol::packet::play::clientbound::internal_ids::MultiBlockChange,
                0x11 => crate::protocol::packet::play::clientbound::internal_ids::ConfirmTransaction,
                0x12 => crate::protocol::packet::play::clientbound::internal_ids::WindowClose,
                0x13 => crate::protocol::packet::play::clientbound::internal_ids::WindowOpen,
                0x14 => crate::protocol::packet::play::clientbound::internal_ids::WindowItems,
                0x15 => crate::protocol::packet::play::clientbound::internal_ids::WindowProperty,
                0x16 => crate::protocol::packet::play::clientbound::internal_ids::WindowSetSlot,
                0x17 => crate::protocol::packet::play::clientbound::internal_ids::SetCooldown,
                0x18 => crate::protocol::packet::play::clientbound::internal_ids::PluginMessageClientbound,
                0x19 => crate::protocol::packet::play::clientbound::internal_ids::NamedSoundEffect,
                0x1a => crate::protocol::packet::play::clientbound::internal_ids::Disconnect,
                0x1b => crate::protocol::packet::play::clientbound::internal_ids::EntityAction,
                0x1c => crate::protocol::packet::play::clientbound::internal_ids::Explosion,
                0x1d => crate::protocol::packet::play::clientbound::internal_ids::ChunkUnload,
                0x1e => crate::protocol::packet::play::clientbound::internal_ids::ChangeGameState,
                0x1f => crate::protocol::packet::play::clientbound::internal_ids::KeepAliveClientbound,
                0x20 => crate::protocol::packet::play::clientbound::internal_ids::ChunkData,
                0x21 => crate::protocol::packet::play::clientbound::internal_ids::Effect,
                0x22 => crate::protocol::packet::play::clientbound::internal_ids::Particle,
                0x23 => crate::protocol::packet::play::clientbound::internal_ids::JoinGame,
                0x24 => crate::protocol::packet::play::clientbound::internal_ids::Maps,
                0x25 => crate::protocol::packet::play::clientbound::internal_ids::Entity,
                0x26 => crate::protocol::packet::play::clientbound::internal_ids::EntityMove,
                0x27 => crate::protocol::packet::play::clientbound::internal_ids::EntityLookAndMove,
                0x28 => crate::protocol::packet::play::clientbound::internal_ids::EntityLook,
                0x29 => crate::protocol::packet::play::clientbound::internal_ids::VehicleTeleport,
                0x2a => crate::protocol::packet::play::clientbound::internal_ids::SignEditorOpen,
                0x2b => crate::protocol::packet::play::clientbound::internal_ids::CraftRecipeResponse,
                0x2c => crate::protocol::packet::play::clientbound::internal_ids::PlayerAbilities,
                0x2d => crate::protocol::packet::play::clientbound::internal_ids::CombatEvent,
                0x2e => crate::protocol::packet::play::clientbound::internal_ids::PlayerInfo,
                0x2f => crate::protocol::packet::play::clientbound::internal_ids::TeleportPlayer,
                0x30 => crate::protocol::packet::play::clientbound::internal_ids::EntityUsedBed,
                0x31 => crate::protocol::packet::play::clientbound::internal_ids::UnlockRecipes,
                0x32 => crate::protocol::packet::play::clientbound::internal_ids::EntityDestroy,
                0x33 => crate::protocol::packet::play::clientbound::internal_ids::EntityRemoveEffect,
                0x34 => crate::protocol::packet::play::clientbound::internal_ids::ResourcePackSend,
                0x35 => crate::protocol::packet::play::clientbound::internal_ids::Respawn,
                0x36 => crate::protocol::packet::play::clientbound::internal_ids::EntityHeadLook,
                0x37 => crate::protocol::packet::play::clientbound::internal_ids::SelectAdvancementTab,
                0x38 => crate::protocol::packet::play::clientbound::internal_ids::WorldBorder,
                0x39 => crate::protocol::packet::play::clientbound::internal_ids::Camera,
                0x3a => crate::protocol::packet::play::clientbound::internal_ids::SetCurrentHotbarSlot,
                0x3b => crate::protocol::packet::play::clientbound::internal_ids::ScoreboardDisplay,
                0x3c => crate::protocol::packet::play::clientbound::internal_ids::EntityMetadata,
                0x3d => crate::protocol::packet::play::clientbound::internal_ids::EntityAttach,
                0x3e => crate::protocol::packet::play::clientbound::internal_ids::EntityVelocity,
                0x3f => crate::protocol::packet::play::clientbound::internal_ids::EntityEquipment,
                0x40 => crate::protocol::packet::play::clientbound::internal_ids::SetExperience,
                0x41 => crate::protocol::packet::play::clientbound::internal_ids::UpdateHealth,
                0x42 => crate::protocol::packet::play::clientbound::internal_ids::ScoreboardObjective,
                0x43 => crate::protocol::packet::play::clientbound::internal_ids::SetPassengers,
                0x44 => crate::protocol::packet::play::clientbound::internal_ids::Teams,
                0x45 => crate::protocol::packet::play::clientbound::internal_ids::UpdateScore,
                0x46 => crate::protocol::packet::play::clientbound::internal_ids::SpawnPosition,
                0x47 => crate::protocol::packet::play::clientbound::internal_ids::TimeUpdate,
                0x48 => crate::protocol::packet::play::clientbound::internal_ids::Title,
                0x49 => crate::protocol::packet::play::clientbound::internal_ids::SoundEffect,
                0x4a => crate::protocol::packet::play::clientbound::internal_ids::PlayerListHeaderFooter,
                0x4b => crate::protocol::packet::play::clientbound::internal_ids::CollectItem,
                0x4c => crate::protocol::packet::play::clientbound::internal_ids::EntityTeleport,
                0x4d => crate::protocol::packet::play::clientbound::internal_ids::Advancements,
                0x4e => crate::protocol::packet::play::clientbound::internal_ids::EntityProperties,
                0x4f => crate::protocol::packet::play::clientbound::internal_ids::EntityEffect,
                _ => panic!("bad packet id in Play Clientbound"),
            }
        }
        State::Login => match dir {
            Direction::Serverbound => match id {
                0x00 => crate::protocol::packet::login::serverbound::internal_ids::LoginStart,
                0x01 => crate::protocol::packet::login::serverbound::internal_ids::EncryptionResponse,
                _ => panic!("bad packet id in Login Serverbound"),
            }
            Direction::Clientbound => match id {
                0x00 => crate::protocol::packet::login::clientbound::internal_ids::LoginDisconnect,
                0x01 => crate::protocol::packet::login::clientbound::internal_ids::EncryptionRequest,
                0x02 => crate::protocol::packet::login::clientbound::internal_ids::LoginSuccess,
                0x03 => crate::protocol::packet::login::clientbound::internal_ids::SetInitialCompression,
                _ => panic!("bad packet id in Login Clientbound"),
            }
        }
        State::Status => match dir {
            Direction::Serverbound => match id {
                0x00 => crate::protocol::packet::status::serverbound::internal_ids::StatusRequest,
                0x01 => crate::protocol::packet::status::serverbound::internal_ids::StatusPing,
                _ => panic!("bad packet id in Status Serverbound"),
            }
            Direction::Clientbound => match id {
                0x00 => crate::protocol::packet::status::clientbound::internal_ids::StatusResponse,
                0x01 => crate::protocol::packet::status::clientbound::internal_ids::StatusPong,
                _ => panic!("bad packet id in Status Clientbound"),
            }
        }
    }
}

pub trait Serializable: Sized {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error>;
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error>;
}

impl Serializable for Vec<u8> {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Vec<u8>, Error> {
        let mut v = Vec::new();
        buf.read_to_end(&mut v)?;
        Ok(v)
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_all(&self[..]).map_err(|v| v.into())
    }
}

impl Serializable for Option<nbt::NamedTag>{
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Option<nbt::NamedTag>, Error> {
        let ty = buf.read_u8()?;
        if ty == 0 {
            Result::Ok(None)
        } else {
            let name = nbt::read_string(buf)?;
            let tag = nbt::Tag::read_from(buf)?;
            Result::Ok(Some(nbt::NamedTag(name, tag)))
        }
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        match *self {
            Some(ref val) => {
                buf.write_u8(10)?;
                nbt::write_string(buf, &val.0)?;
                val.1.write_to(buf)?;
            }
            None => buf.write_u8(0)?,
        }
        Result::Ok(())
    }
}

impl <T> Serializable for Option<T> where T : Serializable {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Option<T>, Error> {
        Result::Ok(Some(T::read_from(buf)?))
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        if self.is_some() {
            self.as_ref().unwrap().write_to(buf)?;
        }
        Result::Ok(())
    }
}

impl Serializable for String {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<String, Error> {
        let len = VarInt::read_from(buf)?.0;
        debug_assert!(len >= 0, "Negative string length: {}", len);
        debug_assert!(len <= 65536, "String length too big: {}", len);
        let mut ret = String::new();
        buf.take(len as u64).read_to_string(&mut ret)?;
        Result::Ok(ret)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        let bytes = self.as_bytes();
        VarInt(bytes.len() as i32).write_to(buf)?;
        buf.write_all(bytes)?;
        Result::Ok(())
    }
}

impl Serializable for format::Component {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Self, Error> {
        let len = VarInt::read_from(buf)?.0;
        let mut ret = String::new();
        buf.take(len as u64).read_to_string(&mut ret)?;
        let val: serde_json::Value = serde_json::from_str(&ret[..]).unwrap();
        Result::Ok(Self::from_value(&val))
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        let val = serde_json::to_string(&self.to_value()).unwrap();
        let bytes = val.as_bytes();
        VarInt(bytes.len() as i32).write_to(buf)?;
        buf.write_all(bytes)?;
        Result::Ok(())
    }
}

impl Serializable for () {
    fn read_from<R: io::Read>(_: &mut R) -> Result<(), Error> {
        Result::Ok(())
    }
    fn write_to<W: io::Write>(&self, _: &mut W) -> Result<(), Error> {
        Result::Ok(())
    }
}

impl Serializable for bool {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<bool, Error> {
        Result::Ok(buf.read_u8()? != 0)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_u8(if *self {
            1
        } else {
            0
        })?;
        Result::Ok(())
    }
}

impl Serializable for i8 {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<i8, Error> {
        Result::Ok(buf.read_i8()?)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_i8(*self)?;
        Result::Ok(())
    }
}

impl Serializable for i16 {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<i16, Error> {
        Result::Ok(buf.read_i16::<BigEndian>()?)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_i16::<BigEndian>(*self)?;
        Result::Ok(())
    }
}

impl Serializable for i32 {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<i32, Error> {
        Result::Ok(buf.read_i32::<BigEndian>()?)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_i32::<BigEndian>(*self)?;
        Result::Ok(())
    }
}

impl Serializable for i64 {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<i64, Error> {
        Result::Ok(buf.read_i64::<BigEndian>()?)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_i64::<BigEndian>(*self)?;
        Result::Ok(())
    }
}

impl Serializable for u8 {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<u8, Error> {
        Result::Ok(buf.read_u8()?)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_u8(*self)?;
        Result::Ok(())
    }
}

impl Serializable for u16 {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<u16, Error> {
        Result::Ok(buf.read_u16::<BigEndian>()?)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_u16::<BigEndian>(*self)?;
        Result::Ok(())
    }
}

impl Serializable for u64 {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<u64, Error> {
        Result::Ok(buf.read_u64::<BigEndian>()?)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_u64::<BigEndian>(*self)?;
        Result::Ok(())
    }
}

impl Serializable for f32 {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<f32, Error> {
        Result::Ok(buf.read_f32::<BigEndian>()?)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_f32::<BigEndian>(*self)?;
        Result::Ok(())
    }
}

impl Serializable for f64 {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<f64, Error> {
        Result::Ok(buf.read_f64::<BigEndian>()?)
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_f64::<BigEndian>(*self)?;
        Result::Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct UUID(u64, u64);

impl UUID {
    pub fn from_str(s: &str) -> UUID {
        use hex;
        // TODO: Panics aren't the best idea here
        if s.len() != 36 {
            panic!("Invalid UUID format");
        }
        let mut parts = hex::decode(&s[..8]).unwrap();
        parts.extend_from_slice(&hex::decode(&s[9..13]).unwrap());
        parts.extend_from_slice(&hex::decode(&s[14..18]).unwrap());
        parts.extend_from_slice(&hex::decode(&s[19..23]).unwrap());
        parts.extend_from_slice(&hex::decode(&s[24..36]).unwrap());
        let mut high = 0u64;
        let mut low = 0u64;
        for i in 0 .. 8 {
            high |= (parts[i] as u64) << (56 - i*8);
            low |= (parts[i + 8] as u64) << (56 - i*8);
        }
        UUID(high, low)
    }
}

impl Default for UUID {
    fn default() -> Self {
        UUID(0, 0)
    }
}

impl Serializable for UUID {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<UUID, Error> {
        Result::Ok(UUID(buf.read_u64::<BigEndian>()?,
                        buf.read_u64::<BigEndian>()?))
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        buf.write_u64::<BigEndian>(self.0)?;
        buf.write_u64::<BigEndian>(self.1)?;
        Result::Ok(())
    }
}


pub trait Lengthable : Serializable + Copy + Default {
    fn into(self) -> usize;
    fn from(_: usize) -> Self;
}

pub struct LenPrefixed<L: Lengthable, V> {
    len: L,
    pub data: Vec<V>,
}

impl <L: Lengthable, V: Default>  LenPrefixed<L, V> {
    pub fn new(data: Vec<V>) -> LenPrefixed<L, V> {
        LenPrefixed {
            len: Default::default(),
            data,
        }
    }
}

impl <L: Lengthable, V: Serializable>  Serializable for LenPrefixed<L, V> {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<LenPrefixed<L, V>, Error> {
        let len_data: L = Serializable::read_from(buf)?;
        let len: usize = len_data.into();
        let mut data: Vec<V> = Vec::with_capacity(len);
        for _ in 0..len {
            data.push(Serializable::read_from(buf)?);
        }
        Result::Ok(LenPrefixed {
            len: len_data,
            data,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        let len_data: L = L::from(self.data.len());
        len_data.write_to(buf)?;
        let data = &self.data;
        for val in data {
            val.write_to(buf)?;
        }
        Result::Ok(())
    }
}


impl <L: Lengthable, V: Default> Default for LenPrefixed<L, V> {
    fn default() -> Self {
        LenPrefixed {
            len: default::Default::default(),
            data: default::Default::default(),
        }
    }
}

impl <L: Lengthable, V: fmt::Debug> fmt::Debug for LenPrefixed<L, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)
    }
}

// Optimization
pub struct LenPrefixedBytes<L: Lengthable> {
    len: L,
    pub data: Vec<u8>,
}

impl <L: Lengthable>  LenPrefixedBytes<L> {
    pub fn new(data: Vec<u8>) -> LenPrefixedBytes<L> {
        LenPrefixedBytes {
            len: Default::default(),
            data,
        }
    }
}

impl <L: Lengthable>  Serializable for LenPrefixedBytes<L> {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<LenPrefixedBytes<L>, Error> {
        let len_data: L = Serializable::read_from(buf)?;
        let len: usize = len_data.into();
        let mut data: Vec<u8> = Vec::with_capacity(len);
        buf.take(len as u64).read_to_end(&mut data)?;
        Result::Ok(LenPrefixedBytes {
            len: len_data,
            data,
        })
    }

    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        let len_data: L = L::from(self.data.len());
        len_data.write_to(buf)?;
        buf.write_all(&self.data[..])?;
        Result::Ok(())
    }
}


impl <L: Lengthable> Default for LenPrefixedBytes<L> {
    fn default() -> Self {
        LenPrefixedBytes {
            len: default::Default::default(),
            data: default::Default::default(),
        }
    }
}

impl <L: Lengthable> fmt::Debug for LenPrefixedBytes<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)
    }
}

impl Lengthable for i16 {
    fn into(self) -> usize {
        self as usize
    }

    fn from(u: usize) -> i16 {
        u as i16
    }
}

impl Lengthable for i32 {
    fn into(self) -> usize {
        self as usize
    }

    fn from(u: usize) -> i32 {
        u as i32
    }
}

/// `VarInt` have a variable size (between 1 and 5 bytes) when encoded based
/// on the size of the number
#[derive(Clone, Copy)]
pub struct VarInt(pub i32);

impl Lengthable for VarInt {
    fn into(self) -> usize {
        self.0 as usize
    }

    fn from(u: usize) -> VarInt {
        VarInt(u as i32)
    }
}

impl Serializable for VarInt {
    /// Decodes a `VarInt` from the Reader
    fn read_from<R: io::Read>(buf: &mut R) -> Result<VarInt, Error> {
        const PART : u32 = 0x7F;
        let mut size = 0;
        let mut val = 0u32;
        loop {
            let b = buf.read_u8()? as u32;
            val |= (b & PART) << (size * 7);
            size += 1;
            if size > 5 {
                return Result::Err(Error::Err("VarInt too big".to_owned()));
            }
            if (b & 0x80) == 0 {
                break
            }
        }

        Result::Ok(VarInt(val as i32))
    }

    /// Encodes a `VarInt` into the Writer
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        const PART : u32 = 0x7F;
        let mut val = self.0 as u32;
        loop {
            if (val & !PART) == 0 {
                buf.write_u8(val as u8)?;
                return Result::Ok(());
            }
            buf.write_u8(((val & PART) | 0x80) as u8)?;
            val >>= 7;
        }
    }
}

impl default::Default for VarInt {
    fn default() -> VarInt {
        VarInt(0)
    }
}

impl fmt::Debug for VarInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// `VarLong` have a variable size (between 1 and 10 bytes) when encoded based
/// on the size of the number
#[derive(Clone, Copy)]
pub struct VarLong(pub i64);

impl Lengthable for VarLong {
    fn into(self) -> usize {
        self.0 as usize
    }

    fn from(u: usize) -> VarLong {
        VarLong(u as i64)
    }
}

impl Serializable for VarLong {
    /// Decodes a `VarLong` from the Reader
    fn read_from<R: io::Read>(buf: &mut R) -> Result<VarLong, Error> {
        const PART : u64 = 0x7F;
        let mut size = 0;
        let mut val = 0u64;
        loop {
            let b = buf.read_u8()? as u64;
            val |= (b & PART) << (size * 7);
            size += 1;
            if size > 10 {
                return Result::Err(Error::Err("VarLong too big".to_owned()));
            }
            if (b & 0x80) == 0 {
                break
            }
        }

        Result::Ok(VarLong(val as i64))
    }

    /// Encodes a `VarLong` into the Writer
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        const PART : u64 = 0x7F;
        let mut val = self.0 as u64;
        loop {
            if (val & !PART) == 0 {
                buf.write_u8(val as u8)?;
                return Result::Ok(());
            }
            buf.write_u8(((val & PART) | 0x80) as u8)?;
            val >>= 7;
        }
    }
}

impl default::Default for VarLong {
    fn default() -> VarLong {
        VarLong(0)
    }
}

impl fmt::Debug for VarLong {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serializable for Position {
    fn read_from<R: io::Read>(buf: &mut R) -> Result<Position, Error> {
        let pos = buf.read_u64::<BigEndian>()?;
        Ok(Position::new(
            ((pos as i64) >> 38) as i32,
            (((pos as i64) >> 26) & 0xFFF) as i32,
            ((pos as i64) << 38 >> 38) as i32
        ))
    }
    fn write_to<W: io::Write>(&self, buf: &mut W) -> Result<(), Error> {
        let pos = (((self.x as u64) & 0x3FFFFFF) << 38)
            | (((self.y as u64) & 0xFFF) << 26)
            | ((self.z as u64) & 0x3FFFFFF);
        buf.write_u64::<BigEndian>(pos)?;
        Result::Ok(())
    }
}


/// Direction is used to define whether packets are going to the
/// server or the client.
#[derive(Clone, Copy)]
pub enum Direction {
    Serverbound,
    Clientbound,
}

/// The protocol has multiple 'sub-protocols' or states which control which
/// packet an id points to.
#[derive(Clone, Copy)]
pub enum State {
    Handshaking,
    Play,
    Status,
    Login,
}

/// Return for any protocol related error.
#[derive(Debug)]
pub enum Error {
    Err(String),
    Disconnect(format::Component),
    IOError(io::Error),
    Json(serde_json::Error),
    Reqwest(reqwest::Error),
}

impl convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IOError(e)
    }
}

impl convert::From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json(e)
    }
}

impl convert::From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error::Reqwest(e)
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Err(ref val) => &val[..],
            Error::Disconnect(_) => "Disconnect",
            Error::IOError(ref e) => e.description(),
            Error::Json(ref e) => e.description(),
            Error::Reqwest(ref e) => e.description(),
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Error::Err(ref val) => write!(f, "protocol error: {}", val),
            Error::Disconnect(ref val) => write!(f, "{}", val),
            Error::IOError(ref e) => e.fmt(f),
            Error::Json(ref e) => e.fmt(f),
            Error::Reqwest(ref e) => e.fmt(f),
        }
    }
}

type Aes128Cfb = Cfb8<Aes128>;

pub struct Conn {
    stream: TcpStream,
    pub host: String,
    pub port: u16,
    direction: Direction,
    pub state: State,

    cipher: Option<Aes128Cfb>,

    compression_threshold: i32,
    compression_read: Option<ZlibDecoder<io::Cursor<Vec<u8>>>>,
    compression_write: Option<ZlibEncoder<io::Cursor<Vec<u8>>>>,
}

impl Conn {
    pub fn new(target: &str) -> Result<Conn, Error> {
        // TODO SRV record support
        let mut parts = target.split(':').collect::<Vec<&str>>();
        let address = if parts.len() == 1 {
            parts.push("25565");
            format!("{}:25565", parts[0])
        } else {
            format!("{}:{}", parts[0], parts[1])
        };
        let stream = TcpStream::connect(&*address)?;
        Result::Ok(Conn {
            stream,
            host: parts[0].to_owned(),
            port: parts[1].parse().unwrap(),
            direction: Direction::Serverbound,
            state: State::Handshaking,
            cipher: Option::None,
            compression_threshold: -1,
            compression_read: Option::None,
            compression_write: Option::None,
        })
    }

    pub fn write_packet<T: PacketType>(&mut self, packet: T) -> Result<(), Error> {
        let mut buf = Vec::new();
        VarInt(packet.packet_id()).write_to(&mut buf)?;
        packet.write(&mut buf)?;

        let mut extra = if self.compression_threshold >= 0 {
            1
        } else {
            0
        };
        if self.compression_threshold >= 0 && buf.len() as i32 > self.compression_threshold {
            if self.compression_write.is_none() {
                self.compression_write = Some(ZlibEncoder::new(io::Cursor::new(Vec::new()), Compression::default()));
            }
            extra = 0;
            let uncompressed_size = buf.len();
            let mut new = Vec::new();
            VarInt(uncompressed_size as i32).write_to(&mut new)?;
            let write = self.compression_write.as_mut().unwrap();
            write.reset(io::Cursor::new(buf));
            write.read_to_end(&mut new)?;
            buf = new;
        }

        VarInt(buf.len() as i32 + extra).write_to(self)?;
        if self.compression_threshold >= 0 && extra == 1 {
            VarInt(0).write_to(self)?;
        }
        self.write_all(&buf)?;

        Result::Ok(())
    }

    pub fn read_packet(&mut self) -> Result<packet::Packet, Error> {
        let len = VarInt::read_from(self)?.0 as usize;
        let mut ibuf = vec![0; len];
        self.read_exact(&mut ibuf)?;

        let mut buf = io::Cursor::new(ibuf);

        if self.compression_threshold >= 0 {
            if self.compression_read.is_none() {
                self.compression_read = Some(ZlibDecoder::new(io::Cursor::new(Vec::new())));
            }
            let uncompressed_size = VarInt::read_from(&mut buf)?.0;
            if uncompressed_size != 0 {
                let mut new = Vec::with_capacity(uncompressed_size as usize);
                {
                    let reader = self.compression_read.as_mut().unwrap();
                    reader.reset(buf);
                    reader.read_to_end(&mut new)?;
                }
                buf = io::Cursor::new(new);
            }
        }
        let id = VarInt::read_from(&mut buf)?.0;

        let dir = match self.direction {
            Direction::Clientbound => Direction::Serverbound,
            Direction::Serverbound => Direction::Clientbound,
        };

        let packet = packet::packet_by_id(self.state, dir, id, &mut buf)?;

        match packet {
            Some(val) => {
                let pos = buf.position() as usize;
                let ibuf = buf.into_inner();
                if ibuf.len() != pos {
                    return Result::Err(Error::Err(format!("Failed to read all of packet 0x{:X}, \
                                                           had {} bytes left",
                                                          id,
                                                          ibuf.len() - pos)))
                }
                Result::Ok(val)
            }
            None => Result::Err(Error::Err("missing packet".to_owned())),
        }
    }

    pub fn enable_encyption(&mut self, key: &[u8], _decrypt: bool) {
        let cipher = Aes128Cfb::new_var(key, key).unwrap();
        self.cipher = Option::Some(cipher);
    }

    pub fn set_compresssion(&mut self, threshold: i32) {
        self.compression_threshold = threshold;
    }

    pub fn do_status(mut self) -> Result<(Status, Duration), Error> {
        use serde_json::Value;
        use self::packet::status::serverbound::*;
        use self::packet::handshake::serverbound::Handshake;
        use self::packet::Packet;
        let host = self.host.clone();
        let port = self.port;
        self.write_packet(Handshake {
            protocol_version: VarInt(SUPPORTED_PROTOCOL),
            host,
            port,
            next: VarInt(1),
        })?;
        self.state = State::Status;

        self.write_packet(StatusRequest { empty: () })?;

        let status = if let Packet::StatusResponse(res) = self.read_packet()? {
            res.status
        } else {
            return Err(Error::Err("Wrong packet".to_owned()));
        };

        let start = Instant::now();
        self.write_packet(StatusPing { ping: 42 })?;

        if let Packet::StatusPong(_) = self.read_packet()? {
        } else {
            return Err(Error::Err("Wrong packet".to_owned()));
        };

        let ping = start.elapsed();

        let val: Value = match serde_json::from_str(&status) {
            Ok(val) => val,
            Err(_) => return Err(Error::Err("Json parse error".to_owned())),
        };

        let invalid_status = || Error::Err("Invalid status".to_owned());

        let version = val.get("version").ok_or(invalid_status())?;
        let players = val.get("players").ok_or(invalid_status())?;

        Ok((Status {
            version: StatusVersion {
                name: version.get("name").and_then(Value::as_str).ok_or(invalid_status())?
                          .to_owned(),
                protocol: version.get("protocol")
                                      .and_then(Value::as_i64)
                                      .ok_or(invalid_status())? as i32,
            },
            players: StatusPlayers {
                max: players.get("max")
                                 .and_then(Value::as_i64)
                                 .ok_or(invalid_status())? as i32,
                online: players.get("online")
                                    .and_then(Value::as_i64)
                                    .ok_or(invalid_status())? as i32,
                sample: Vec::new(), /* TODO */
            },
            description: format::Component::from_value(val.get("description")
                                                               .ok_or(invalid_status())?),
            favicon: val.get("favicon").and_then(Value::as_str).map(|v| v.to_owned()),
        },
            ping))
    }
}

#[derive(Debug)]
pub struct Status {
    pub version: StatusVersion,
    pub players: StatusPlayers,
    pub description: format::Component,
    pub favicon: Option<String>,
}

#[derive(Debug)]
pub struct StatusVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug)]
pub struct StatusPlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<StatusPlayer>,
}

#[derive(Debug)]
pub struct StatusPlayer {
    name: String,
    id: String,
}

impl Read for Conn {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.cipher.as_mut() {
            Option::None => self.stream.read(buf),
            Option::Some(cipher) => {
                let ret = self.stream.read(buf)?;
                cipher.decrypt(&mut buf[..ret]);

                Ok(ret)
            }
        }
    }
}

impl Write for Conn {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.cipher.as_mut() {
            Option::None => self.stream.write(buf),
            Option::Some(cipher) => {
                // TODO: avoid copying, but trait requires non-mutable buf
                let mut data = vec![0; buf.len()];
                for i in 0..buf.len() {
                    data[i] = buf[i];
                }

                cipher.encrypt(&mut data);

                self.stream.write_all(&data)?;
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl Clone for Conn {
    fn clone(&self) -> Self {
        Conn {
            stream: self.stream.try_clone().unwrap(),
            host: self.host.clone(),
            port: self.port,
            direction: self.direction,
            state: self.state,
            cipher: Option::None,
            compression_threshold: self.compression_threshold,
            compression_read: Option::None,
            compression_write: Option::None,
        }
    }
}

pub trait PacketType {
    fn packet_id(&self) -> i32;

    fn write<W: io::Write>(self, buf: &mut W) -> Result<(), Error>;
}
