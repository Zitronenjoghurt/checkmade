pub struct ValidatedInput<'a> {
    value: &'a mut String,
    hint: &'a str,
    validator: fn(&str) -> bool,
    char_filter: Option<fn(char) -> Option<char>>,
    char_limit: Option<usize>,
    uppercase: bool,
    width: f32,
}

impl<'a> ValidatedInput<'a> {
    pub fn new(value: &'a mut String, validator: fn(&str) -> bool) -> Self {
        Self {
            value,
            hint: "",
            validator,
            char_filter: None,
            char_limit: None,
            uppercase: false,
            width: 150.0,
        }
    }

    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }

    pub fn char_filter(mut self, filter: fn(char) -> Option<char>) -> Self {
        self.char_filter = Some(filter);
        self
    }

    pub fn uppercase(mut self) -> Self {
        self.uppercase = true;
        self
    }
}

impl egui::Widget for ValidatedInput<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let is_empty = self.value.is_empty();
        let is_valid = (self.validator)(self.value);

        let prev_visuals = ui.visuals().clone();
        if !is_empty && !is_valid {
            ui.visuals_mut().selection.stroke.color = ui.visuals().error_fg_color;
            ui.visuals_mut().widgets.inactive.bg_stroke.color =
                ui.visuals().error_fg_color.linear_multiply(0.4);
        }

        let mut edit = egui::TextEdit::singleline(self.value)
            .hint_text(self.hint)
            .desired_width(self.width);

        if let Some(limit) = self.char_limit {
            edit = edit.char_limit(limit);
        }

        let response = ui.add(edit);

        if self.uppercase {
            *self.value = self.value.to_uppercase();
        }

        if let Some(filter) = self.char_filter {
            *self.value = self.value.chars().filter_map(filter).collect();
        }

        *ui.visuals_mut() = prev_visuals;

        response
    }
}
