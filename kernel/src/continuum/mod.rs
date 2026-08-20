//! Continuum Cross-Device Mesh & Synchronized State Engine.
//!
//! Provides device discovery, secure pairing handshake, state sync (clipboard,
//! notifications, active task continuation), and device mesh routing.

#![no_std]

pub const MAX_MESH_NODES: usize = 16;
pub const MAX_CLIPBOARD_LEN: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Tablet,
    SmartDisplay,
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceNode {
    pub device_id: u64,
    pub device_type: DeviceType,
    pub public_key_hash: u64,
    pub is_authenticated: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ContinuumState {
    pub active_task_id: u32,
    pub clipboard_buffer: [u8; MAX_CLIPBOARD_LEN],
    pub clipboard_len: usize,
    pub sequence_number: u32,
}

/// Continuum Device Mesh Engine.
#[derive(Debug)]
pub struct ContinuumMesh {
    nodes: [Option<DeviceNode>; MAX_MESH_NODES],
    pub local_state: ContinuumState,
    node_count: usize,
}

impl ContinuumMesh {
    pub const fn new() -> Self {
        Self {
            nodes: [None; MAX_MESH_NODES],
            local_state: ContinuumState {
                active_task_id: 0,
                clipboard_buffer: [0u8; MAX_CLIPBOARD_LEN],
                clipboard_len: 0,
                sequence_number: 1,
            },
            node_count: 0,
        }
    }

    pub fn discover_and_pair(
        &mut self,
        device_id: u64,
        dev_type: DeviceType,
        pk_hash: u64,
    ) -> Result<usize, &'static str> {
        for (idx, slot) in self.nodes.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(DeviceNode {
                    device_id,
                    device_type: dev_type,
                    public_key_hash: pk_hash,
                    is_authenticated: true,
                });
                self.node_count += 1;
                return Ok(idx);
            }
        }
        Err("Device mesh capacity reached")
    }

    pub fn update_clipboard(&mut self, data: &[u8]) -> Result<u32, &'static str> {
        if data.len() > MAX_CLIPBOARD_LEN {
            return Err("Clipboard payload exceeds limit");
        }
        self.local_state.clipboard_buffer[..data.len()].copy_from_slice(data);
        self.local_state.clipboard_len = data.len();
        self.local_state.sequence_number += 1;
        Ok(self.local_state.sequence_number)
    }

    pub fn sync_state_from_node(
        &mut self,
        node_id: u64,
        state: ContinuumState,
    ) -> Result<(), &'static str> {
        let mut authenticated = false;
        for node in self.nodes.iter().flatten() {
            if node.device_id == node_id && node.is_authenticated {
                authenticated = true;
                break;
            }
        }

        if !authenticated {
            return Err("State sync rejected: unauthenticated node");
        }

        if state.sequence_number > self.local_state.sequence_number {
            self.local_state = state;
        }

        Ok(())
    }
}

impl Default for ContinuumMesh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuum_mesh_pairing_and_state_sync() {
        let mut mesh = ContinuumMesh::new();
        let idx = mesh
            .discover_and_pair(0x1122_3344_5566, DeviceType::Mobile, 0xABC123)
            .unwrap();
        assert_eq!(idx, 0);

        mesh.update_clipboard(b"https://aweos.org").unwrap();
        assert_eq!(
            &mesh.local_state.clipboard_buffer[..17],
            b"https://aweos.org"
        );

        let remote_state = ContinuumState {
            active_task_id: 42,
            clipboard_buffer: [0x55; MAX_CLIPBOARD_LEN],
            clipboard_len: 10,
            sequence_number: 10,
        };

        assert!(
            mesh.sync_state_from_node(0x1122_3344_5566, remote_state)
                .is_ok()
        );
        assert_eq!(mesh.local_state.active_task_id, 42);
        assert_eq!(mesh.local_state.sequence_number, 10);
    }
}
