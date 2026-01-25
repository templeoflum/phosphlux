//! Multi-Channel Texture System
//!
//! Provides multiple independent signal channels for complex routing.
//! Each channel is a separate texture that can be read/written by nodes.
//!
//! Channels:
//! - Main: Primary output signal
//! - Feedback: Previous frame for feedback effects
//! - Key: Luminance key/matte signal
//! - Aux1/Aux2: General purpose auxiliary channels

use wgpu;

/// Channel identifiers
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ChannelId {
    /// Primary output channel
    Main = 0,
    /// Feedback from previous frame
    Feedback = 1,
    /// Luminance key/matte channel
    Key = 2,
    /// First auxiliary channel
    Aux1 = 3,
    /// Second auxiliary channel
    Aux2 = 4,
}

impl ChannelId {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(ChannelId::Main),
            1 => Some(ChannelId::Feedback),
            2 => Some(ChannelId::Key),
            3 => Some(ChannelId::Aux1),
            4 => Some(ChannelId::Aux2),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ChannelId::Main => "Main",
            ChannelId::Feedback => "Feedback",
            ChannelId::Key => "Key",
            ChannelId::Aux1 => "Aux 1",
            ChannelId::Aux2 => "Aux 2",
        }
    }
}

/// Number of channels in the system
pub const CHANNEL_COUNT: usize = 5;

/// Textures for a single channel (ping-pong for feedback)
pub struct ChannelTextures {
    /// Current frame texture (read)
    pub read_texture: wgpu::Texture,
    pub read_view: wgpu::TextureView,
    /// Next frame texture (write)
    pub write_texture: wgpu::Texture,
    pub write_view: wgpu::TextureView,
}

impl ChannelTextures {
    /// Create a new channel texture pair
    pub fn new(device: &wgpu::Device, width: u32, height: u32, label: &str) -> Self {
        let texture_desc = wgpu::TextureDescriptor {
            label: Some(&format!("{} Read", label)),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let read_texture = device.create_texture(&texture_desc);
        let read_view = read_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let write_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{} Write", label)),
            ..texture_desc
        });
        let write_view = write_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            read_texture,
            read_view,
            write_texture,
            write_view,
        }
    }

    /// Swap read and write textures (for ping-pong feedback)
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.read_texture, &mut self.write_texture);
        std::mem::swap(&mut self.read_view, &mut self.write_view);
    }

    /// Resize textures
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32, label: &str) {
        *self = Self::new(device, width, height, label);
    }
}

/// Manager for all signal channels
pub struct ChannelManager {
    /// All channel textures
    pub channels: [ChannelTextures; CHANNEL_COUNT],
    /// Current dimensions
    pub width: u32,
    pub height: u32,
}

impl ChannelManager {
    /// Create a new channel manager
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let channels = [
            ChannelTextures::new(device, width, height, "Main Channel"),
            ChannelTextures::new(device, width, height, "Feedback Channel"),
            ChannelTextures::new(device, width, height, "Key Channel"),
            ChannelTextures::new(device, width, height, "Aux1 Channel"),
            ChannelTextures::new(device, width, height, "Aux2 Channel"),
        ];

        Self {
            channels,
            width,
            height,
        }
    }

    /// Get a channel by ID
    pub fn get(&self, id: ChannelId) -> &ChannelTextures {
        &self.channels[id as usize]
    }

    /// Get a mutable channel by ID
    pub fn get_mut(&mut self, id: ChannelId) -> &mut ChannelTextures {
        &mut self.channels[id as usize]
    }

    /// Swap all channel read/write buffers (call at end of frame)
    pub fn swap_all(&mut self) {
        for channel in &mut self.channels {
            channel.swap();
        }
    }

    /// Swap only the feedback channel
    pub fn swap_feedback(&mut self) {
        self.channels[ChannelId::Feedback as usize].swap();
    }

    /// Resize all channels
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == self.width && height == self.height {
            return;
        }

        let labels = ["Main", "Feedback", "Key", "Aux1", "Aux2"];
        for (i, channel) in self.channels.iter_mut().enumerate() {
            channel.resize(device, width, height, labels[i]);
        }

        self.width = width;
        self.height = height;
    }

    /// Get read views for all channels (for shader binding)
    pub fn read_views(&self) -> [&wgpu::TextureView; CHANNEL_COUNT] {
        [
            &self.channels[0].read_view,
            &self.channels[1].read_view,
            &self.channels[2].read_view,
            &self.channels[3].read_view,
            &self.channels[4].read_view,
        ]
    }
}
