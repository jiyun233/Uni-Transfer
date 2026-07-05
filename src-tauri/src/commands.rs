use local_ip_address::list_afinet_netifas;
use std::collections::HashMap;
use tauri::command;

#[command]
pub fn get_all_ipv4() -> Result<HashMap<String, String>, String> {
    let mut result = HashMap::new();

    let exclude_keywords = vec![
        "127.",
        "169.254.",
        "bluetooth",
        "蓝牙",
        "virtual",
        "本地连接*",
        "local connection*",
    ];

    let preferred_keywords = vec!["wlan", "无线", "wifi", "以太网", "ethernet"];

    match list_afinet_netifas() {
        Ok(interfaces) => {
            for (name, ip) in interfaces {
                if !ip.is_ipv4() {
                    continue;
                }

                let ip_str = ip.to_string();
                let name_lower = name.to_lowercase();

                if exclude_keywords.iter().any(|&k| {
                    if k.ends_with('.') {
                        ip_str.starts_with(k)
                    } else {
                        name_lower.contains(k)
                    }
                }) {
                    continue;
                }

                let is_preferred = preferred_keywords.iter().any(|&k| name_lower.contains(k));

                if is_preferred {
                    result.insert(name, ip_str);
                }
            }

            if result.is_empty() {
                if let Ok(all) = list_afinet_netifas() {
                    for (name, ip) in all {
                        let ip_str = ip.to_string();
                        if ip.is_ipv4() && !ip_str.starts_with("127.") {
                            result.insert(name, ip_str);
                            break;
                        }
                    }
                }
            }

            Ok(result)
        }
        Err(e) => Err(format!("获取网卡信息失败: {}", e)),
    }
}
