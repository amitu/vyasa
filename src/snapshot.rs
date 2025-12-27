use crate::parser::{MantraDefinition, Repository};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const SNAPSHOT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptedEntry {
    pub mantra: String,
    pub commentary: String,
    pub hash: String,
    pub file: String,
    pub line: usize,
    pub accepted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u32,
    pub accepted: Vec<AcceptedEntry>,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            version: SNAPSHOT_VERSION,
            accepted: Vec::new(),
        }
    }
}

impl Snapshot {
    pub fn load(repo_root: &Path) -> Self {
        let snapshot_path = repo_root.join(".vyasa/snapshot.json");
        if let Ok(content) = fs::read_to_string(&snapshot_path) {
            if let Ok(snapshot) = serde_json::from_str(&content) {
                return snapshot;
            }
        }
        Self::default()
    }

    pub fn save(&self, repo_root: &Path) -> Result<(), String> {
        let vyasa_dir = repo_root.join(".vyasa");
        if !vyasa_dir.exists() {
            fs::create_dir_all(&vyasa_dir)
                .map_err(|e| format!("failed to create .vyasa directory: {}", e))?;
        }

        let snapshot_path = vyasa_dir.join("snapshot.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize snapshot: {}", e))?;
        fs::write(&snapshot_path, content)
            .map_err(|e| format!("failed to write snapshot: {}", e))?;
        Ok(())
    }

    pub fn is_accepted(&self, mantra: &str, commentary_hash: &str) -> bool {
        self.accepted
            .iter()
            .any(|e| e.mantra == mantra && e.hash == commentary_hash)
    }

    pub fn find_entry(&self, mantra: &str, commentary_hash: &str) -> Option<&AcceptedEntry> {
        self.accepted
            .iter()
            .find(|e| e.mantra == mantra && e.hash == commentary_hash)
    }

    pub fn add_entry(&mut self, entry: AcceptedEntry) {
        // remove any existing entry with same mantra+hash
        self.accepted
            .retain(|e| !(e.mantra == entry.mantra && e.hash == entry.hash));
        self.accepted.push(entry);
    }
}

pub fn compute_hash(commentary: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(commentary.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}

#[derive(Debug)]
pub enum DefinitionStatus {
    Accepted,
    New,
    Changed { old_commentary: String },
}

#[derive(Debug)]
pub struct DefinitionWithStatus {
    pub definition: MantraDefinition,
    pub status: DefinitionStatus,
    pub hash: String,
}

pub fn compare_with_snapshot(repo: &Repository, snapshot: &Snapshot) -> Vec<DefinitionWithStatus> {
    let mut results = Vec::new();

    // build a lookup for snapshot entries by mantra text
    let mut snapshot_by_mantra: HashMap<&str, Vec<&AcceptedEntry>> = HashMap::new();
    for entry in &snapshot.accepted {
        snapshot_by_mantra
            .entry(&entry.mantra)
            .or_default()
            .push(entry);
    }

    for def in &repo.definitions {
        let hash = compute_hash(&def.commentary);

        if snapshot.is_accepted(&def.mantra_text, &hash) {
            results.push(DefinitionWithStatus {
                definition: def.clone(),
                status: DefinitionStatus::Accepted,
                hash,
            });
        } else {
            // check if there's an accepted entry with same mantra but different hash
            // (at same file:line, meaning commentary changed)
            let changed_entry = snapshot_by_mantra
                .get(def.mantra_text.as_str())
                .and_then(|entries| {
                    entries
                        .iter()
                        .find(|e| e.file == def.file && e.line == def.line)
                })
                .cloned();

            if let Some(old_entry) = changed_entry {
                results.push(DefinitionWithStatus {
                    definition: def.clone(),
                    status: DefinitionStatus::Changed {
                        old_commentary: old_entry.commentary.clone(),
                    },
                    hash,
                });
            } else {
                results.push(DefinitionWithStatus {
                    definition: def.clone(),
                    status: DefinitionStatus::New,
                    hash,
                });
            }
        }
    }

    results
}

pub fn pending_definitions(statuses: &[DefinitionWithStatus]) -> Vec<&DefinitionWithStatus> {
    statuses
        .iter()
        .filter(|s| !matches!(s.status, DefinitionStatus::Accepted))
        .collect()
}

pub fn create_entry(def: &MantraDefinition, hash: &str) -> AcceptedEntry {
    AcceptedEntry {
        mantra: def.mantra_text.clone(),
        commentary: def.commentary.clone(),
        hash: hash.to_string(),
        file: def.file.clone(),
        line: def.line,
        accepted_at: Utc::now(),
    }
}
