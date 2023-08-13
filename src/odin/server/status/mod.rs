mod bepinex_info;
mod jobs_info;

use crate::constants::{AUTO_BACKUP_JOB, AUTO_UPDATE_JOB};
use crate::utils::environment::fetch_var;
use a2s::info::Info;
use a2s::A2SClient;
use bepinex_info::BepInExInfo;
use jobs_info::JobInfo;
use log::error;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::net::SocketAddrV4;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerInfo {
  pub name: String,
  pub game_version: String,
  pub network_version: String,
  pub players: u8,
  pub max_players: u8,
  pub map: String,
  pub online: bool,
  pub bepinex: BepInExInfo,
  pub jobs: Vec<JobInfo>,
}

impl ServerInfo {
  pub fn new(address: SocketAddrV4) -> ServerInfo {
    let query_client = A2SClient::new().unwrap();
    match query_client.info(address) {
      Ok(a2s_info) => ServerInfo::from(a2s_info),
      Err(_err) => {
        error!("Failed to request server information!");
        ServerInfo::offline()
      }
    }
  }
  pub fn offline() -> ServerInfo {
    let unknown = String::from("Unknown");
    ServerInfo {
      name: fetch_var("NAME", &unknown),
      game_version: unknown.clone(),
      network_version: unknown.clone(),
      players: 0,
      max_players: 0,
      map: fetch_var("NAME", &unknown),
      online: false,
      bepinex: BepInExInfo::disabled(),
      jobs: vec![],
    }
  }
}

impl From<SocketAddrV4> for ServerInfo {
  fn from(address: SocketAddrV4) -> ServerInfo {
    ServerInfo::new(address)
  }
}

impl From<Info> for ServerInfo {
  fn from(info: Info) -> ServerInfo {
    let version = String::from(&info.clone().extended_server_info.keywords.unwrap());
    let unknown = String::from("Unknown");
    let game_version: Option<String> = version
      .split(',')
      .filter_map(|x| match x.contains("gameversion") {
        true => x
          .split('=')
          .last()
          .map(|y| String::from(y.trim_matches('"'))),
        false => None,
      })
      .next();
    let network_version: Option<String> = version
      .split(',')
      .filter_map(|x| match x.contains("networkversion") {
        true => x
          .split('=')
          .last()
          .map(|y| String::from(y.trim_matches('"'))),
        false => None,
      })
      .next();
    ServerInfo {
      name: info.name,
      game_version: game_version.unwrap_or(unknown.clone()),
      network_version: network_version.unwrap_or(unknown.clone()),
      players: info.players,
      max_players: info.max_players,
      map: info.map,
      online: true,
      bepinex: BepInExInfo::new(),
      jobs: vec![
        JobInfo::from_str(AUTO_UPDATE_JOB).unwrap(),
        JobInfo::from_str(AUTO_BACKUP_JOB).unwrap(),
      ],
    }
  }
}

impl Display for ServerInfo {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let bepinex = &self.bepinex;
    let mut server_info = vec![
      format!("Name: {}", &self.name),
      format!("Game Version: {}", &self.game_version),
      format!("Network Version: {}", &self.network_version),
      format!("Players: {}/{}", &self.players, &self.max_players),
      format!("Map: {}", &self.map),
      format!("BepInEx Enabled: {}", bepinex.enabled),
    ];
    if bepinex.enabled {
      let mods: Vec<String> = bepinex.mods.iter().map(|m| String::from(&m.name)).collect();
      server_info.push(format!("BepInEx Mods: {}", mods.join(", ")))
    }
    write!(f, "{}", server_info.join("\n"))
  }
}
