use crate::i18n::Translatable;
use checkmade_core::giga_chess::prelude::clock::ChessClockConfig;
use checkmade_core::giga_chess::prelude::config::TimeControl;
use checkmade_core::lingo::Lingo::{
    CustomTime, DailyMode, Days, Hours, Increment, MinuteShort, SecondShort, Time,
    TimeControl as TimeControlLingo, Unlimited,
};
use egui::{Response, Ui};

const PRESETS: &[(u64, u64); 8] = &[
    (1, 0),
    (1, 2),
    (3, 0),
    (3, 2),
    (5, 3),
    (10, 5),
    (15, 10),
    (90, 30),
];

pub struct TimeControlWidget<'a> {
    time_control: &'a mut TimeControl,
}

impl<'a> TimeControlWidget<'a> {
    pub fn new(time_control: &'a mut TimeControl) -> Self {
        Self { time_control }
    }
}

impl egui::Widget for TimeControlWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label(TimeControlLingo.t());
            ui.horizontal_wrapped(|ui| {
                if ui.button(Unlimited.t()).clicked() {
                    *self.time_control = TimeControl::Unlimited;
                }

                for &(base_min, inc_sec) in PRESETS {
                    let label = format!("{}+{}", base_min, inc_sec);
                    if ui.button(label).clicked() {
                        let ms = base_min * 60 * 1000;
                        let inc_ms = inc_sec * 1000;
                        *self.time_control = TimeControl::Clock(ChessClockConfig {
                            white_ms: ms,
                            black_ms: ms,
                            white_inc_ms: inc_ms,
                            black_inc_ms: inc_ms,
                        });
                    }
                }
            });

            let mut is_custom = matches!(self.time_control, TimeControl::Clock(_));

            ui.horizontal(|ui| {
                if ui.checkbox(&mut is_custom, CustomTime.t()).changed() {
                    if is_custom {
                        *self.time_control = TimeControl::Clock(ChessClockConfig {
                            white_ms: 10 * 60 * 1000,
                            black_ms: 10 * 60 * 1000,
                            white_inc_ms: 0,
                            black_inc_ms: 0,
                        });
                    } else {
                        *self.time_control = TimeControl::Unlimited;
                    }
                }
            });

            if let TimeControl::Clock(clock) = self.time_control {
                let daily_mode_id = ui.make_persistent_id("time_control_daily_mode");
                let mut is_daily = ui.data_mut(|d| d.get_temp(daily_mode_id).unwrap_or(false));

                ui.checkbox(&mut is_daily, DailyMode.t());
                ui.data_mut(|d| d.insert_temp(daily_mode_id, is_daily));

                ui.horizontal(|ui| {
                    if is_daily {
                        const MS_PER_HOUR: u64 = 60 * 60 * 1000;
                        const MS_PER_DAY: u64 = 24 * MS_PER_HOUR;

                        let mut base_days = (clock.white_ms / MS_PER_DAY) as u32;
                        let mut inc_hours = (clock.white_inc_ms / MS_PER_HOUR) as u32;

                        ui.label(format!("{} ({}):", Time.t(), Days.t()));
                        if ui
                            .add(
                                egui::DragValue::new(&mut base_days)
                                    .speed(0.1)
                                    .range(1..=14),
                            )
                            .changed()
                        {
                            let new_ms = (base_days as u64) * MS_PER_DAY;
                            clock.white_ms = new_ms;
                            clock.black_ms = new_ms;
                        }

                        ui.label(format!("{} ({}):", Increment.t(), Hours.t()));
                        if ui
                            .add(
                                egui::DragValue::new(&mut inc_hours)
                                    .speed(0.1)
                                    .range(0..=48),
                            )
                            .changed()
                        {
                            let new_inc_ms = (inc_hours as u64) * MS_PER_HOUR;
                            clock.white_inc_ms = new_inc_ms;
                            clock.black_inc_ms = new_inc_ms;
                        }
                    } else {
                        let mut base_mins = (clock.white_ms / 60_000) as u32;
                        let mut inc_secs = (clock.white_inc_ms / 1_000) as u32;

                        ui.label(format!("{} ({}):", Time.t(), MinuteShort.tl()));
                        if ui
                            .add(
                                egui::DragValue::new(&mut base_mins)
                                    .speed(1.0)
                                    .range(1..=500),
                            )
                            .changed()
                        {
                            let new_ms = (base_mins as u64) * 60_000;
                            clock.white_ms = new_ms;
                            clock.black_ms = new_ms;
                        }

                        ui.label(format!("{} ({}):", Increment.t(), SecondShort.tl()));
                        if ui
                            .add(
                                egui::DragValue::new(&mut inc_secs)
                                    .speed(1.0)
                                    .range(0..=600),
                            )
                            .changed()
                        {
                            let new_inc_ms = (inc_secs as u64) * 1_000;
                            clock.white_inc_ms = new_inc_ms;
                            clock.black_inc_ms = new_inc_ms;
                        }
                    }
                });
            }
        })
        .response
    }
}
