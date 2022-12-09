use tdn::types::{
    primitives::{PeerId, Result},
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
    pub addr: PeerId,
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
    pub fn prepare(addr: PeerId) -> Self {
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
            addr: PeerId::from_hex(v.pop().unwrap().as_str()).unwrap_or(Default::default()),
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

    /// use in rpc when load provider by id.
    pub fn get_default(db: &DStorage) -> Result<Self> {
        let mut matrix = db.query("SELECT id, name, addr, is_ok, is_default, is_proxy, is_actived FROM providers WHERE is_default = true")?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("provider is missing"))
    }

    /// insert a new provider.
    pub fn get_by_addr(db: &DStorage, addr: &PeerId) -> Result<Self> {
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

    /// set default provider.
    pub fn default(&self, db: &DStorage, default: bool) -> Result<()> {
        let sql = format!(
            "UPDATE providers SET is_default = {} WHERE id = {}",
            default, self.id
        );
        db.update(&sql)?;
        Ok(())
    }

    /// delete provider.
    pub fn delete(db: &DStorage, id: &i64) -> Result<()> {
        let sql = format!("DELETE FROM providers WHERE id = {}", id);
        db.update(&sql)?;
        Ok(())
    }
}

/// Name Model.
pub(crate) struct Name {
    /// db auto-increment id.
    pub id: i64,
    /// provider database id.
    pub provider: i64,
    /// name.
    pub name: String,
    /// bio.
    pub bio: String,
    /// is add ok.
    pub is_ok: bool,
    /// is actived.
    pub is_actived: bool,
}

impl Name {
    pub fn prepare(name: String, bio: String, provider: i64) -> Self {
        Self {
            name,
            bio,
            provider,
            is_ok: false,
            is_actived: false,
            id: 0,
        }
    }

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
        let matrix = db.query(
            "SELECT id, provider, name, bio, is_ok, is_actived FROM names WHERE is_ok = true",
        )?;
        let mut names = vec![];
        for values in matrix {
            names.push(Self::from_values(values));
        }
        Ok(names)
    }

    /// get name register.
    pub fn get_by_provider(db: &DStorage, provider: &i64) -> Result<Vec<Self>> {
        let sql = format!(
            "SELECT id, provider, name, bio, is_ok, is_actived FROM names WHERE provider = {}",
            provider
        );
        let matrix = db.query(&sql)?;
        let mut names = vec![];
        for values in matrix {
            names.push(Self::from_values(values));
        }
        Ok(names)
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
    pub fn delete(&self, db: &DStorage) -> Result<()> {
        let sql = format!("DELETE FROM names WHERE id = {}", self.id);
        db.delete(&sql)?;
        Ok(())
    }

    /// active/suspend the name.
    pub fn active(db: &DStorage, id: &i64, active: bool) -> Result<()> {
        let sql = format!(
            "UPDATE names SET is_ok = true, is_actived = {} WHERE id = {}",
            active, id
        );
        db.update(&sql)?;
        Ok(())
    }
}
