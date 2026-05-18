// 梵天OSのFSM核
// プラットフォーム非依存。実機・VM・WASM(将来)のすべてで同じコードが動く
//
// 最小プロトタイプとして、Lao Tzu的な生成的構造を取る:
//   Void (無) → Arising (一) → Present (二) → Ceasing (三) → Void (...)
//
// この4状態は後で増減・改変可能。とりあえず動かすための叩き台。

/// FSMが取りうる状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// 無 — 何も生じていない
    Void,
    /// 起 — 何かが生じつつある
    Arising,
    /// 在 — 在り続けている
    Present,
    /// 滅 — 滅しつつある
    Ceasing,
}

/// 外部からFSMに入ってくる入力
#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    /// 次の状態へ進む(Space)
    Advance,
    /// Voidに戻す(R)
    Reset,
}

/// 世界の状態。FSMと、状態に紐づくデータを保持する
pub struct World {
    state: State,
    /// この状態に入った時刻(秒)
    entered_at: f64,
    /// 現在時刻(秒) - step()で更新
    now: f64,
}

impl World {
    pub fn new() -> Self {
        Self {
            state: State::Void,
            entered_at: 0.0,
            now: 0.0,
        }
    }

    /// 現在の状態を返す(描画側が参照する)
    pub fn state(&self) -> State {
        self.state
    }

    /// 現在の状態に入ってからの経過秒数
    pub fn time_in_state(&self) -> f64 {
        (self.now - self.entered_at).max(0.0)
    }

    /// 1フレーム分の状態進行
    /// 時間駆動の自動遷移はここで判定する
    pub fn step(&mut self, now: f64) {
        self.now = now;

        // 自動遷移の例: Arisingは2秒経過するとPresentに自動遷移
        // (この遷移ルールは仮置き。後で哲学的構造に合わせて練る)
        let t = self.time_in_state();
        let next = match self.state {
            State::Arising if t >= 2.0 => Some(State::Present),
            State::Ceasing if t >= 2.0 => Some(State::Void),
            _ => None,
        };

        if let Some(next_state) = next {
            self.transition_to(next_state);
        }
    }

    /// 外部入力に応じた遷移
    pub fn handle_input(&mut self, event: InputEvent, _now: f64) {
        let next = match (self.state, event) {
            // SpaceでVoid→Arising、Present→Ceasing
            (State::Void, InputEvent::Advance) => Some(State::Arising),
            (State::Present, InputEvent::Advance) => Some(State::Ceasing),
            // Rでいつでも初期状態へ
            (_, InputEvent::Reset) => Some(State::Void),
            _ => None,
        };

        if let Some(next_state) = next {
            self.transition_to(next_state);
        }
    }

    fn transition_to(&mut self, next: State) {
        log::info!("transition: {:?} → {:?}", self.state, next);
        self.state = next;
        self.entered_at = self.now;
    }
}
