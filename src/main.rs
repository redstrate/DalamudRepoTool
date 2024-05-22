use std::collections::HashMap;
use std::fs::{read, read_to_string, write};
use std::str::FromStr;
use chrono::format::Fixed;
use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use toml::value::{Datetime, Offset};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    repo_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PluginManifest {
    #[serde(rename = "Author")]
    author: String,

    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "InternalName")]
    internal_name: String,

    #[serde(rename = "AssemblyVersion")]
    assembly_version: String,

    #[serde(rename = "Description")]
    description: String,

    #[serde(rename = "ApplicableVersion")]
    applicable_version: String,

    #[serde(rename = "RepoUrl")]
    repo_url: String,

    #[serde(rename = "Tags")]
    tags: Vec<String>,

    #[serde(rename = "DalamudApiLevel")]
    dalamud_api_level: i32,

    #[serde(rename = "LoadRequiredState")]
    load_required_state: i32,

    #[serde(rename = "LoadSync")]
    load_sync: bool,

    #[serde(rename = "CanUnloadAsync")]
    can_unload_async: bool,

    #[serde(rename = "LoadPriority")]
    load_priority: i32,

    #[serde(rename = "Punchline")]
    punchline: String,

    #[serde(rename = "AcceptsFeedback")]
    accepts_feedback: bool,

    // internal and repo usage
    #[serde(rename = "_isDip17Plugin")]
    is_dip17_plugin: bool,

    #[serde(rename = "_Dip17Channel")]
    dip17_channel: String,

    #[serde(default)]
    changelog: String,

    #[serde(default)]
    category_tags: Option<Vec<String>>,

    #[serde(default)]
    is_hide: bool,

    #[serde(default)]
    testing_assembly_version: Option<String>,

    #[serde(default)]
    is_testing_exclusive: bool,

    #[serde(default)]
    download_count: i32,

    #[serde(default)]
    last_update: i64,

    #[serde(default)]
    download_link_install: String,

    #[serde(default)]
    download_link_update: String,

    #[serde(default)]
    download_link_testing: String,

    #[serde(default)]
    image_urls: Option<Vec<String>>,

    #[serde(default)]
    icon_url: Option<String>,

    #[serde(default)]
    feedback_message: Option<String>,

    #[serde(default)]
    feedback_webhook: Option<String>
}

fn read_manifest(repo_path: String, name: &str) -> Option<PluginManifest> {
    let manifest: PluginManifest = serde_json::from_str(&read_to_string(repo_path + &*format!("/stable/{name}/{name}.json")).unwrap()).unwrap();

    Some(manifest)
}

struct StatePlugin {
    name: String,
    time_built: chrono::DateTime<FixedOffset>
}

fn main() {
    let args = Args::parse();

    let config: HashMap<String, toml::Value> = toml::from_str(&read_to_string(args.repo_path.clone() + "/State.toml").unwrap()).unwrap();

    let mut plugin_names = vec![];

    for (channel, plugins) in config.get("channels").unwrap().as_table().unwrap() {
        for (plugin_name, plugin_data) in plugins.get("plugins").unwrap().as_table().unwrap() {
            let time_built_raw = *plugin_data.get("time_built").unwrap().as_datetime().unwrap();
            let time_built = chrono::DateTime::parse_from_rfc3339(&format!("{}Z", &time_built_raw.to_string())).unwrap();

            plugin_names.push(StatePlugin {
                name: plugin_name.clone(),
                time_built
            });
        }
    }

    let mut plugin_list: Vec<PluginManifest> = vec![];

    for plugin in plugin_names {
        let Some(mut manifest) = read_manifest(args.repo_path.to_string(), &plugin.name) else {
            println!("Could not parse plugin manifest.");
            continue;
        };

        // TODO: implement changelogs
        //manifest.changelog = "".to_string();
        manifest.last_update = plugin.time_built.timestamp_millis() / 1000;

        manifest.download_link_install = format!("https://dalamud.xiv.zone/stable/{}/latest.zip", plugin.name);
        // TODO: this aren't supposed to be the same
        manifest.download_link_testing = manifest.download_link_install.clone();
        manifest.download_link_update = manifest.download_link_install.clone();

        manifest.download_count = 0;
        manifest.is_dip17_plugin = true;
        manifest.dip17_channel = "stable".to_string();

        plugin_list.push(manifest);
    }

    write(args.repo_path + "/repo.json", serde_json::to_string(&plugin_list).unwrap());
}
