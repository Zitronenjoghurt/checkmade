#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct MoveHistoryState {
    /// None = Live
    /// Some(0) = Start position
    /// Some(n) = Position after move n
    index: Option<usize>,
}

impl MoveHistoryState {
    pub fn is_at_present(&self) -> bool {
        self.index.is_none()
    }

    pub fn current_index(&self) -> Option<usize> {
        self.index
    }

    pub fn go_to(&mut self, index: usize, total_moves: usize) {
        if index >= total_moves {
            self.index = None;
        } else {
            self.index = Some(index);
        }
    }

    pub fn snap_to_present(&mut self) {
        self.index = None;
    }

    pub fn go_back(&mut self, total_moves: usize) {
        let current = self.index.unwrap_or(total_moves);
        if current > 0 {
            self.index = Some(current - 1);
        }
    }

    pub fn go_forward(&mut self, total_moves: usize) {
        let Some(idx) = self.index else { return };
        if idx + 1 >= total_moves {
            self.index = None;
        } else {
            self.index = Some(idx + 1);
        }
    }
}
