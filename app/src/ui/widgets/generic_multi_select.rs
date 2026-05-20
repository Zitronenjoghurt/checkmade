use egui::{Ui, Widget};
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

pub struct GenericMultiSelect<'a, T, V>
where
    T: Eq + Hash + Copy + Display,
    V: IntoIterator<Item = T>,
{
    selected: &'a mut HashSet<T>,
    variants: V,
    label: Option<&'a str>,
    id: &'a str,
}

impl<'a, T, V> GenericMultiSelect<'a, T, V>
where
    T: Eq + Hash + Copy + Display,
    V: IntoIterator<Item = T>,
{
    pub fn new(selected: &'a mut HashSet<T>, variants: V, id: &'a str) -> Self {
        Self {
            selected,
            variants,
            label: None,
            id,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl<'a, T> GenericMultiSelect<'a, T, <T as strum::IntoEnumIterator>::Iterator>
where
    T: strum::IntoEnumIterator + Eq + Hash + Copy + Display,
{
    pub fn from_enum(selected: &'a mut HashSet<T>, id: &'a str) -> Self {
        Self::new(selected, T::iter(), id)
    }
}

impl<T, V> Widget for GenericMultiSelect<'_, T, V>
where
    T: Eq + Hash + Copy + Display,
    V: IntoIterator<Item = T>,
{
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let count = self.selected.len();
        let summary = if count == 0 {
            "All".to_string()
        } else {
            format!("{} selected", count)
        };

        let mut changed = false;

        let mut response = egui::ComboBox::new(self.id, self.label.unwrap_or_default())
            .selected_text(summary)
            .show_ui(ui, |ui| {
                for variant in self.variants {
                    let mut is_selected = self.selected.contains(&variant);
                    if ui.checkbox(&mut is_selected, variant.to_string()).changed() {
                        if is_selected {
                            self.selected.insert(variant);
                        } else {
                            self.selected.remove(&variant);
                        }
                        changed = true;
                    }
                }
            })
            .response;

        if changed {
            response.mark_changed();
        }

        response
    }
}
