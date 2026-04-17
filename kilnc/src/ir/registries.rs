use std::collections::HashMap;

use crate::ir::{FuncId, GlobalId, IrFuncDecl, IrGlobal, IrGlobalInit, LabelId};

#[derive(Default, Clone)]
pub struct StringTable {
    entries: Vec<String>,
    indices: HashMap<String, usize>,
}

impl StringTable {
    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(&id) = self.indices.get(s) {
            return id;
        }

        let id = self.entries.len();
        let owned = s.to_string();
        self.entries.push(owned.clone());
        self.indices.insert(owned, id);
        id
    }

    pub fn label_for(&self, id: usize) -> String {
        format!("__str_{}", id)
    }

    pub fn get(&self, id: usize) -> Option<&str> {
        self.entries.get(id).map(|s| s.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &str)> {
        self.entries
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s.as_str()))
    }
}

impl std::fmt::Debug for StringTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.entries.iter().enumerate())
            .finish()
    }
}

#[derive(Default, Clone)]
pub struct Globals {
    entries: Vec<IrGlobal>,
    indices: HashMap<String, GlobalId>,
}

impl Globals {
    pub fn declare(
        &mut self,
        name: impl Into<String>,
        init: IrGlobalInit,
        is_const: bool,
    ) -> GlobalId {
        let name = name.into();
        if let Some(&id) = self.indices.get(&name) {
            return id;
        }

        let id = self.entries.len() as GlobalId;
        self.entries.push(IrGlobal {
            id,
            init,
            is_const,
            name: name.clone(),
        });
        self.indices.insert(name, id);
        id
    }

    pub fn label_for(&self, id: GlobalId) -> String {
        format!("__global_{}", id)
    }

    pub fn get(&self, name: &str) -> Option<&IrGlobal> {
        self.indices
            .get(name)
            .and_then(|&id| self.entries.get(id as usize))
    }

    pub fn get_by_id(&self, id: u32) -> Option<&IrGlobal> {
        self.entries.get(id as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = (GlobalId, &IrGlobal)> {
        self.entries
            .iter()
            .map(|g| (g.id, g))
    }
}

impl std::fmt::Debug for Globals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.entries.iter().map(|g| (g.id, g)))
            .finish()
    }
}

#[derive(Default, Clone)]
pub struct Functions {
    entries: Vec<IrFuncDecl>,
    indices: HashMap<String, FuncId>,
}

impl Functions {
    pub fn declare(&mut self, name: impl Into<String>, param_count: usize) -> FuncId {
        let name = name.into();

        if let Some(&id) = self.indices.get(&name) {
            return id;
        }

        let id = self.entries.len() as FuncId;

        self.entries.push(IrFuncDecl {
            id,
            name: name.clone(),
            param_count,
            entry_label: 0,
        });

        self.indices.insert(name, id);
        id
    }

    pub fn set_entry_label(&mut self, id: FuncId, label: LabelId) {
        self.entries[id as usize].entry_label = label;
    }

    pub fn label_for(&self, id: FuncId) -> String {
        format!("__func_{}", id)
    }

    pub fn get(&self, name: &str) -> Option<&IrFuncDecl> {
        self.indices
            .get(name)
            .and_then(|&id| self.entries.get(id as usize))
    }

    pub fn get_by_id(&self, id: u32) -> Option<&IrFuncDecl> {
        self.entries.get(id as usize)
    }

    pub fn get_id_by_name(&self, name: &str) -> Option<FuncId> {
        self.indices.get(name).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (FuncId, &IrFuncDecl)> {
        self.entries
            .iter()
            .map(|d| (d.id, d))
    }
}

impl std::fmt::Debug for Functions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.entries.iter().map(|d| (d.id, d)))
            .finish()
    }
}
