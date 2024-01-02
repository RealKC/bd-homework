use std::future::Future;

use adw::{glib, prelude::*};

pub struct ConfirmationDialogBuilder<F> {
    title: String,
    heading: String,
    body: String,
    confirm_text: String,
    action_is_destructive: bool,
    on_confirmation: Option<Box<dyn Fn() -> F>>,
}

impl<F> Default for ConfirmationDialogBuilder<F> {
    fn default() -> Self {
        Self {
            title: Default::default(),
            heading: Default::default(),
            body: Default::default(),
            confirm_text: Default::default(),
            action_is_destructive: Default::default(),
            on_confirmation: Default::default(),
        }
    }
}

impl<Fut> ConfirmationDialogBuilder<Fut>
where
    Fut: Future<Output = ()> + 'static,
{
    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn heading(mut self, heading: impl ToString) -> Self {
        self.heading = heading.to_string();
        self
    }

    pub fn body(mut self, body: impl ToString) -> Self {
        self.body = body.to_string();
        self
    }

    pub fn confirm_text(mut self, confirm_text: impl ToString) -> Self {
        self.confirm_text = confirm_text.to_string();
        self
    }

    pub fn action_is_destructive(mut self, b: bool) -> Self {
        self.action_is_destructive = b;
        self
    }

    pub fn on_confirmation<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Fut + 'static,
    {
        self.on_confirmation = Some(Box::new(f));
        self
    }

    pub fn build(self) -> adw::MessageDialog {
        const CANCEL: &str = "cancel";
        const CONFIRM: &str = "confirm";

        let dialog = adw::MessageDialog::builder()
            .title(self.title)
            .heading(self.heading)
            .body(self.body)
            .build();

        dialog.add_response(CANCEL, "Anulează");
        dialog.set_default_response(Some(CANCEL));
        dialog.set_close_response(CANCEL);

        dialog.add_response(CONFIRM, &self.confirm_text);
        if self.action_is_destructive {
            dialog.set_response_appearance(CONFIRM, adw::ResponseAppearance::Destructive);
        }

        dialog.connect_response(None, {
            move |_, response| {
                if response == CONFIRM {
                    let future = self.on_confirmation.as_ref().unwrap();
                    glib::MainContext::default().spawn_local((future)());
                }
            }
        });

        dialog
    }
}
