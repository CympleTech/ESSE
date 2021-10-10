use tdn::types::{
    primitive::{PeerAddr, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use group_chat_types::GroupType;

/// Group Chat Provider Model.
pub(crate) struct Provider {
    pub id: i64,
    name: String,
    addr: PeerAddr,
    kinds: Vec<GroupType>,
    remain: i64,
    is_ok: bool,
}

fn parse_kinds(kinds: i64) -> Vec<GroupType> {
    let s = kinds.to_string();
    s.chars()
        .filter_map(|c| match c {
            '0' => Some(GroupType::Encrypted),
            '1' => Some(GroupType::Private),
            '2' => Some(GroupType::Open),
            _ => None,
        })
        .collect()
}

fn kinds_print(kinds: &Vec<GroupType>) -> i64 {
    let mut v: Vec<u32> = kinds.iter().map(|k| k.to_u32()).collect();
    v.sort_by(|a, b| b.cmp(a));
    let s = v
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join("");
    s.parse().unwrap_or(0)
}

impl Provider {
    pub fn new(addr: PeerAddr) -> Self {
        Self {
            addr,
            name: String::new(),
            kinds: vec![],
            is_ok: false,
            remain: 0,
            id: 0,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        let kinds: Vec<u32> = self.kinds.iter().map(|v| v.to_u32()).collect();
        json!([
            self.id,
            self.name,
            self.addr.to_hex(),
            kinds,
            self.remain,
            self.is_ok
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            is_ok: v.pop().unwrap().as_bool(),
            remain: v.pop().unwrap().as_i64(),
            kinds: parse_kinds(v.pop().unwrap().as_i64()),
            addr: PeerAddr::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            name: v.pop().unwrap().as_string(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn list(db: &DStorage) -> Result<Vec<Self>> {
        let matrix = db.query("SELECT id, name, addr, kinds, remain, is_ok FROM providers")?;
        let mut providers = vec![];
        for values in matrix {
            providers.push(Self::from_values(values));
        }
        Ok(providers)
    }

    pub fn get_by_addr(db: &DStorage, addr: &PeerAddr) -> Result<Self> {
        let sql = format!(
            "SELECT id, name, addr, kinds, remain, is_ok FROM providers WHERE addr = '{}'",
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
        let sql = format!(
            "INSERT INTO providers (name,addr,kinds,remain,is_ok) VALUES ('{}','{}',{},{},{})",
            self.name,
            self.addr.to_hex(),
            kinds_print(&self.kinds),
            self.remain,
            self.is_ok,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn update(
        &mut self,
        db: &DStorage,
        name: String,
        kinds: Vec<GroupType>,
        remain: i64,
    ) -> Result<()> {
        self.name = name;
        self.kinds = kinds;
        self.remain = remain;

        let sql = format!(
            "UPDATE providers SET is_ok=true, name='{}', kinds={}, remain={} WHERE id = {}",
            self.name,
            kinds_print(&self.kinds),
            self.remain,
            self.id
        );
        db.update(&sql)?;
        Ok(())
    }

    pub fn suspend(&mut self, db: &DStorage) -> Result<()> {
        self.is_ok = false;
        let sql = format!("UPDATE providers SET is_ok=false WHERE id = {}", self.id);
        db.update(&sql)?;
        Ok(())
    }

    pub fn delete(db: &DStorage, id: &i64) -> Result<()> {
        let sql = format!("DELETE FROM providers WHERE id = {}", id);
        db.update(&sql)?;
        Ok(())
    }
}
