use std::collections::HashMap;
use std::fs::{read_to_string, write};
use chrono::FixedOffset;
use serde::{Deserialize, Serialize};
use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    repo_path: String,

    #[clap(short, long, value_parser)]
    download_host: String,
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
    #[serde(rename = "Changelog")]
    changelog: String,

    #[serde(default)]
    #[serde(rename = "CategoryTags")]
    category_tags: Option<Vec<String>>,

    #[serde(default)]
    #[serde(rename = "IsHide")]
    is_hide: bool,

    #[serde(default)]
    #[serde(rename = "TestingAssemblyVersion")]
    testing_assembly_version: Option<String>,

    #[serde(default)]
    #[serde(rename = "IsTestingExclusive")]
    is_testing_exclusive: bool,

    #[serde(default)]
    #[serde(rename = "DownloadCount")]
    download_count: i32,

    #[serde(default)]
    #[serde(rename = "LastUpdate")]
    last_update: i64,

    #[serde(default)]
    #[serde(rename = "DownloadLinkInstall")]
    download_link_install: String,

    #[serde(default)]
    #[serde(rename = "DownloadLinkUpdate")]
    download_link_update: String,

    #[serde(default)]
    #[serde(rename = "DownloadLinkTesting")]
    download_link_testing: String,

    #[serde(default)]
    #[serde(rename = "ImageUrls")]
    image_urls: Option<Vec<String>>,

    #[serde(default)]
    #[serde(rename = "IconUrl")]
    icon_url: Option<String>,

    #[serde(default)]
    #[serde(rename = "FeedbackMessage")]
    feedback_message: Option<String>,

    #[serde(default)]
    #[serde(rename = "FeedbackWebhook")]
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

    let config: HashMap<String, serde_json::Value> = serde_json::from_str(&read_to_string(args.repo_path.clone() + "/state.json").unwrap()).unwrap();

    let mut plugin_names = vec![];

    for (_, plugins) in config.get("Channels").unwrap().as_object().unwrap() {
        for (plugin_name, plugin_data) in plugins.get("Plugins").unwrap().as_object().unwrap() {
            let time_built_raw = plugin_data.get("TimeBuilt").unwrap().as_str().unwrap();
            let time_built = chrono::DateTime::parse_from_rfc3339(time_built_raw).unwrap();

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

        manifest.download_link_install = format!("{}/stable/{}/latest.zip", args.download_host, plugin.name);
        // TODO: this aren't supposed to be the same
        manifest.download_link_testing = manifest.download_link_install.clone();
        manifest.download_link_update = manifest.download_link_install.clone();

        manifest.download_count = 0;
        manifest.is_dip17_plugin = true;
        manifest.dip17_channel = "stable".to_string();

        plugin_list.push(manifest);
    }

    write(args.repo_path + "/repo.json", serde_json::to_string(&plugin_list).unwrap()).unwrap();
}
