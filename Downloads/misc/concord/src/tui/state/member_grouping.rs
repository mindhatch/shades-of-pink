use std::collections::HashMap;

use crate::discord::ids::{
    Id,
    marker::{RoleMarker, UserMarker},
};

use crate::discord::{ChannelRecipientState, ChannelState, GuildMemberState, RoleState};

use super::presentation::{
    is_direct_message_channel, is_online_status, sort_member_entries, sort_recipient_entries,
    sorted_hoisted_roles,
};

#[derive(Debug)]
pub struct MemberGroup<'a> {
    pub label: String,
    pub color: Option<u32>,
    pub entries: Vec<MemberEntry<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub enum MemberEntry<'a> {
    Guild(&'a GuildMemberState),
    Recipient(&'a ChannelRecipientState),
}

impl MemberEntry<'_> {
    pub fn user_id(self) -> Id<UserMarker> {
        match self {
            Self::Guild(member) => member.user_id,
            Self::Recipient(recipient) => recipient.user_id,
        }
    }

    pub fn display_name(self) -> String {
        match self {
            Self::Guild(member) => member.display_name.clone(),
            Self::Recipient(recipient) => recipient.display_name.clone(),
        }
    }

    /// Discord login handle (username), distinct from `display_name` which
    /// already prefers the per-server alias / global display name.
    pub fn username(self) -> Option<String> {
        match self {
            Self::Guild(member) => member.username.clone(),
            Self::Recipient(recipient) => recipient.username.clone(),
        }
    }

    pub fn has_fallback_identity(self) -> bool {
        match self {
            Self::Guild(member) => member.username.is_none() && member.display_name == "unknown",
            Self::Recipient(recipient) => {
                recipient.username.is_none() && recipient.display_name == "unknown"
            }
        }
    }

    pub fn is_bot(self) -> bool {
        match self {
            Self::Guild(member) => member.is_bot,
            Self::Recipient(recipient) => recipient.is_bot,
        }
    }

    pub fn status(self) -> crate::discord::PresenceStatus {
        match self {
            Self::Guild(member) => member.status,
            Self::Recipient(recipient) => recipient.status,
        }
    }
}

pub(super) fn guild_member_groups<'a>(
    members: Vec<&'a GuildMemberState>,
    roles: Vec<&'a RoleState>,
) -> Vec<MemberGroup<'a>> {
    let hoisted_roles = sorted_hoisted_roles(&roles);
    let hoisted_role_ranks: HashMap<Id<RoleMarker>, usize> = hoisted_roles
        .iter()
        .enumerate()
        .map(|(rank, role)| (role.id, rank))
        .collect();
    let mut role_entries: Vec<Vec<&GuildMemberState>> = vec![Vec::new(); hoisted_roles.len()];
    let mut groups: Vec<MemberGroup<'a>> = Vec::new();

    let mut online_unroled = Vec::new();
    let mut offline = Vec::new();

    for member in members {
        if let Some(rank) = primary_hoisted_role_rank(member, &hoisted_role_ranks) {
            role_entries[rank].push(member);
        } else if is_online_status(member.status) {
            online_unroled.push(member);
        } else {
            offline.push(member);
        }
    }

    for (role, mut entries) in hoisted_roles.into_iter().zip(role_entries) {
        if entries.is_empty() {
            continue;
        }
        sort_member_entries(&mut entries);
        groups.push(MemberGroup {
            label: role.name.clone(),
            color: role.color,
            entries: entries.into_iter().map(MemberEntry::Guild).collect(),
        });
    }

    if !online_unroled.is_empty() {
        sort_member_entries(&mut online_unroled);
        groups.push(MemberGroup {
            label: "Online".to_owned(),
            color: None,
            entries: online_unroled.into_iter().map(MemberEntry::Guild).collect(),
        });
    }

    if !offline.is_empty() {
        sort_member_entries(&mut offline);
        groups.push(MemberGroup {
            label: "Offline".to_owned(),
            color: None,
            entries: offline.into_iter().map(MemberEntry::Guild).collect(),
        });
    }

    groups
}

fn primary_hoisted_role_rank(
    member: &GuildMemberState,
    hoisted_role_ranks: &HashMap<Id<RoleMarker>, usize>,
) -> Option<usize> {
    member
        .role_ids
        .iter()
        .filter_map(|role_id| hoisted_role_ranks.get(role_id).copied())
        .min()
}

pub(super) fn channel_recipient_group(channel: &ChannelState) -> Vec<MemberGroup<'_>> {
    if !is_direct_message_channel(channel) || channel.recipients.is_empty() {
        return Vec::new();
    }

    let mut recipients: Vec<&ChannelRecipientState> = channel.recipients.iter().collect();
    sort_recipient_entries(&mut recipients);
    vec![MemberGroup {
        label: "Members".to_owned(),
        color: None,
        entries: recipients.into_iter().map(MemberEntry::Recipient).collect(),
    }]
}

pub(super) fn flatten_member_groups(groups: Vec<MemberGroup<'_>>) -> Vec<MemberEntry<'_>> {
    groups.into_iter().flat_map(|group| group.entries).collect()
}
