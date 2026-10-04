use rusqlite::Connection;

const DB_PATH: &str = "souls.db";

pub struct Db(Connection);

impl Db {
    pub fn open() -> crate::Result<Db> {
        Ok(Self(Connection::open(DB_PATH)?))
    }

    pub fn table_exists(&self) -> crate::Result<bool> {
        self.0.table_exists(None, "souls").map_err(|e| e.into())
    }

    pub fn create_table(&self) -> crate::Result<usize> {
        self.0
            .execute(
                "CREATE TABLE souls (name TEXT, password TEXT, privileged BOOLEAN)",
                [],
            )
            .map_err(|e| e.into())
    }

    fn user_exists(&self, username: &str) -> Result<bool, rusqlite::Error> {
        self.0.query_one(
            "SELECT COUNT(*) FROM souls WHERE name = ?1",
            [username],
            |row| row.get::<_, bool>(0),
        )
    }

    pub fn add_user(&self, username: &str, password: &str) -> Result<(), AddUserError> {
        if self.user_exists(username).map_err(AddUserError::Db)? {
            return Err(AddUserError::UserAlreadyExists);
        }

        if password.is_empty() {
            return Err(AddUserError::EmptyPassword);
        }

        let password_hash: String = md5::compute(password)
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        self.0
            .execute(
                "INSERT INTO souls (name, password, privileged) VALUES (?1, ?2, ?3)",
                (username, password_hash, true),
            )
            .map(|_| ())
            .map_err(AddUserError::Db)
    }

    pub fn is_user_privileged(&self, username: &str) -> crate::Result<bool> {
        self.0
            .query_one(
                "SELECT privileged FROM souls WHERE name = ?1",
                [username],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|e| e.into())
    }

    pub fn get_user_password(&self, username: &str) -> Option<String> {
        self.0
            .query_one(
                "SELECT password FROM souls WHERE name = ?1",
                [username],
                |row| row.get::<_, String>(0),
            )
            .ok()
    }
}

pub enum AddUserError {
    UserAlreadyExists,
    EmptyPassword,
    Db(rusqlite::Error),
}

impl std::fmt::Display for AddUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserAlreadyExists => f.write_str("user already exists"),
            Self::EmptyPassword => f.write_str("empty password"),
            Self::Db(e) => f.write_fmt(format_args!("db error: {e}")),
        }
    }
}
