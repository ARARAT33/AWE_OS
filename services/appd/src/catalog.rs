//! Native AWE application catalog. Application code remains user-space.

use crate::{AWE_APP_ABI_MAJOR, AWE_APP_ABI_MINOR, AppId, AppManifest};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinApp {
    pub id: AppId,
    pub name: &'static str,
    pub memory_limit_pages: u32,
    pub capability_mask: u64,
}

pub const BUILTIN_APPS: [BuiltinApp; 8] = [
    BuiltinApp {
        id: AppId(1),
        name: "awe-shell",
        memory_limit_pages: 4096,
        capability_mask: 0x0001,
    },
    BuiltinApp {
        id: AppId(2),
        name: "awe-files",
        memory_limit_pages: 4096,
        capability_mask: 0x0003,
    },
    BuiltinApp {
        id: AppId(3),
        name: "awe-terminal",
        memory_limit_pages: 2048,
        capability_mask: 0x0001,
    },
    BuiltinApp {
        id: AppId(4),
        name: "awe-settings",
        memory_limit_pages: 2048,
        capability_mask: 0x0001,
    },
    BuiltinApp {
        id: AppId(5),
        name: "awe-monitor",
        memory_limit_pages: 2048,
        capability_mask: 0x0005,
    },
    BuiltinApp {
        id: AppId(6),
        name: "awe-store",
        memory_limit_pages: 4096,
        capability_mask: 0x0009,
    },
    BuiltinApp {
        id: AppId(7),
        name: "awe-network",
        memory_limit_pages: 4096,
        capability_mask: 0x0011,
    },
    BuiltinApp {
        id: AppId(8),
        name: "awe-devkit",
        memory_limit_pages: 8192,
        capability_mask: 0x0021,
    },
];

pub const fn manifest(app: BuiltinApp) -> AppManifest {
    AppManifest {
        id: app.id,
        abi_major: AWE_APP_ABI_MAJOR,
        abi_minor: AWE_APP_ABI_MINOR,
        memory_limit_pages: app.memory_limit_pages,
        capability_mask: app.capability_mask,
        dependency_count: 0,
        resource_count: 0,
    }
}
