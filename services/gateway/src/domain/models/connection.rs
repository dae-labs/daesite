use super::id::ID;
use std::net::IpAddr;

pub struct Connection {
    pub id: ID,
    pub ip: IpAddr,
    pub user_id: Option<ID>,
    pub missed_heartbeats: u32,
}
