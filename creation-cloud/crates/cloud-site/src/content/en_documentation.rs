//! 提供英文文档入口、实操指南与安全参考内容。

use crate::{
    DocumentationContent, DocumentationGroup, DocumentationItem, DocumentationLink,
    DocumentationNotice, DocumentationScreenshot, DocumentationSection, PageContent, PageId,
};

use super::en::{action, page};

pub(super) fn page_content() -> PageContent {
    page(
        PageId::Documentation,
        "C-SSH Docs | Connect and Operate",
        "C-SSH 0.9.4 documentation for CSV import, a skippable guide, agent deployment, persistent terminals, monitoring, files and automatic Skills.",
        "DOCS / TASK GUIDES",
        "Start with your first C-SSH host",
        "Follow the C-SSH 0.9.4 task index: preview CSV imports, skip the lightweight guide if preferred, and use terminals, monitoring and AI when needed.",
    )
    .with_actions(vec![
        action("Add your first host", "#add-host", "button button-primary"),
        action("Review security boundaries", "/security", "button button-secondary"),
    ])
    .with_documentation_page(documentation())
}

fn documentation() -> DocumentationContent {
    DocumentationContent {
        index_label: "Documentation",
        mobile_index_label: "Open documentation index",
        search_label: "Filter titles on this page",
        search_placeholder: "Try host, terminal, or monitoring",
        search_help: "This filters titles already loaded on this page. It does not search article text or external sites.",
        search_empty: "No title on this page matches.",
        status: DocumentationNotice::new(
            "BEFORE YOU START",
            "Confirm the host identity and connection mode",
            "Agent mode provides the full integrated workflow; SSH mode provides native connection capabilities. Verify the host key on first connect and stop on unexpected changes.",
        ),
        groups: groups(),
        tutorials: super::en_tutorials::content(),
        sections: sections(),
        screenshot: DocumentationScreenshot {
            code: "PRODUCT VIEW / REDACTED DEMO",
            title: "A standard PTY and a persistent terminal are separate paths",
            lead: "The image shows a direct standard PTY. Only persistent-terminal mode uses the client, agent, and tmux to provide a reconnectable session.",
            src: "/static/img/product-terminal.png",
            alt: "Redacted C-SSH demo terminal showing an example server and a standard SSH PTY",
            caption: "Redacted demo with an RFC 5737 example address. It explains the UI path only and is not persistent-session or no-mock evidence.",
            width: 1650,
            height: 1080,
        },
        final_code: "NEXT / KEEP EVIDENCE",
        final_title: "Preserve the workspace before reporting a problem",
        final_body: "Never silently accept a changed host key, end an unauthorized remote session, or submit real addresses, passwords, private keys, tokens, or complete sensitive logs in feedback.",
    }
}

fn groups() -> Vec<DocumentationGroup> {
    vec![
        group(
            "Quick start",
            vec![link("getting-started", "01", "Before you connect")],
        ),
        group(
            "Hands-on guides",
            vec![
                link("add-host", "02", "Add your first host"),
                link("deploy-agent", "03", "Deploy or repair the agent"),
                link(
                    "persistent-terminal",
                    "04",
                    "Create a reconnectable terminal",
                ),
                link("monitoring", "05", "Enable monitoring and history"),
                link("files", "06", "Browse and transfer files"),
                link("ai-assistant", "07", "Configure and run the AI assistant"),
            ],
        ),
        group(
            "Reference and safety",
            vec![
                link("port-forwarding", "08", "Local SSH forwarding"),
                link("cloud-security", "09", "Cloud and the data boundary"),
                link("troubleshooting", "10", "Safe stop conditions"),
            ],
        ),
    ]
}

fn sections() -> Vec<DocumentationSection> {
    vec![
        section(
            "getting-started",
            "01 / QUICK START",
            "Before you connect",
            "Open the installed client and confirm the connection mode, host identity, and data boundary.",
            vec![
                text(
                    "0.9.4",
                    "CSV, guide, Skills and diagnostics",
                    "Preview CSV imports before committing and skip the lightweight guide if preferred. AI can learn and match Skills automatically. Diagnostics use one record per line; Frost White buttons are clearer on Windows, Mac and Linux.",
                    false,
                ),
                text(
                    "MODE",
                    "Choose Agent or SSH mode",
                    "Use Agent mode for persistent terminals, monitoring, and structured operations. Choose SSH mode when you only need a native connection or the target should not run the agent.",
                    true,
                ),
                text(
                    "HOST KEY",
                    "Verify the host identity",
                    "Verify the host-key fingerprint on first connect. If a known host key changes unexpectedly, stop instead of accepting it silently.",
                    true,
                ),
                text(
                    "DATA",
                    "Confirm the Cloud data boundary",
                    "The SSH data plane always connects directly from the client to the server. Creation Cloud stores accounts, devices, and allowed sync data only; it never proxies terminals or remote commands.",
                    false,
                ),
            ],
        ),
        section(
            "port-forwarding",
            "08 / PORT FORWARDING",
            "Use native SSH local forwarding",
            "Port forwarding is an intentional pure-SSH exception and does not depend on the agent.",
            vec![command(
                "LOOPBACK",
                "Bind to loopback by default",
                "Map a server-reachable target to 127.0.0.1 on the client. Change the listen address only when you understand the exposure.",
                "127.0.0.1:<local-port> -> <remote-host>:<remote-port>",
                true,
            )],
        ),
        section(
            "cloud-security",
            "09 / CLOUD & SECURITY",
            "A Cloud account is optional and SSH remains the data plane",
            "The Creation Cloud production control plane is deployed. A Cloud account is still optional for local SSH workflows.",
            vec![
                text(
                    "OPTIONAL",
                    "Manage local hosts without a Cloud account",
                    "SSH connections, standard terminals and local workflows do not require Cloud sign-in. Cloud provides accounts, devices, the model catalog and explicit encrypted sync of hosts, AI accounts and proxy profiles.",
                    false,
                ),
                text(
                    "BOUNDARY",
                    "Cloud never proxies the SSH data plane",
                    "Hosts, AI accounts and proxy profiles sync manually as client-encrypted opaque ciphertext. Data-protection passwords and keys never upload; AI conversations and long-term memory are excluded from Cloud sync.",
                    true,
                ),
            ],
        ),
        section(
            "troubleshooting",
            "10 / TROUBLESHOOTING",
            "Stop safely when something is wrong",
            "Protect identity and remote work first, then diagnose network, architecture, resources, and permissions.",
            vec![
                text(
                    "HOST KEY",
                    "The host key changed",
                    "Stop connecting and verify the new fingerprint and reason through a trusted channel. Do not delete known_hosts entries to skip confirmation.",
                    true,
                ),
                text(
                    "DEPLOY",
                    "Architecture or paired resources are missing",
                    "Run real uname -m detection again; the last architecture in SQLite is only a record. Do not upload another architecture or send both sets.",
                    true,
                ),
                text(
                    "SESSION",
                    "The session did not return",
                    "Confirm you used a persistent terminal rather than a standard PTY, then inspect agent and tmux ownership. Do not kill sessions or remove unknown sockets without authorization.",
                    true,
                ),
            ],
        ),
    ]
}

fn group(title: &'static str, links: Vec<DocumentationLink>) -> DocumentationGroup {
    DocumentationGroup::new(title, links)
}

const fn link(anchor: &'static str, code: &'static str, title: &'static str) -> DocumentationLink {
    DocumentationLink::new(anchor, code, title)
}

fn section(
    anchor: &'static str,
    code: &'static str,
    title: &'static str,
    lead: &'static str,
    items: Vec<DocumentationItem>,
) -> DocumentationSection {
    DocumentationSection::new(anchor, code, title, lead, items)
}

const fn text(
    badge: &'static str,
    title: &'static str,
    body: &'static str,
    caution: bool,
) -> DocumentationItem {
    DocumentationItem::text(badge, title, body, caution)
}

const fn command(
    badge: &'static str,
    title: &'static str,
    body: &'static str,
    value: &'static str,
    caution: bool,
) -> DocumentationItem {
    DocumentationItem::command(badge, title, body, value, caution)
}
