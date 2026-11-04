//! Launcher self-update. The Tauri updater plugin is configured here, not in the page: the
//! endpoint (`latest.json`) comes from the release source (GitHub owner or static mirror), which
//! the JS API of the plugin cannot set. The signature check against the public key in
//! `tauri.conf.json` is the plugin's and stays on whatever the source is.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, State, ipc::Channel};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::commands::{App, Res};

/// The update a check found, kept until `update_install` takes it.
#[derive(Default)]
pub struct PendingUpdate(Mutex<Option<Update>>);

#[derive(Serialize)]
pub struct UpdateMeta {
    pub version: String,
    pub current_version: String,
    pub date: Option<String>,
    pub body: Option<String>,
}

/// Same shape as the plugin's own events, so the page code is unchanged.
#[derive(Serialize, Clone)]
#[serde(tag = "event", content = "data")]
pub enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        content_length: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        chunk_length: usize,
    },
    Finished,
}

/// Looks for a newer launcher at the release source's `latest.json`.
#[tauri::command]
pub async fn update_check(
    handle: AppHandle,
    app: App<'_>,
    pending: State<'_, PendingUpdate>,
) -> Res<Option<UpdateMeta>> {
    let endpoint = app.release_source().updater_endpoint();
    let url = reqwest::Url::parse(&endpoint).map_err(|e| format!("{endpoint}: {e}"))?;
    if url.scheme() == "file" {
        return Err(
            "the launcher cannot update from a file:// mirror; serve the mirror folder with a web \
             server (https) or pick another release source"
                .into(),
        );
    }
    let updater = handle
        .updater_builder()
        .endpoints(vec![url])
        .and_then(|b| b.build())
        .map_err(|e| e.to_string())?;
    let found = updater.check().await.map_err(|e| e.to_string())?;
    let meta = found.as_ref().map(|u| UpdateMeta {
        version: u.version.clone(),
        current_version: u.current_version.clone(),
        date: u.date.map(|d| d.to_string()),
        body: u.body.clone(),
    });
    *pending.0.lock().unwrap() = found;
    Ok(meta)
}

/// Downloads, verifies the signature and installs the update `update_check` found.
#[tauri::command]
pub async fn update_install(
    pending: State<'_, PendingUpdate>,
    on_event: Channel<DownloadEvent>,
) -> Res<()> {
    let update = pending
        .0
        .lock()
        .unwrap()
        .take()
        .ok_or("no update was found; check for updates first")?;
    let mut started = false;
    let progress = on_event.clone();
    update
        .download_and_install(
            move |chunk, total| {
                if !started {
                    started = true;
                    let _ = progress.send(DownloadEvent::Started {
                        content_length: total,
                    });
                }
                let _ = progress.send(DownloadEvent::Progress {
                    chunk_length: chunk,
                });
            },
            move || {
                let _ = on_event.send(DownloadEvent::Finished);
            },
        )
        .await
        .map_err(|e| e.to_string())
}
