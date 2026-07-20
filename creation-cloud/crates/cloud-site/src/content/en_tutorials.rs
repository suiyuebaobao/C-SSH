//! 提供英文教程总览，覆盖六条可执行的核心产品路径。

use crate::{PageContent, PageId, Tutorial, TutorialPageContent, TutorialStep};

use super::en::{action, page};

macro_rules! tutorial {
    ($anchor:expr, $sequence:expr, $title:expr, $summary:expr, $prerequisites:expr, $steps:expr, $verification:expr, $boundary:expr $(,)?) => {
        Tutorial::new($anchor, $sequence, $title, $summary)
            .with_procedure($prerequisites, $steps)
            .with_outcome($verification, $boundary)
    };
}

pub(super) fn page_content() -> PageContent {
    page(
        PageId::Tutorials,
        "SSH client and agent tutorials | Creation-SSH",
        "Follow verifiable steps to add a host, deploy the agent, use persistent terminals, monitor, manage files, and configure AI.",
        "Start with your first host",
        "Six tutorials for the core Creation-SSH workflow",
        "Each tutorial provides prerequisites, concrete actions, completion checks, and the agent boundary so you can prove the path works.",
    )
    .with_actions(vec![
        action(
            "Read the getting-started guide",
            "/docs/getting-started",
            "button button-secondary",
        ),
        action("Download the client", "/downloads", "button button-primary"),
    ])
    .with_tutorial_page(TutorialPageContent::new(
        "Tutorial index",
        "Before you start",
        "Steps",
        "Verify completion",
        "Agent boundary",
        vec![
            tutorial!(
                "add-host",
                "01",
                "Add your first host",
                "Save an authenticatable Linux host and establish first-use host-key trust.",
                vec![
                    "Have the SSH address, port, login user, and a password for the initial connection.",
                    "Be able to identify the server host-key fingerprint instead of trusting an unknown host blindly.",
                ],
                vec![
                    step(
                        "Step 1",
                        "Open Hosts",
                        "Open host management, choose Add host, and give the server a recognizable name.",
                    ),
                    step(
                        "Step 2",
                        "Enter SSH details",
                        "Enter the SSH target, port, and login user. Treat any display address as separate from the SSH target.",
                    ),
                    step(
                        "Step 3",
                        "Save and connect",
                        "Save the host and enter the password when prompted. It is used for authentication and encrypted local storage, not sent to Creation Cloud.",
                    ),
                    step(
                        "Step 4",
                        "Review the host key",
                        "Read the fingerprint before trusting it. Cancel and investigate if it differs from the value supplied by the administrator.",
                    ),
                    step(
                        "Step 5",
                        "Confirm the host record",
                        "Return to the host list and confirm the name, SSH target, and observed connection result before deploying the agent.",
                    ),
                ],
                "The host can be selected again and authenticated. Its host key is recorded, and any later change requires another explicit decision.",
                "Host creation, authentication, and host-key checks do not need the agent. Monitoring, persistent terminals, and structured management do.",
            ),
            tutorial!(
                "deploy-agent",
                "02",
                "Deploy or repair the agent",
                "Let the client detect server architecture, upload only the matching agent and static tmux pair, and complete the protocol handshake.",
                vec![
                    "The host must be an SSH-accessible Linux system with supported x86_64 or aarch64 resources.",
                    "The login user needs permission to write its home directory and establish a supported service lifecycle.",
                ],
                vec![
                    step(
                        "Step 1",
                        "Open maintenance",
                        "Open the target host menu and choose Install agent or Update / repair agent.",
                    ),
                    step(
                        "Step 2",
                        "Start architecture detection",
                        "After confirmation, the client runs a read-only uname -m over authenticated SSH. Do not select an architecture manually.",
                    ),
                    step(
                        "Step 3",
                        "Watch deployment progress",
                        "Wait for the matching agent and tmux upload, length and SHA256 validation, atomic installation, and service readiness.",
                    ),
                    step(
                        "Step 4",
                        "Wait for strict handshake",
                        "The client then reaches the local Unix socket and verifies both protocol and agent versions.",
                    ),
                    step(
                        "Step 5",
                        "Handle visible failures",
                        "Fix the reported unsupported architecture, missing resource, permission, or readiness problem and rerun the operation without bypassing its gate.",
                    ),
                ],
                "The maintenance view reports READY and an agent version. If collection is enabled, a real MetricsSnapshot is fetched instead of merely clearing an error flag.",
                "Deployment itself uses an SSH bootstrap. After READY, persistent terminal, monitoring, files, AI, and system capabilities use the agent.",
            ),
            tutorial!(
                "persistent-terminal",
                "03",
                "Create a reconnectable terminal",
                "Create a server-hosted tmux window and verify that its task survives a client disconnect.",
                vec![
                    "The host agent must be READY and the matching bundled static tmux must have been installed.",
                    "Prepare a harmless command that emits observable output for several seconds.",
                ],
                vec![
                    step(
                        "Step 1",
                        "Select persistent mode",
                        "Open Terminal, select the host, and switch the mode to Persistent terminal.",
                    ),
                    step(
                        "Step 2",
                        "Create a window",
                        "Create a clearly named tmux window or attach to an existing window from the list.",
                    ),
                    step(
                        "Step 3",
                        "Run an observation task",
                        "Enter the harmless continuous-output command and confirm live output and resize behavior.",
                    ),
                    step(
                        "Step 4",
                        "Disconnect and return",
                        "Disconnect or leave the page without choosing Close window, then reopen Terminal and attach to the same window.",
                    ),
                    step(
                        "Step 5",
                        "Check restored state",
                        "Confirm that the previous screen snapshot appears first and new output from the same task continues afterward.",
                    ),
                ],
                "The window ID and name are unchanged, the task continued while disconnected, and both restored screen content and live output are continuous.",
                "Persistent mode requires the agent and tmux. Without the agent, use an ordinary terminal, which cannot recover after closure or disconnect.",
            ),
            tutorial!(
                "monitoring",
                "04",
                "Enable monitoring and inspect history",
                "Move from one background snapshot to live detail, then use history and process data to assess the host.",
                vec![
                    "The host agent is READY and the login user can read standard system metrics.",
                    "Allow at least one full collection round. The default interval is six seconds after a round finishes.",
                ],
                vec![
                    step(
                        "Step 1",
                        "Enable collection",
                        "Enable monitoring for the host and adjust the interval and cross-host concurrency when needed.",
                    ),
                    step(
                        "Step 2",
                        "Inspect the host overview",
                        "Return to Hosts and confirm CPU, memory, disk, load, uptime, and state are read from the latest local cache.",
                    ),
                    step(
                        "Step 3",
                        "Open monitoring details",
                        "Open the host detail and observe the on-demand live subscription. This stream is separate from background short-request concurrency.",
                    ),
                    step(
                        "Step 4",
                        "Change the history range",
                        "Choose a range that already contains samples and inspect the high- or low-resolution history for continuity.",
                    ),
                    step(
                        "Step 5",
                        "Correlate top processes",
                        "Read top processes and system information, then relate resource changes to a process before acting.",
                    ),
                ],
                "The overview becomes fresh and its timestamp advances. Live detail, history, and top processes return structured data.",
                "Monitoring is agent-only and has no shell fallback. An unreachable agent should produce stale or failed state, never simulated online status.",
            ),
            tutorial!(
                "files",
                "05",
                "Browse and transfer files",
                "Create, edit, upload or download inside a safe remote test directory and use validation to prove transfer integrity.",
                vec![
                    "The host agent is READY and the login user can read and write the chosen test directory.",
                    "Use a dedicated temporary directory and non-sensitive files rather than production data or system configuration.",
                ],
                vec![
                    step(
                        "Step 1",
                        "Enter a safe directory",
                        "Open Files, choose the host, and use the breadcrumb trail to enter a writable temporary test directory.",
                    ),
                    step(
                        "Step 2",
                        "Create a test file",
                        "Create a text file, write recognizable content, save it, and reopen it to confirm the content.",
                    ),
                    step(
                        "Step 3",
                        "Run one transfer",
                        "Upload a non-sensitive file or download the file you just created while watching chunked transfer progress.",
                    ),
                    step(
                        "Step 4",
                        "Check the commit",
                        "Wait for checksum and commit success. For a larger directory, compress it on the server before download.",
                    ),
                    step(
                        "Step 5",
                        "Clean up your test data",
                        "After verifying file identity, remove only the test content created by this tutorial.",
                    ),
                ],
                "A refreshed listing shows the right name and size, reopened or downloaded content matches, and validation reports a successful commit.",
                "The full file workspace requires the agent. An ordinary terminal remains available, but Files does not replace its protocol with shell strings.",
            ),
            tutorial!(
                "ai-assistant",
                "06",
                "Configure and run the AI assistant",
                "Use your own model account to perform one read-only diagnosis against the selected host under controlled permissions.",
                vec![
                    "The host agent is READY and can complete a monitoring or read-only file request.",
                    "Have your own model service, API key, and model name. The key should remain in encrypted client storage.",
                ],
                vec![
                    step(
                        "Step 1",
                        "Configure the model",
                        "Choose the API type and provider in AI settings, enter the API key, and select an available model.",
                    ),
                    step(
                        "Step 2",
                        "Limit the run",
                        "Choose the target host, set permission to read-only, and review the workspace and tool-loop limit.",
                    ),
                    step(
                        "Step 3",
                        "Ask for a verifiable task",
                        "For example, request a summary of system information and resource state while explicitly forbidding file changes or destructive commands.",
                    ),
                    step(
                        "Step 4",
                        "Observe tool steps",
                        "Review every structured read-only tool call and reject any write or execution request that exceeds the selected permission.",
                    ),
                    step(
                        "Step 5",
                        "Cross-check and retain",
                        "Compare the result with Monitoring or System, then confirm the conversation and redacted audit record remain available.",
                    ),
                ],
                "The assistant returns a host-consistent read-only summary with no write steps, and the conversation and run can be restored after navigation.",
                "Remote tools require the agent and never expose SSH credentials to the model. Without the agent, remote work cannot fall back to arbitrary SSH execution.",
            ),
        ],
    ))
}

const fn step(label: &'static str, title: &'static str, instruction: &'static str) -> TutorialStep {
    TutorialStep::new(label, title, instruction)
}
