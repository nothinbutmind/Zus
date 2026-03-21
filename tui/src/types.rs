use std::io::Stdout;

use ratatui::{Terminal, backend::CrosstermBackend};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Focus {
    Actions,
    Fields,
}

#[derive(Clone, Copy, Debug)]
pub enum ActionKind {
    CheckAddress,
    CreateWallet,
    ImportWallet,
}

#[derive(Clone, Debug)]
pub struct FormField {
    pub key: &'static str,
    pub label: &'static str,
    pub hint: &'static str,
    pub value: String,
    pub sensitive: bool,
    pub required: bool,
}

#[derive(Clone, Debug)]
pub struct ActionForm {
    pub kind: ActionKind,
    pub label: &'static str,
    pub command_label: &'static str,
    pub description: &'static str,
    pub fields: Vec<FormField>,
}

pub struct App {
    pub forms: Vec<ActionForm>,
    pub selected_action: usize,
    pub selected_field: usize,
    pub focus: Focus,
    pub output: String,
    pub last_command: String,
    pub status: String,
}

pub struct CommandResult {
    pub command_preview: String,
    pub output: String,
    pub success: bool,
}

pub type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

impl App {
    pub fn new() -> Self {
        Self {
            forms: vec![
                ActionForm {
                    kind: ActionKind::CheckAddress,
                    label: "Check Address",
                    command_label: "cast wallet address",
                    description: "Paste a private key and derive the wallet address with Foundry.",
                    fields: vec![FormField {
                        key: "private_key",
                        label: "Private Key",
                        hint: "paste your private key here",
                        value: String::new(),
                        sensitive: true,
                        required: true,
                    }],
                },
                ActionForm {
                    kind: ActionKind::CreateWallet,
                    label: "Create New",
                    command_label: "cast wallet new",
                    description: "Generate a new wallet. Add password plus account name to save it as a Foundry keystore.",
                    fields: vec![
                        FormField {
                            key: "account_name",
                            label: "Account Name",
                            hint: "zus-main",
                            value: String::new(),
                            sensitive: false,
                            required: false,
                        },
                        FormField {
                            key: "keystore_dir",
                            label: "Keystore Dir",
                            hint: "~/.foundry/keystores",
                            value: String::new(),
                            sensitive: false,
                            required: false,
                        },
                        FormField {
                            key: "password",
                            label: "Password",
                            hint: "required if saving keystore",
                            value: String::new(),
                            sensitive: true,
                            required: false,
                        },
                        FormField {
                            key: "number",
                            label: "Number",
                            hint: "1",
                            value: "1".to_string(),
                            sensitive: false,
                            required: false,
                        },
                    ],
                },
                ActionForm {
                    kind: ActionKind::ImportWallet,
                    label: "Import Private Key",
                    command_label: "cast wallet import",
                    description: "Store an existing private key as an encrypted Foundry keystore entry.",
                    fields: vec![
                        FormField {
                            key: "account_name",
                            label: "Account Name",
                            hint: "my-imported-wallet",
                            value: String::new(),
                            sensitive: false,
                            required: true,
                        },
                        FormField {
                            key: "private_key",
                            label: "Private Key",
                            hint: "paste your private key here",
                            value: String::new(),
                            sensitive: true,
                            required: true,
                        },
                        FormField {
                            key: "password",
                            label: "Password",
                            hint: "keystore password",
                            value: String::new(),
                            sensitive: true,
                            required: true,
                        },
                        FormField {
                            key: "keystore_dir",
                            label: "Keystore Dir",
                            hint: "~/.foundry/keystores",
                            value: String::new(),
                            sensitive: false,
                            required: false,
                        },
                    ],
                },
            ],
            selected_action: 0,
            selected_field: 0,
            focus: Focus::Actions,
            output:
                "Yes, this is possible. Pick a flow on the left, fill the fields, then press Enter."
                    .to_string(),
            last_command: "cast wallet address --private-key <PRIVATE_KEY>".to_string(),
            status: "Ready".to_string(),
        }
    }

    pub fn current_form(&self) -> &ActionForm {
        &self.forms[self.selected_action]
    }

    pub fn current_form_mut(&mut self) -> &mut ActionForm {
        &mut self.forms[self.selected_action]
    }

    pub fn current_field(&self) -> Option<&FormField> {
        self.current_form().fields.get(self.selected_field)
    }

    pub fn current_field_mut(&mut self) -> Option<&mut FormField> {
        let index = self.selected_field;
        self.current_form_mut().fields.get_mut(index)
    }

    pub fn select_next_action(&mut self) {
        self.selected_action = (self.selected_action + 1) % self.forms.len();
        self.selected_field = 0;
    }

    pub fn select_prev_action(&mut self) {
        self.selected_action = if self.selected_action == 0 {
            self.forms.len() - 1
        } else {
            self.selected_action - 1
        };
        self.selected_field = 0;
    }

    pub fn select_next_field(&mut self) {
        if self.current_form().fields.is_empty() {
            return;
        }
        self.selected_field = (self.selected_field + 1) % self.current_form().fields.len();
    }

    pub fn select_prev_field(&mut self) {
        if self.current_form().fields.is_empty() {
            return;
        }
        self.selected_field = if self.selected_field == 0 {
            self.current_form().fields.len() - 1
        } else {
            self.selected_field - 1
        };
    }

    pub fn move_focus_left(&mut self) {
        self.focus = Focus::Actions;
    }

    pub fn move_focus_right(&mut self) {
        self.focus = Focus::Fields;
    }

    pub fn backspace(&mut self) {
        if let Some(field) = self.current_field_mut() {
            field.value.pop();
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        if let Some(field) = self.current_field_mut() {
            field.value.push(ch);
        }
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
        self.status = "Output cleared".to_string();
    }
}

impl ActionForm {
    pub fn value(&self, key: &str) -> &str {
        self.fields
            .iter()
            .find(|field| field.key == key)
            .map(|field| field.value.trim())
            .unwrap_or("")
    }
}
