use crate::{Context, User, buffer::BufferWriter};

pub enum UserStatus {
    Offline = 0,
    Away = 1,
    Online = 2,
}

impl UserStatus {
    fn try_from_i32(value: i32) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Offline),
            1 => Ok(Self::Away),
            2 => Ok(Self::Online),
            _ => Err(format!("{value} is not a valid user status")),
        }
    }
}

pub enum ServerMessage {
    Login = 1,
    SetWaitPort = 2,
    GetUserStatus = 7,
    SetStatus = 28,
    SharedFoldersFiles = 35,
    RoomList = 64,
    HaveNoParent = 71,
    CheckPrivileges = 92,
    AcceptChildren = 100,
    BranchLevel = 126,
    BranchRoot = 127,
    PrivateRoomToggle = 141,
}

impl ServerMessage {
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            1 => Some(Self::Login),
            2 => Some(Self::SetWaitPort),
            7 => Some(Self::GetUserStatus),
            28 => Some(Self::SetStatus),
            35 => Some(Self::SharedFoldersFiles),
            64 => Some(Self::RoomList),
            71 => Some(Self::HaveNoParent),
            92 => Some(Self::CheckPrivileges),
            100 => Some(Self::AcceptChildren),
            126 => Some(Self::BranchLevel),
            127 => Some(Self::BranchRoot),
            141 => Some(Self::PrivateRoomToggle),
            _ => None,
        }
    }

    pub fn process(&self, ctx: &mut Context) -> crate::Result<Option<Vec<u8>>> {
        let response = match self {
            Self::Login => login(ctx),
            Self::SetWaitPort => set_wait_port(ctx),
            Self::GetUserStatus => get_user_status(ctx),
            Self::SetStatus => set_status(ctx),
            Self::SharedFoldersFiles => shared_folders_files(ctx),
            Self::RoomList => room_list(ctx),
            Self::HaveNoParent => have_no_parent(ctx),
            Self::CheckPrivileges => check_privileges(ctx),
            Self::AcceptChildren => accept_children(ctx),
            Self::BranchLevel => branch_level(ctx),
            Self::BranchRoot => branch_root(ctx),
            Self::PrivateRoomToggle => private_room_toggle(ctx),
        }?;
        Ok(response.map(|r| r.to_vec()))
    }
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-1
fn login(ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let username = ctx.reader.read_string()?;
    let password = ctx.reader.read_string()?;
    let _version = ctx.reader.read_u32()?;
    let _hash = ctx.reader.read_string()?;
    let _minor_version = ctx.reader.read_u32()?;

    let mut writer = BufferWriter::new();
    writer.write_u32(ServerMessage::Login as u32);

    if !ctx.db.user_exists(&username)? {
        // writer.write_bool(false).write_string("INVALIDPASS");
        // return Ok(Some(writer));
        ctx.db.insert_user(&username, &password)?;
    }

    let hash: String = md5::compute(password.as_bytes())
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();

    writer
        .write_bool(true)
        .write_string("Hello")
        .write_u32(ctx.socket_addr.ip().to_bits())
        .write_string(&hash)
        .write_bool(true);

    ctx.users.write().unwrap().users.insert(
        ctx.socket_addr,
        User {
            name: username,
            addr: ctx.socket_addr,
            status: UserStatus::Online,
        },
    );

    Ok(Some(writer))
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-2
fn set_wait_port(ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let _port = ctx.reader.read_u32()?;

    if !ctx.reader.is_empty() {
        let _obfuscation_type = ctx.reader.read_u32()?;
        let _obfuscated_port = ctx.reader.read_u32()?;
    }

    Ok(None)
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-7
fn get_user_status(ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let username = ctx.reader.read_string()?;
    let mut writer = BufferWriter::new();
    writer
        .write_u32(ServerMessage::GetUserStatus as u32)
        .write_string(&username)
        .write_u32(UserStatus::Online as u32)
        .write_bool(ctx.db.is_user_privileged(&username)?);
    Ok(Some(writer))
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-28
fn set_status(ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let status = UserStatus::try_from_i32(ctx.reader.read_i32()?)?;

    ctx.users
        .write()
        .unwrap()
        .users
        .entry(ctx.socket_addr)
        .and_modify(|v| v.status = status);
    Ok(None)
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-35
fn shared_folders_files(ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let _dirs = ctx.reader.read_u32()?;
    let _files = ctx.reader.read_u32()?;
    Ok(None)
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-64
fn room_list(_ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let mut writer = BufferWriter::new();
    writer
        .write_u32(ServerMessage::RoomList as u32)
        .write_u32(0);
    Ok(Some(writer))
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-71
fn have_no_parent(ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let _no_parent = ctx.reader.read_bool()?;
    Ok(None)
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-92
fn check_privileges(_ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let mut writer = BufferWriter::new();
    writer
        .write_u32(ServerMessage::CheckPrivileges as u32)
        .write_u32(u32::MAX);
    Ok(Some(writer))
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-100
fn accept_children(ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let _accept = ctx.reader.read_bool()?;
    Ok(None)
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-126
fn branch_level(ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let _branch_level = ctx.reader.read_u32()?;
    Ok(None)
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-127
fn branch_root(ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let _branch_root = ctx.reader.read_string()?;
    Ok(None)
}

// https://github.com/nicotine-plus/nicotine-plus/blob/master/doc/SLSKPROTOCOL.md#server-code-141
fn private_room_toggle(ctx: &mut Context) -> crate::Result<Option<BufferWriter>> {
    let enable = ctx.reader.read_bool()?;
    let mut writer = BufferWriter::new();
    writer
        .write_u32(ServerMessage::PrivateRoomToggle as u32)
        .write_bool(enable);
    Ok(Some(writer))
}
