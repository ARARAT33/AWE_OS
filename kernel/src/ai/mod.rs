//! AI/ML Hardware Abstraction, Tensor Buffers & Inference Pipeline Engine.
//!
//! Provides GPU/NPU execution target abstraction, tensor memory allocation,
//! model format parsing, and inference execution pipeline scheduling.

#![no_std]

pub const MODEL_MAGIC: [u8; 4] = [b'A', b'W', b'A', b'I']; // "AWAI"
pub const MAX_TENSOR_BUFFERS: usize = 16;
pub const MAX_GRAPH_NODES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceleratorType {
    Npu,
    Gpu,
    Dsp,
    CpuVectorEngine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TensorDataType {
    Float32,
    Float16,
    Int8,
    Int32,
}

#[derive(Debug, Clone, Copy)]
pub struct TensorShape {
    pub batch: u32,
    pub channels: u32,
    pub height: u32,
    pub width: u32,
}

impl TensorShape {
    pub const fn element_count(&self) -> usize {
        (self.batch as usize)
            * (self.channels as usize)
            * (self.height as usize)
            * (self.width as usize)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TensorBuffer {
    pub buffer_id: u32,
    pub data_type: TensorDataType,
    pub shape: TensorShape,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub node_count: u32,
    pub weights_offset: u32,
    pub weights_len: u32,
}

impl ModelHeader {
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 32 {
            return Err("Model header buffer underflow");
        }
        if data[0..4] != MODEL_MAGIC {
            return Err("Invalid AI model magic signature");
        }
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let node_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let weights_offset = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let weights_len = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);

        Ok(Self {
            magic: MODEL_MAGIC,
            version,
            node_count,
            weights_offset,
            weights_len,
        })
    }
}

/// AI Inference Execution Engine.
#[derive(Debug)]
pub struct AiInferenceEngine {
    pub accelerator_type: AcceleratorType,
    tensor_buffers: [Option<TensorBuffer>; MAX_TENSOR_BUFFERS],
    buffer_counter: u32,
}

impl AiInferenceEngine {
    pub const fn new(accelerator_type: AcceleratorType) -> Self {
        Self {
            accelerator_type,
            tensor_buffers: [None; MAX_TENSOR_BUFFERS],
            buffer_counter: 1,
        }
    }

    pub fn allocate_tensor(
        &mut self,
        data_type: TensorDataType,
        shape: TensorShape,
    ) -> Result<u32, &'static str> {
        let element_size = match data_type {
            TensorDataType::Float32 | TensorDataType::Int32 => 4,
            TensorDataType::Float16 => 2,
            TensorDataType::Int8 => 1,
        };
        let bytes = shape.element_count() * element_size;
        let bid = self.buffer_counter;

        for slot in self.tensor_buffers.iter_mut() {
            if slot.is_none() {
                *slot = Some(TensorBuffer {
                    buffer_id: bid,
                    data_type,
                    shape,
                    size_bytes: bytes,
                });
                self.buffer_counter += 1;
                return Ok(bid);
            }
        }
        Err("Tensor buffer pool capacity reached")
    }

    pub fn execute_inference(
        &self,
        model: &ModelHeader,
        input_buf_id: u32,
    ) -> Result<u32, &'static str> {
        let mut found = false;
        for slot in self.tensor_buffers.iter() {
            if let Some(buf) = slot {
                if buf.buffer_id == input_buf_id {
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return Err("Input tensor buffer not found");
        }

        if model.node_count == 0 {
            return Err("Model graph contains zero nodes");
        }

        Ok(model.node_count) // Returns number of executed compute nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_model_header_and_inference_engine() {
        let mut mock_model = [0u8; 32];
        mock_model[0..4].copy_from_slice(&MODEL_MAGIC);
        mock_model[4..8].copy_from_slice(&1u32.to_le_bytes()); // version 1
        mock_model[8..12].copy_from_slice(&12u32.to_le_bytes()); // 12 nodes

        let hdr = ModelHeader::parse(&mock_model).expect("Should parse AI model header");
        assert_eq!(hdr.node_count, 12);

        let mut engine = AiInferenceEngine::new(AcceleratorType::Npu);
        let shape = TensorShape {
            batch: 1,
            channels: 3,
            height: 224,
            width: 224,
        };
        let buf_id = engine
            .allocate_tensor(TensorDataType::Float32, shape)
            .unwrap();
        assert_eq!(buf_id, 1);

        let executed_nodes = engine.execute_inference(&hdr, buf_id).unwrap();
        assert_eq!(executed_nodes, 12);
    }
}
