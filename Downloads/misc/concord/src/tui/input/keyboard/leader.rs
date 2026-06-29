use crossterm::event::KeyEvent;

use crate::discord::AppCommand;
use crate::tui::keybindings::{KeyMapLookup, LeaderActionMenuAction};
use crate::tui::state::{ActiveModalPopupKind, DashboardState};

use super::execute_ui_action;

pub(super) fn handle_leader_key(state: &mut DashboardState, key: KeyEvent) -> Option<AppCommand> {
    if state.is_leader_action_mode() {
        return handle_leader_action_key(state, key);
    }

    if let Some(command) = handle_leader_keymap_key(state, key) {
        return command;
    }

    state.close_leader();

    None
}

fn handle_leader_keymap_key(
    state: &mut DashboardState,
    key: KeyEvent,
) -> Option<Option<AppCommand>> {
    let focus = state.focus();
    let lookup = state
        .key_bindings()
        .keymap_lookup_with_key(state.leader_keymap_prefix(), key);
    match lookup {
        Some(KeyMapLookup::Pending) => {
            let chord = state.key_bindings().keymap_chord_for_event(key);
            state.push_leader_keymap_key(chord);
            Some(None)
        }
        Some(KeyMapLookup::Action(action)) => {
            state.close_leader();
            Some(execute_ui_action(state, focus, action))
        }
        None if state.leader_keymap_prefix().len() > 1 => {
            state.close_leader();
            Some(None)
        }
        None => None,
    }
}

fn handle_leader_action_key(state: &mut DashboardState, key: KeyEvent) -> Option<AppCommand> {
    match state.key_bindings().leader_action_menu_action(key) {
        LeaderActionMenuAction::BackOrClose => {
            if state.is_active_modal_popup(ActiveModalPopupKind::MessageUrlPicker) {
                state.close_message_url_picker();
                return None;
            }
            if state.back_channel_leader_action() || state.back_guild_leader_action() {
                return None;
            }
            state.close_all_action_contexts();
            state.close_leader();
            None
        }
        LeaderActionMenuAction::Close => {
            state.close_all_action_contexts();
            state.close_leader();
            None
        }
        LeaderActionMenuAction::ActivateShortcut(shortcut) => {
            let activation = state.activate_active_action_shortcut(shortcut);
            if !activation.matched || !state.is_any_action_context_active() {
                state.close_all_action_contexts();
                state.close_leader();
            }
            activation.command
        }
        LeaderActionMenuAction::UnknownClose => {
            state.close_all_action_contexts();
            state.close_leader();
            None
        }
    }
}
