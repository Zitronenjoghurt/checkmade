use crate::ui::icons;
use egui::{Ui, Widget};
use std::fmt::Display;

pub struct GenericSelect<'a, T, V>
where
    T: PartialEq + Copy,
    V: IntoIterator<Item = T>,
{
    value: &'a mut T,
    variants: V,
    label: Option<&'a str>,
    id: &'a str,
    default_value: Option<T>,
    filter: Option<Box<dyn Fn(T) -> bool + 'a>>,
    fmt: Box<dyn Fn(T) -> String + 'a>,
}

impl<'a, T, V> GenericSelect<'a, T, V>
where
    T: PartialEq + Copy,
    V: IntoIterator<Item = T>,
{
    pub fn new(value: &'a mut T, variants: V, id: &'a str, fmt: impl Fn(T) -> String + 'a) -> Self {
        Self {
            value,
            variants,
            label: None,
            id,
            default_value: None,
            filter: None,
            fmt: Box::new(fmt),
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn default_value(mut self, default_value: T) -> Self {
        self.default_value = Some(default_value);
        self
    }

    pub fn filter(mut self, predicate: impl Fn(T) -> bool + 'a) -> Self {
        self.filter = Some(Box::new(predicate));
        self
    }
}

impl<'a, T, V> GenericSelect<'a, T, V>
where
    T: PartialEq + Copy + Display,
    V: IntoIterator<Item = T>,
{
    pub fn new_display(value: &'a mut T, variants: V, id: &'a str) -> Self {
        Self::new(value, variants, id, |v| v.to_string())
    }
}

impl<'a, T> GenericSelect<'a, T, <T as strum::IntoEnumIterator>::Iterator>
where
    T: strum::IntoEnumIterator + PartialEq + Copy + Display,
{
    pub fn from_enum(value: &'a mut T, id: &'a str) -> Self {
        Self::new_display(value, T::iter(), id)
    }
}

impl<'a, T> GenericSelect<'a, T, <T as strum::IntoEnumIterator>::Iterator>
where
    T: strum::IntoEnumIterator + PartialEq + Copy,
{
    pub fn from_enum_with(value: &'a mut T, id: &'a str, fmt: impl Fn(T) -> String + 'a) -> Self {
        Self::new(value, T::iter(), id, fmt)
    }
}

impl<'a, T> GenericSelect<'a, Option<T>, Vec<Option<T>>>
where
    T: PartialEq + Copy,
{
    pub fn new_optional(
        value: &'a mut Option<T>,
        variants: impl IntoIterator<Item = T>,
        id: &'a str,
        fmt: impl Fn(T) -> String + 'a,
    ) -> Self {
        let options: Vec<Option<T>> = std::iter::once(None)
            .chain(variants.into_iter().map(Some))
            .collect();

        Self::new(value, options, id, move |v| match v {
            Some(inner) => fmt(inner),
            None => "—".to_string(),
        })
    }
}

impl<'a, T> GenericSelect<'a, Option<T>, Vec<Option<T>>>
where
    T: PartialEq + Copy + Display,
{
    pub fn new_optional_display(
        value: &'a mut Option<T>,
        variants: impl IntoIterator<Item = T>,
        id: &'a str,
    ) -> Self {
        Self::new_optional(value, variants, id, |v| v.to_string())
    }
}

impl<'a, T> GenericSelect<'a, Option<T>, Vec<Option<T>>>
where
    T: strum::IntoEnumIterator + PartialEq + Copy + Display,
{
    pub fn from_enum_optional(value: &'a mut Option<T>, id: &'a str) -> Self {
        Self::new_optional_display(value, T::iter(), id)
    }
}

impl<'a, T> GenericSelect<'a, Option<T>, Vec<Option<T>>>
where
    T: strum::IntoEnumIterator + PartialEq + Copy,
{
    pub fn from_enum_optional_with(
        value: &'a mut Option<T>,
        id: &'a str,
        fmt: impl Fn(T) -> String + 'a,
    ) -> Self {
        Self::new_optional(value, T::iter(), id, fmt)
    }
}

impl<T, V> Widget for GenericSelect<'_, T, V>
where
    T: PartialEq + Copy,
    V: IntoIterator<Item = T>,
{
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let old_value = *self.value;

        ui.horizontal(|ui| {
            let mut response = egui::ComboBox::new(self.id, self.label.unwrap_or_default())
                .selected_text((self.fmt)(*self.value))
                .show_ui(ui, |ui| {
                    for variant in self.variants {
                        if let Some(ref filter_fn) = self.filter
                            && !filter_fn(variant)
                        {
                            continue;
                        }
                        ui.selectable_value(self.value, variant, (self.fmt)(variant));
                    }
                })
                .response;

            if let Some(default_value) = self.default_value {
                let is_default = self.value == &default_value;
                if ui
                    .add_enabled(
                        !is_default,
                        egui::Button::new(icons::ARROW_COUNTER_CLOCKWISE).small(),
                    )
                    .on_hover_text(format!("Reset to {}", (self.fmt)(default_value)))
                    .clicked()
                {
                    *self.value = default_value;
                }
            }

            if *self.value != old_value {
                response.mark_changed();
            }

            response
        })
        .inner
    }
}
