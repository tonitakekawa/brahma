// 梵天OS最小プロトタイプ
// VM環境(UTM上のFedora ARM64)で動作することを想定
// winit + wgpuでウィンドウを開き、FSMの状態に応じて画面を描画する

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod fsm;
mod renderer;

use fsm::{InputEvent, World};
use renderer::Renderer;

// アプリケーションの全体状態
// winit 0.30のApplicationHandlerパターン
struct App {
    // resumedで初期化されるのでOption
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    world: World,
    start_time: std::time::Instant,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            world: World::new(),
            start_time: std::time::Instant::now(),
        }
    }

    fn elapsed_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // ウィンドウとレンダラを初期化
        let window_attrs = Window::default_attributes()
            .with_title("梵天 - Bonten")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        // wgpuの初期化は非同期なのでpollster::block_onで同期化
        let renderer = pollster::block_on(Renderer::new(window.clone()));

        self.window = Some(window.clone());
        self.renderer = Some(renderer);
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                // キー入力をFSMの入力イベントに変換
                let input = match code {
                    KeyCode::Space => Some(InputEvent::Advance),
                    KeyCode::KeyR => Some(InputEvent::Reset),
                    KeyCode::Escape => {
                        event_loop.exit();
                        None
                    }
                    _ => None,
                };
                if let Some(ev) = input {
                    self.world.handle_input(ev, self.elapsed_seconds());
                }
            }
            WindowEvent::RedrawRequested => {
                // 状態更新と描画
                let now = self.elapsed_seconds();
                self.world.step(now);

                if let Some(renderer) = &mut self.renderer {
                    renderer.render(&self.world);
                }

                // 連続描画のために再リクエスト
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
