use tdn::types::{
    primitive::{PeerAddr, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

/// Provider Model.
pub(crate) struct Provider {
    /// db auto-increment id.
    pub id: i64,
    /// name.
    name: String,
    /// address.
    addr: PeerAddr,
    /// is add ok.
    is_ok: bool,
    /// is default.
    is_default: bool,
    /// support request proxy.
    is_proxy: bool,
    /// is actived.
    is_actived: bool,
}

impl Provider {
    pub fn prepare(addr: PeerAddr) -> Self {
        Self {
            id: 0,
            name: addr.to_hex(),
            addr: addr,
            is_ok: false,
            is_default: false,
            is_proxy: false,
            is_actived: false,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.name,
            self.addr.to_hex(),
            self.is_ok,
            self.is_default,
            self.is_proxy,
            self.is_actived,
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            is_actived: v.pop().unwrap().as_bool(),
            is_proxy: v.pop().unwrap().as_bool(),
            is_default: v.pop().unwrap().as_bool(),
            is_ok: v.pop().unwrap().as_bool(),
            addr: PeerAddr::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            name: v.pop().unwrap().as_string(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    /// use in rpc when load providers.
    pub fn list(db: &DStorage) -> Result<Vec<Self>> {
        let matrix = db.query(
            "SELECT id, name, addr, is_ok, is_default, is_proxy, is_actived FROM providers",
        )?;
        let mut providers = vec![];
        for values in matrix {
            providers.push(Self::from_values(values));
        }
        Ok(providers)
    }

    /// use in rpc when load provider by id.
    pub fn get(db: &DStorage, id: &i64) -> Result<Self> {
        let sql = format!(
            "SELECT id, name, addr, is_ok, is_default, is_proxy, is_actived FROM providers WHERE id = {}",
            id
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("provider is missing"))
    }

    /// insert a new provider.
    pub fn get_by_addr(db: &DStorage, addr: &PeerAddr) -> Result<Self> {
        let sql = format!(
            "SELECT id, name, addr, is_ok, is_default, is_proxy, is_actived FROM providers WHERE addr = '{}'",
            addr.to_hex()
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("provider is missing"))
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from providers WHERE addr = '{}'",
            self.addr.to_hex()
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;
            let sql = format!("UPDATE providers SET name = '{}', addr = '{}', is_ok = {}, is_default = {}, is_proxy = {}, is_actived = {} WHERE id = {}",
                self.name,
                self.addr.to_hex(),
                self.is_ok,
                self.is_default,
                self.is_proxy,
                self.is_actived,
                self.id
            );
            db.update(&sql)?;
        } else {
            let sql = format!(
                "INSERT INTO providers (name, addr, is_ok, is_default, is_proxy, is_actived) VALUES ('{}', '{}', {}, {}, {}, {})",
                self.name,
                self.addr.to_hex(),
                self.is_ok,
                self.is_default,
                self.is_proxy,
                self.is_actived,
            );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    pub fn ok(&mut self, db: &DStorage, name: String, is_proxy: bool) -> Result<()> {
        self.name = name;
        self.is_proxy = is_proxy;
        self.is_actived = true;
        self.is_ok = true;

        let sql = format!("UPDATE providers SET name = '{}', is_ok = true, is_proxy = {}, is_actived = true WHERE id = {}",
                self.name,
                self.is_proxy,
                self.id
            );
        db.update(&sql)?;
        Ok(())
    }

    /// return if is closed
    pub fn delete(db: &DStorage, id: &i64) -> Result<()> {
        let sql = format!("UPDATE providers SET is_actived = false WHERE id = {}", id);
        db.update(&sql)?;
        Ok(())
    }
}

/// Name Model.
pub(crate) struct Name {
    /// db auto-increment id.
    id: i64,
    /// provider database id.
    provider: i64,
    /// name.
    name: String,
    /// bio.
    bio: String,
    /// is add ok.
    is_ok: bool,
    /// is actived.
    is_actived: bool,
}

impl Name {
    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.provider,
            self.name,
            self.bio,
            self.is_ok,
            self.is_actived,
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            is_actived: v.pop().unwrap().as_bool(),
            is_ok: v.pop().unwrap().as_bool(),
            bio: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            provider: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    /// use in rpc when load providers.
    pub fn list(db: &DStorage) -> Result<Vec<Self>> {
        let matrix = db.query("SELECT id, provider, name, bio, is_ok, is_actived FROM names")?;
        let mut names = vec![];
        for values in matrix {
            names.push(Self::from_values(values));
        }
        Ok(names)
    }

    /// use in rpc when load provider by id.
    pub fn get(db: &DStorage, id: &i64) -> Result<Self> {
        let sql = format!(
            "SELECT id, provider, name, bio, is_ok, is_actived FROM names WHERE id = {}",
            id
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("name is missing"))
    }

    /// get name register.
    pub fn get_by_name_provider(db: &DStorage, name: &str, provider: &i64) -> Result<Self> {
        let sql = format!(
            "SELECT id, provider, name, bio, is_ok, is_actived FROM names WHERE name = '{}' AND provider = {}",
            name, provider
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("name is missing"))
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from names WHERE provider = {} AND name = '{}'",
            self.provider, self.name
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;
            let sql = format!(
                "UPDATE names SET bio = '{}', is_ok = {}, is_actived = {} WHERE id = {}",
                self.bio, self.is_ok, self.is_actived, self.id
            );
            db.update(&sql)?;
        } else {
            let sql = format!(
                "INSERT INTO names (provider, name, bio, is_ok, is_actived) VALUES ({}, '{}', '{}', {}, {})",
                self.provider,
                self.name,
                self.bio,
                self.is_ok,
                self.is_actived,
            );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    /// delete the name.
    pub fn delete(db: &DStorage, id: &i64) -> Result<()> {
        let sql = format!("DELETE names WHERE id = {}", id);
        db.delete(&sql)?;
        Ok(())
    }

    /// active/suspend the name.
    pub fn active(db: &DStorage, id: &i64, active: bool) -> Result<()> {
        let sql = format!("UPDATE names SET is_actived = {} WHERE id = {}", active, id);
        db.update(&sql)?;
        Ok(())
    }
}
