/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implementation of the Philips Hue API
//!
//! This module is used in various places, for example in the Hub
//! objects and in the Light objects.

use serde_json;
use std;
use std::collections::BTreeMap;
use std::error::Error;
use super::http;
use super::structs;

#[derive(Debug, Clone)]
pub struct HubApi {
    pub id: String,
    pub ip: String,
    pub token: String,
}

impl std::fmt::Display for HubApi {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Hue Bridge id:{} at {:?}", self.id, self.ip)
    }
}

impl HubApi {
    pub fn new(id: &str, ip: &str, token: &str) -> HubApi {
        HubApi { id: id.to_owned(), ip: ip.to_owned(), token: token.to_owned() }
    }

    pub fn get(&self, cmd: &str) -> Result<String, Box<Error>> {
        let url = format!("http://{}/api/{}/{}", self.ip, self.token, cmd);
        debug!("GET request to Philips Hue bridge {}: {}", self.id, url);
        let content = http::get(&url);
        trace!("Philips Hue API response: {:?}", content);
        content
    }

    #[allow(dead_code)]
    pub fn post(&self, cmd: &str, data: &str) -> Result<String, Box<Error>> {
        let url = format!("http://{}/api/{}/{}", self.ip, self.token, cmd);
        debug!("POST request to Philips Hue bridge {}: {} data: {}", self.id, url, data);
        let content = http::post(&url, data);
        trace!("Philips Hue API response: {:?}", content);
        content
    }

    pub fn post_unauth(&self, cmd: &str, data: &str) -> Result<String, Box<Error>> {
        let url = format!("http://{}/{}", self.ip, cmd);
        debug!("POST request to Philips Hue bridge {}: {} data: {}", self.id, url, data);
        let content = http::post(&url, data);
        trace!("Philips Hue API response: {:?}", content);
        content
    }

    pub fn put(&self, cmd: &str, data: &str) -> Result<String, Box<Error>> {
        let url = format!("http://{}/api/{}/{}", self.ip, self.token, cmd);
        debug!("PUT request to Philips Hue bridge {}: {} data: {}", self.id, url, data);
        let content = http::put(&url, data);
        trace!("Philips Hue API response: {:?}", content);
        content
    }

    pub fn is_available(&self) -> bool {
        let url = format!("http://{}/", self.ip);
        let content = http::get(&url);
        match content {
            Ok(value) => {
                value.contains("hue personal wireless lighting")
            },
            Err(_) => {
                false
            }
        }
    }

    pub fn get_settings(&self) -> String {
        // [{"error":{"type":1,"address":"/","description":"unauthorized user"}}]
        self.get("").unwrap_or("".to_owned()) // TODO no unwrap
    }

    pub fn is_paired(&self) -> bool {
        let settings = self.get_settings();
        !settings.contains("unauthorized user")
    }

    pub fn try_pairing(&self) -> bool {
        // [{"success":{"username":"foxboxb-001788fffe25681a"}}]
        // [{"error":{"type":101,"address":"/","description":"link button not pressed"}}]
        let url = "api";
        let req = json!({ username: self.token, devicetype: "foxbox_hub"});
        let response = self.post_unauth(&url, &req).unwrap_or("".to_owned()); // TODO: no unwrap
        response.contains("success")
    }

    pub fn get_lights(&self) -> Vec<String> {
        let mut lights: Vec<String> = Vec::new();
        let url = "lights";
        let res = self.get(url).unwrap(); // TODO: remove unwrap
        let json: BTreeMap<String, structs::SettingsLightEntry> =
            structs::parse_json(&res).unwrap(); // TODO: no unwrap

        for key in json.keys() {
            lights.push(key.to_owned());
        }

        lights
    }

    pub fn get_light_status(&self, id: &str) -> structs::SettingsLightEntry {
        let url = format!("lights/{}", id);
        let res = self.get(&url).unwrap(); // TODO: remove unwrap
        structs::parse_json(&res).unwrap() // TODO no unwrap
    }

    #[allow(dead_code)]
    pub fn set_light_color(&self, light_id: &str, hue: u32, sat: u32, val: u32, on: bool) {
        let url = format!("lights/{}/state", light_id);
        let cmd = json!({ hue: hue, sat: sat, bri: val, on: on });
        let _ = self.put(&url, &cmd);
    }

    pub fn set_light_power(&self, light_id: &str, on: bool) {
        let url = format!("lights/{}/state", light_id);
        let cmd = json!({ on: on });
        let _ = self.put(&url, &cmd);
    }

}
