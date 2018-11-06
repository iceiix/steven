
#[allow(non_upper_case_globals)]
pub mod internal_ids {
    pub const Handshake: i32 = 0x00;
    pub const TeleportConfirm: i32 = 0x00;
    pub const TabComplete: i32 = 0x01;
    pub const ChatMessage: i32 = 0x02;
    pub const ClientStatus: i32 = 0x03;
    pub const ClientSettings: i32 = 0x04;
    pub const ConfirmTransactionServerbound: i32 = 0x05;
    pub const EnchantItem: i32 = 0x06;
    pub const ClickWindow: i32 = 0x07;
    pub const CloseWindow: i32 = 0x08;
    pub const PluginMessageServerbound: i32 = 0x09;
    pub const UseEntity: i32 = 0x0a;
    pub const KeepAliveServerbound: i32 = 0x0b;
    pub const PlayerPosition: i32 = 0x0c;
    pub const PlayerPositionLook: i32 = 0x0d;
    pub const PlayerLook: i32 = 0x0e;
    pub const Player: i32 = 0x0f;
    pub const VehicleMove: i32 = 0x10;
    pub const SteerBoat: i32 = 0x11;
    pub const ClientAbilities: i32 = 0x12;
    pub const PlayerDigging: i32 = 0x13;
    pub const PlayerAction: i32 = 0x14;
    pub const SteerVehicle: i32 = 0x15;
    pub const ResourcePackStatus: i32 = 0x16;
    pub const HeldItemChange: i32 = 0x17;
    pub const CreativeInventoryAction: i32 = 0x18;
    pub const SetSign: i32 = 0x19;
    pub const ArmSwing: i32 = 0x1a;
    pub const SpectateTeleport: i32 = 0x1b;
    pub const PlayerBlockPlacement: i32 = 0x1c;
    pub const PlayerBlockPlacementInt: i32 = -1;
    pub const UseItem: i32 = 0x1d;
    pub const SpawnObject: i32 = 0x00;
    pub const SpawnExperienceOrb: i32 = 0x01;
    pub const SpawnGlobalEntity: i32 = 0x02;
    pub const SpawnMob: i32 = 0x03;
    pub const SpawnMobInt: i32 = -1;
    pub const SpawnPainting: i32 = 0x04;
    pub const SpawnPlayer: i32 = 0x05;
    pub const Animation: i32 = 0x06;
    pub const Statistics: i32 = 0x07;
    pub const BlockBreakAnimation: i32 = 0x08;
    pub const UpdateBlockEntity: i32 = 0x09;
    pub const BlockAction: i32 = 0x0a;
    pub const BlockChange: i32 = 0x0b;
    pub const BossBar: i32 = 0x0c;
    pub const ServerDifficulty: i32 = 0x0d;
    pub const TabCompleteReply: i32 = 0x0e;
    pub const ServerMessage: i32 = 0x0f;
    pub const MultiBlockChange: i32 = 0x10;
    pub const ConfirmTransaction: i32 = 0x11;
    pub const WindowClose: i32 = 0x12;
    pub const WindowOpen: i32 = 0x13;
    pub const WindowItems: i32 = 0x14;
    pub const WindowProperty: i32 = 0x15;
    pub const WindowSetSlot: i32 = 0x16;
    pub const SetCooldown: i32 = 0x17;
    pub const PluginMessageClientbound: i32 = 0x18;
    pub const NamedSoundEffect: i32 = 0x19;
    pub const Disconnect: i32 = 0x1a;
    pub const EntityAction: i32 = 0x1b;
    pub const Explosion: i32 = 0x1c;
    pub const ChunkUnload: i32 = 0x1d;
    pub const ChangeGameState: i32 = 0x1e;
    pub const KeepAliveClientbound: i32 = 0x1f;
    pub const ChunkData: i32 = 0x20;
    pub const Effect: i32 = 0x21;
    pub const Particle: i32 = 0x22;
    pub const JoinGame: i32 = 0x23;
    pub const Maps: i32 = 0x24;
    pub const EntityMove: i32 = 0x25;
    pub const EntityLookAndMove: i32 = 0x26;
    pub const EntityLook: i32 = 0x27;
    pub const Entity: i32 = 0x28;
    pub const VehicleTeleport: i32 = 0x29;
    pub const SignEditorOpen: i32 = 0x2a;
    pub const PlayerAbilities: i32 = 0x2b;
    pub const CombatEvent: i32 = 0x2c;
    pub const PlayerInfo: i32 = 0x2d;
    pub const TeleportPlayer: i32 = 0x2e;
    pub const EntityUsedBed: i32 = 0x2f;
    pub const EntityDestroy: i32 = 0x30;
    pub const EntityRemoveEffect: i32 = 0x31;
    pub const ResourcePackSend: i32 = 0x32;
    pub const Respawn: i32 = 0x33;
    pub const EntityHeadLook: i32 = 0x34;
    pub const WorldBorder: i32 = 0x35;
    pub const Camera: i32 = 0x36;
    pub const SetCurrentHotbarSlot: i32 = 0x37;
    pub const ScoreboardDisplay: i32 = 0x38;
    pub const EntityMetadata: i32 = 0x39;
    pub const EntityAttach: i32 = 0x3a;
    pub const EntityVelocity: i32 = 0x3b;
    pub const EntityEquipment: i32 = 0x3c;
    pub const SetExperience: i32 = 0x3d;
    pub const UpdateHealth: i32 = 0x3e;
    pub const ScoreboardObjective: i32 = 0x3f;
    pub const SetPassengers: i32 = 0x40;
    pub const Teams: i32 = 0x41;
    pub const UpdateScore: i32 = 0x42;
    pub const SpawnPosition: i32 = 0x43;
    pub const TimeUpdate: i32 = 0x44;
    pub const Title: i32 = 0x45;
    pub const TitleShifted: i32 = -1;
    pub const SoundEffect: i32 = 0x46;
    pub const PlayerListHeaderFooter: i32 = 0x47;
    pub const CollectItem: i32 = 0x48;
    pub const CollectItemUncounted: i32 = -1;
    pub const EntityTeleport: i32 = 0x49;
    pub const EntityProperties: i32 = 0x4a;
    pub const EntityEffect: i32 = 0x4b;
    pub const LoginStart: i32 = 0x00;
    pub const EncryptionResponse: i32 = 0x01;
    pub const LoginDisconnect: i32 = 0x00;
    pub const EncryptionRequest: i32 = 0x01;
    pub const LoginSuccess: i32 = 0x02;
    pub const SetInitialCompression: i32 = 0x03;
    pub const StatusRequest: i32 = 0x00;
    pub const StatusPing: i32 = 0x01;
    pub const StatusResponse: i32 = 0x00;
    pub const StatusPong: i32 = 0x01;
}
