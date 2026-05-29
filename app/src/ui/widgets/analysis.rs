use crate::i18n::Translatable;
use crate::ui::icons;
use crate::ui::state::analysis::{format_score, score_to_cp, AnalysisState, EvalPoint};
use checkmade_core::lingo::Lingo::{Black, Depth, Score, Thinking, Turn, White};
use egui::{vec2, Color32, Rect, Response, Sense, Ui};
use egui_plot::{GridMark, Line, Plot, PlotPoints, Points};

pub struct AnalysisWidget<'a> {
    state: &'a mut AnalysisState,
}

impl<'a> AnalysisWidget<'a> {
    pub fn new(state: &'a mut AnalysisState) -> Self {
        Self { state }
    }
}

impl egui::Widget for AnalysisWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let mut enabled = self.state.is_enabled();
                if ui.checkbox(&mut enabled, "").changed() {
                    self.state.toggle(ui.ctx());
                }
                ui.add(egui::Slider::new(&mut self.state.depth, 1..=30).text(Depth.t()));
            });

            if let Some(eval) = &self.state.eval {
                ui.add_space(4.0);

                let is_black_turn = !self.state.viewed_ply().is_multiple_of(2);

                ui.horizontal(|ui| {
                    let text = format_score(&eval.score);
                    let cp = score_to_cp(&eval.score);
                    let color = if cp > 50.0 {
                        Color32::from_rgb(120, 190, 120)
                    } else if cp < -50.0 {
                        Color32::from_rgb(200, 100, 100)
                    } else {
                        ui.visuals().text_color()
                    };

                    ui.label(egui::RichText::new(text).size(22.0).strong().color(color));

                    ui.separator();

                    if let Some(best_move) = eval.best_move {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} {} {}",
                                best_move.0,
                                icons::ARROW_RIGHT,
                                best_move.1
                            ))
                            .size(14.0)
                            .weak(),
                        );
                    } else {
                        ui.spinner();
                        ui.label(egui::RichText::new(format!("{}...", Thinking.t())).weak());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.state.is_thinking() {
                            ui.spinner();
                        };
                        ui.label(
                            egui::RichText::new(format!("d{}", eval.depth))
                                .size(12.0)
                                .weak(),
                        );
                    });
                });

                if let Some(wdl) = &eval.wdl {
                    ui.add_space(2.0);
                    if is_black_turn {
                        draw_wdl_bar(ui, wdl.2, wdl.1, wdl.0);
                    } else {
                        draw_wdl_bar(ui, wdl.0, wdl.1, wdl.2);
                    }
                }

                let raw_cp = score_to_cp(&eval.score);
                let absolute_cp = if is_black_turn { -raw_cp } else { raw_cp };
                draw_cp_eval_bar(ui, absolute_cp);
            }

            if self.state.eval_history.len() >= 2 {
                ui.add_space(6.0);
                draw_eval_graph(ui, &self.state.eval_history);
            }
        })
        .response
    }
}

fn draw_wdl_bar(ui: &mut Ui, win: u32, draw: u32, loss: u32) {
    let total = (win + draw + loss).max(1) as f32;
    let available = ui.available_width();
    let height = 6.0;

    let (rect, _) = ui.allocate_exact_size(vec2(available, height), Sense::hover());

    let w_width = (win as f32 / total) * available;
    let d_width = (draw as f32 / total) * available;

    let white_color = Color32::from_rgb(220, 220, 215);
    let draw_color = Color32::from_rgb(140, 140, 140);
    let black_color = Color32::from_rgb(50, 50, 50);

    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 2.0, black_color);
    if w_width + d_width > 0.0 {
        let wd_rect = Rect::from_min_size(rect.min, vec2(w_width + d_width, height));
        painter.rect_filled(wd_rect, 2.0, draw_color);
    }
    if w_width > 0.0 {
        let w_rect = Rect::from_min_size(rect.min, vec2(w_width, height));
        painter.rect_filled(w_rect, 2.0, white_color);
    }
}

fn draw_eval_graph(ui: &mut Ui, history: &[Option<EvalPoint>]) {
    let max_x = history.len().saturating_sub(1) as f64;

    let raw_points: Vec<[f64; 2]> = history
        .iter()
        .enumerate()
        .filter_map(|(ply, opt_p)| {
            let p = opt_p.as_ref()?;
            Some([ply as f64, (p.cp / 100.0).clamp(-10.0, 10.0)])
        })
        .collect();

    Plot::new("eval_plot")
        .height(100.0)
        .show_axes([true, true])
        .show_grid(false)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .set_margin_fraction(vec2(0.0, 0.1))
        .default_y_bounds(-10.0, 10.0)
        .x_grid_spacer(|input| {
            let mut marks = vec![];
            let (min, max) = input.bounds;
            let step = 2.0;
            let mut current = (min / step).ceil() * step;
            while current <= max {
                marks.push(GridMark {
                    value: current,
                    step_size: step,
                });
                current += step;
            }
            marks
        })
        .x_axis_formatter(|grid_mark, _range| {
            let move_num = (grid_mark.value / 2.0).floor() as i32 + 1;
            format!("{}", move_num)
        })
        .y_axis_formatter(|grid_mark, _range| {
            if grid_mark.value > 0.0 {
                format!("+{:.1}", grid_mark.value)
            } else {
                format!("{:.1}", grid_mark.value)
            }
        })
        .label_formatter(|name, value| {
            if name == "Eval" {
                let move_num = (value.x / 2.0).floor() as i32 + 1;
                let sign = if value.y > 0.0 { "+" } else { "" };
                format!(
                    "{} {move_num}\n{}: {sign}{:.2}",
                    Turn.t(),
                    Score.t(),
                    value.y
                )
            } else {
                String::new()
            }
        })
        .show(ui, |plot_ui| {
            let zero: PlotPoints = vec![[0.0, 0.0], [max_x, 0.0]].into();
            plot_ui.line(
                Line::new("zero_line", zero)
                    .color(Color32::from_gray(80))
                    .width(1.0),
            );

            plot_ui.text(
                egui_plot::Text::new("white", egui_plot::PlotPoint::new(0.0, 4.0), White.t())
                    .color(Color32::from_gray(150))
                    .anchor(egui::Align2::LEFT_CENTER),
            );
            plot_ui.text(
                egui_plot::Text::new("black", egui_plot::PlotPoint::new(0.0, -4.0), Black.t())
                    .color(Color32::from_gray(100))
                    .anchor(egui::Align2::LEFT_CENTER),
            );

            plot_ui.line(
                Line::new("eval_line", PlotPoints::new(raw_points.clone()))
                    .name("Eval")
                    .color(Color32::from_rgb(100, 160, 230))
                    .width(2.0)
                    .fill(0.0),
            );

            plot_ui.points(
                Points::new("eval_pts", PlotPoints::new(raw_points))
                    .name("Eval")
                    .radius(2.0)
                    .color(Color32::from_rgb(150, 200, 255)),
            );
        });
}

fn draw_cp_eval_bar(ui: &mut Ui, cp: f64) {
    let available = ui.available_width();
    let height = 6.0;

    let (rect, _) = ui.allocate_exact_size(vec2(available, height), Sense::hover());

    let white_ratio = 1.0 / (1.0 + 10_f64.powf(-cp / 400.0));

    let w_width = (white_ratio as f32) * available;

    let white_color = Color32::from_rgb(220, 220, 215);
    let black_color = Color32::from_rgb(50, 50, 50);

    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 2.0, black_color);
    if w_width > 0.0 {
        let w_rect = Rect::from_min_size(rect.min, vec2(w_width, height));
        painter.rect_filled(w_rect, 2.0, white_color);
    }
}
