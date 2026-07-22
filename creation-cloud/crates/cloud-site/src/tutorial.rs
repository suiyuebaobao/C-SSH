//! 定义文档中心实操指南的前置条件、操作步骤、验证与 agent 边界模型。

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialStep {
    pub label: &'static str,
    pub title: &'static str,
    pub instruction: &'static str,
}

impl TutorialStep {
    pub(crate) const fn new(
        label: &'static str,
        title: &'static str,
        instruction: &'static str,
    ) -> Self {
        Self {
            label,
            title,
            instruction,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tutorial {
    pub anchor: &'static str,
    pub sequence: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub prerequisites: Vec<&'static str>,
    pub steps: Vec<TutorialStep>,
    pub verification: &'static str,
    pub agent_boundary: &'static str,
}

impl Tutorial {
    pub(crate) fn new(
        anchor: &'static str,
        sequence: &'static str,
        title: &'static str,
        summary: &'static str,
    ) -> Self {
        Self {
            anchor,
            sequence,
            title,
            summary,
            prerequisites: Vec::new(),
            steps: Vec::new(),
            verification: "",
            agent_boundary: "",
        }
    }

    #[must_use]
    pub(crate) fn with_procedure(
        mut self,
        prerequisites: Vec<&'static str>,
        steps: Vec<TutorialStep>,
    ) -> Self {
        self.prerequisites = prerequisites;
        self.steps = steps;
        self
    }

    #[must_use]
    pub(crate) fn with_outcome(
        mut self,
        verification: &'static str,
        agent_boundary: &'static str,
    ) -> Self {
        self.verification = verification;
        self.agent_boundary = agent_boundary;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialContent {
    pub index_label: &'static str,
    pub prerequisites_label: &'static str,
    pub steps_label: &'static str,
    pub verification_label: &'static str,
    pub agent_boundary_label: &'static str,
    pub items: Vec<Tutorial>,
}

impl TutorialContent {
    pub(crate) fn new(
        index_label: &'static str,
        prerequisites_label: &'static str,
        steps_label: &'static str,
        verification_label: &'static str,
        agent_boundary_label: &'static str,
        items: Vec<Tutorial>,
    ) -> Self {
        Self {
            index_label,
            prerequisites_label,
            steps_label,
            verification_label,
            agent_boundary_label,
            items,
        }
    }
}
