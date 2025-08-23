use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::SystemTime;
use widestring::{U16CStr, U16Str, U16String};

use crate::util::{
    normalize_and_canonicalize_for_key, normalize_candidate_path, unix_ts_to_system_time,
};

#[derive(Debug, Clone)]
pub struct EntryMeta {
    pub mft_no: u64,
    pub is_dir: bool,
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub name_u16: U16String,
}

#[derive(Debug, Clone)]
pub enum EntryOrDir {
    Dir,
    File(EntryMeta),
}

#[derive(Default)]
pub struct DeletedIndex {
    pub nodes: HashMap<String, EntryOrDir>,
    pub children_names: HashMap<String, Vec<U16String>>,
    pub children_ci: HashMap<String, HashSet<String>>,
}

// 削除済みファイルのインデックス構築
impl DeletedIndex {
    pub fn ensure_dirs_from_root(&mut self, dir_path_str: &str) {
        let norm = normalize_candidate_path(dir_path_str);
        let comps: Vec<&str> = norm.split('\\').filter(|s| !s.is_empty()).collect();

        let mut parent_key = normalize_and_canonicalize_for_key("\\");
        self.nodes
            .entry(parent_key.clone())
            .or_insert(EntryOrDir::Dir);
        self.children_names
            .entry(parent_key.clone())
            .or_insert_with(Vec::new);
        self.children_ci
            .entry(parent_key.clone())
            .or_insert_with(HashSet::new);

        let mut acc = String::from("\\");
        for comp in comps {
            let comp_u16 = U16String::from_str(comp);
            let lower = comp_u16.to_string_lossy().to_lowercase();
            let names = self
                .children_names
                .entry(parent_key.clone())
                .or_insert_with(Vec::new);
            let set = self
                .children_ci
                .entry(parent_key.clone())
                .or_insert_with(HashSet::new);
            if set.insert(lower) {
                names.push(comp_u16.clone());
            }
            if acc == "\\" {
                acc = format!("\\{}", comp);
            } else {
                acc = format!("{}\\{}", acc, comp);
            }
            // エントリを正規化してから処理
            let dir_key = normalize_and_canonicalize_for_key(&acc);
            self.nodes.entry(dir_key.clone()).or_insert(EntryOrDir::Dir);
            self.children_names
                .entry(dir_key.clone())
                .or_insert_with(Vec::new);
            self.children_ci
                .entry(dir_key.clone())
                .or_insert_with(HashSet::new);
            parent_key = dir_key;
        }
    }

    // 同じ名前の削除済みファイルがあった場合、連番を付けてユニークな名前を生成
    fn unique_child_name(&mut self, parent_key: &str, desired: &U16String) -> U16String {
        let set = self
            .children_ci
            .entry(parent_key.to_string())
            .or_insert_with(HashSet::new);
        let names = self
            .children_names
            .entry(parent_key.to_string())
            .or_insert_with(Vec::new);
        let mut candidate = desired.clone();
        let mut lower = candidate.to_string_lossy().to_lowercase();
        if set.contains(&lower) {
            let mut n = 2u32;
            loop {
                let s = candidate.to_string_lossy();
                if let Some(dot) = s.rfind('.') {
                    if dot > 0 {
                        let (base, ext) = s.split_at(dot);
                        // 例: file.txt -> file_2.txt
                        candidate = U16String::from_str(&format!("{base}_{n}{ext}"));
                    } else {
                        candidate = U16String::from_str(&format!("{}_{}", s, n));
                    }
                } else {
                    candidate = U16String::from_str(&format!("{}_{}", s, n));
                }
                lower = candidate.to_string_lossy().to_lowercase();
                if !set.contains(&lower) {
                    break;
                }
                n += 1;
            }
        }
        set.insert(lower);
        names.push(candidate.clone());
        candidate
    }

    pub fn insert_file(&mut self, full_path_u16: &U16String, mut meta: EntryMeta) {
        let full_str = normalize_candidate_path(&full_path_u16.to_string_lossy());
        let parent_str = {
            let pb = PathBuf::from(&full_str);
            pb.parent()
                .map(|x| normalize_candidate_path(&x.to_string_lossy()))
                .unwrap_or_else(|| "\\".to_string())
        };
        self.ensure_dirs_from_root(&parent_str);
        let parent_key = normalize_and_canonicalize_for_key(&parent_str);
        let base = basename_u16str(U16Str::from_slice(
            U16String::from_str(&full_str).as_slice(),
        ));
        let unique = self.unique_child_name(&parent_key, &base);
        meta.name_u16 = unique.clone();
        let full_shown = if parent_str == "\\" {
            format!(r"\{}", unique.to_string_lossy())
        } else {
            format!(r"{}\{}", parent_str, unique.to_string_lossy())
        };
        let key = normalize_and_canonicalize_for_key(&full_shown);
        self.nodes.insert(key, EntryOrDir::File(meta));
    }

    pub fn insert_dir(&mut self, full_dir_u16: &U16String) {
        let dir_str = normalize_candidate_path(&full_dir_u16.to_string_lossy());
        self.ensure_dirs_from_root(&dir_str);
    }

    pub fn get(&self, key_lc: &str) -> Option<&EntryOrDir> {
        self.nodes.get(key_lc)
    }

    pub fn list_children(&self, dir_key_lc: &str) -> Vec<U16String> {
        self.children_names
            .get(dir_key_lc)
            .cloned()
            .unwrap_or_default()
    }
}

fn basename_u16str(full: &U16Str) -> U16String {
    let slice = full.as_slice();
    if let Some(pos) = slice.iter().rposition(|c| *c == b'\\' as u16) {
        U16String::from_vec(slice[pos + 1..].to_vec())
    } else {
        U16String::from_vec(slice.to_vec())
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub mft_no: u64,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub created: Option<i64>,
    pub modified: Option<i64>,
    pub accessed: Option<i64>,
}

pub fn apply_staging(
    idx: &mut DeletedIndex,
    staging: &mut Vec<Candidate>,
    found_counter: &std::sync::Arc<std::sync::atomic::AtomicU64>,
) {
    use std::sync::atomic::Ordering;
    if staging.is_empty() {
        return;
    }
    for c in staging.drain(..) {
        let mut full_str = normalize_candidate_path(&c.path);
        if is_basename_only(&full_str) {
            let base = &full_str[1..];
            if !base.is_empty() {
                // 元々のパスが不明な場合、直下の「fakepath」というディレクトリに保存
                full_str = format!(r"\fakepath\{}", base);
            }
        }
        let full_u16 = U16String::from_str(&full_str);
        if c.is_dir {
            idx.insert_dir(&full_u16);
        } else {
            let meta = EntryMeta {
                mft_no: c.mft_no,
                is_dir: false,
                size: c.size,
                created: c.created.map(unix_ts_to_system_time),
                modified: c.modified.map(unix_ts_to_system_time),
                accessed: c.accessed.map(unix_ts_to_system_time),
                name_u16: basename_u16str(U16Str::from_slice(full_u16.as_slice())),
            };
            idx.insert_file(&full_u16, meta);
        }
        found_counter.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn is_basename_only(norm_full: &str) -> bool {
    if !norm_full.starts_with('\\') {
        return false;
    }
    !norm_full[1..].contains('\\')
}

pub fn path_key_lc_from_u16(full: &U16CStr) -> String {
    let s = U16String::from_vec(full.as_slice().to_vec()).to_string_lossy();
    normalize_and_canonicalize_for_key(&s)
}
