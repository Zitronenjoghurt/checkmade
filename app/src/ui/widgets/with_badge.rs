enum Badge {
    None,
    Dot,
    Count(usize),
}

pub struct WithBadge<W> {
    inner: W,
    badge: Badge,
    color: egui::Color32,
}

impl<W> WithBadge<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            badge: Badge::None,
            color: egui::Color32::RED,
        }
    }

    pub fn dot(mut self, show: bool) -> Self {
        self.badge = if show { Badge::Dot } else { Badge::None };
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.badge = if count > 0 {
            Badge::Count(count)
        } else {
            Badge::None
        };
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = color;
        self
    }
}

impl<W: egui::Widget> egui::Widget for WithBadge<W> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = self.inner.ui(ui);

        match self.badge {
            Badge::None => {}
            Badge::Dot => {
                let center = response.rect.right_top() + egui::vec2(-3.0, 3.0);
                let painter = ui.painter();
                let bg = ui.visuals().panel_fill;
                painter.circle_filled(center, 4.5, bg);
                painter.circle_filled(center, 3.0, self.color);
            }
            Badge::Count(n) => {
                let center = response.rect.right_top() + egui::vec2(-5.0, 5.0);
                let painter = ui.painter();
                let bg = ui.visuals().panel_fill;
                painter.circle_filled(center, 6.5, bg);
                painter.circle_filled(center, 5.0, self.color);
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    if n > 99 { "99+".into() } else { n.to_string() },
                    egui::FontId::proportional(7.0),
                    egui::Color32::WHITE,
                );
            }
        }

        response
    }
}
