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

    pub fn user_exists(&self, username: &str) -> crate::Result<bool> {
        self.0
            .query_one(
                "SELECT COUNT(*) FROM souls WHERE name = ?1",
                [username],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|e| e.into())
    }

    pub fn insert_user(&self, username: &str, password: &str) -> crate::Result<usize> {
        self.0
            .execute(
                "INSERT INTO souls (name, password, privileged) VALUES (?1, ?2, ?3)",
                (&username, &password, true),
            )
            .map_err(|e| e.into())
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
}
