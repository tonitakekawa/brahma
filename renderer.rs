// wgpuベースのレンダラ
// 最小プロトタイプとして、FSMの状態に応じて画面をクリアする色を変えるだけ
// 後でこのモジュールにシェーダパイプライン、ジオメトリ、テクスチャを足していく

use std::sync::Arc;
use winit::window::Window;

use crate::fsm::{State, World};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // wgpu Instance: バックエンド選択(Vulkan/Metal/GL等)
        // Linux ARM64では通常Vulkan。Asahi/HoneykrispのVulkan実装が使われる
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Surface: ウィンドウとの結合
        let surface = instance.create_surface(window).unwrap();

        // Adapter: 物理GPUの選択
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("適合するGPUアダプタが見つからない");

        log::info!("adapter: {:?}", adapter.get_info());

        // Device + Queue: 論理デバイスとコマンドキュー
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Bonten Device"),
                    required_features: wgpu::Features::empty(),
                    // 8GB RAMのM1 Airも視野に入れて、下限スペックで動くように
                    required_limits: wgpu::Limits::downlevel_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("デバイス取得失敗");

        // Surface configuration: フォーマット、サイズ、提示モード
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// 1フレーム描画
    /// 現状は世界の状態に応じた色でクリアするだけ
    pub fn render(&mut self, world: &World) {
        let clear_color = color_for_state(world.state(), world.time_in_state());

        let output = match self.surface.get_current_texture() {
            Ok(o) => o,
            Err(e) => {
                log::warn!("surface texture取得失敗: {:?}", e);
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });

        {
            // 描画パス: ここではclearするだけ。後でdraw_call群を足す
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_writes: None,
            });
            // ここに draw_call を追加していく
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

/// FSMの状態を背景色にマップする
/// 状態に入ってからの経過時間で滑らかに変化させる(プロトタイプの楽しみ)
fn color_for_state(state: State, t_in_state: f64) -> wgpu::Color {
    // 各状態の「中心色」を定義
    let (r, g, b) = match state {
        State::Void => (0.02, 0.02, 0.04),     // 深い藍黒 - 無
        State::Arising => (0.1, 0.3, 0.6),     // 青 - 起こり始め
        State::Present => (0.7, 0.6, 0.2),     // 金 - 在ること
        State::Ceasing => (0.4, 0.1, 0.15),    // 暗赤 - 滅
    };

    // 状態に入った瞬間に明るくし、時間経過で安定値へ収束させる
    // (キオスク作品としての"呼吸"のような変化を仮実装)
    let pulse = (-t_in_state * 0.8).exp() * 0.3;

    wgpu::Color {
        r: (r + pulse).min(1.0) as f64,
        g: (g + pulse).min(1.0) as f64,
        b: (b + pulse).min(1.0) as f64,
        a: 1.0,
    }
}
