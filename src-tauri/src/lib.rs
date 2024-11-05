// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// use std::net::SocketAddr;
use std::env;
use std::net::SocketAddr;
use std::str;

#[macro_use]
extern crate lazy_static;
use lazy_static::lazy_static;

use pnet_datalink::interfaces;
use serde::de;
// use tauri::{command, Builder, State};
use std::net::Ipv4Addr;
use std::net::IpAddr;
use tokio::net::UdpSocket;
use tokio::task;
use tokio::time::{self, Duration};
use tauri::{AppHandle, Emitter};
use tauri::Manager;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};


#[derive(Deserialize, Serialize,Debug)]
struct Device {
    device_name: String,
    ip_addresses: HashMap<String, String>,
    mac: String,
    last_updated: i64, // 增加时间戳字段
}


#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
const LOCAL_ADDR: &str = "0.0.0.0:21246";
// const LOCAL_ADDR: &str = "192.168.108.2:21246";

fn find_broadcast_ips() -> Vec<IpAddr> {
    let all_interfaces = interfaces();
    let default_interface = all_interfaces.iter().filter(|e| true);
    let mut valid_broadcasts = Vec::new();
    for ip in default_interface {
        if(ip.ips.len() == 0) {
            continue;
        }
        let broadcast = ip.ips[0].broadcast();
        if broadcast != Ipv4Addr::new(255, 255, 255, 255) {
            valid_broadcasts.push(broadcast);
        }
    }
    valid_broadcasts 
}
async fn send_discov_req() {
    // Get a vector with all network interfaces found
    let interval = Duration::from_secs(5); // 每5秒发送一次
    loop {
        println!("Sent: {}", "EMNAVI_DEV_DISCOV_REQ");
        let valid_broadcasts = find_broadcast_ips();
        for broadcast_ip in valid_broadcasts {
            let broadcast_addr: SocketAddr = format!("{}:21245", broadcast_ip)
                .parse()
                .expect("Invalid address");
            let socket = UdpSocket::bind("0.0.0.0:0")
                .await
                .expect("Failed to bind socket");
            let message = "EMNAVI_DEV_DISCOV_REQ";

            // 发送 UDP 广播
            if let Err(e) = socket.send_to(message.as_bytes(), &broadcast_addr).await {
                eprintln!("Failed to send UDP message: {:?}", e);
            } else {
                // println!("Sent: {}", message);
            }
        }
        time::sleep(interval).await;
    }
}

lazy_static! {
    static ref DEVICES: Arc<Mutex<Vec<Device>>> = Arc::new(Mutex::new(Vec::new()));
}

#[tauri::command]
fn get_devices (app_handle: tauri::AppHandle) -> String {
    let devices_lock = DEVICES.lock().unwrap(); // 锁定以读取
    for device in devices_lock.iter() {
        println!("{:?}", device); // 打印每个设备的信息
    }
    let devices_json: String = serde_json::to_string(&*devices_lock).unwrap();
    devices_json
}

async fn receive_multicast() {
    let socket = UdpSocket::bind(LOCAL_ADDR)
        .await
        .expect("Failed to bind socket");
    // socket.join_multicast_v4("239.255.0.1".parse().unwrap(), "0.0.0.0".parse().unwrap())
    //     .expect("Failed to join multicast group");

    let mut buf = [0; 1024];

    loop {
        let (len, _addr) = socket
            .recv_from(&mut buf)
            .await
            .expect("Failed to receive data");
        // let message = str::from_utf8(&buf[..len]).unwrap();
        let json_str = String::from_utf8_lossy(&buf[..len]);
        println!("Received: {}", json_str);
        match serde_json::from_str::<Device>(&json_str) {
            Ok(mut device) => {
                // 判断mac地址是否已经存在
                let mut is_exist = false;
                let mut devices_lock: std::sync::MutexGuard<'_, Vec<Device>> = DEVICES.lock().unwrap(); // 锁定
                for d in devices_lock.iter_mut() {
                    d.last_updated = Utc::now().timestamp(); // 更新时间戳
                    if d.mac == device.mac {
                        is_exist = true;
                        println!("Device already exists: {:?}", device);
                        // break;
                    }
                }
                if is_exist == false {
                    device.last_updated = Utc::now().timestamp(); // 更新时间戳
                    println!("New device found: {:?}", device);
                    devices_lock.push(device); // 存储设备信息
                }
                // let devices_json: String = serde_json::to_string(&*devices_lock).unwrap();
            }
            Err(e) => eprintln!("Failed to parse JSON: {}", e),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {

    // 启动接收多播的任务
    // let state_clone = state.clone();
    task::spawn(send_discov_req());
    task::spawn(receive_multicast());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet,get_devices])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
