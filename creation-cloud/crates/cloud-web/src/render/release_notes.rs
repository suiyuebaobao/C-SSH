//! 将发布说明纯文本分成标题、段落和列表，保留模板自动转义边界。

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NotesBlock {
    Heading(String),
    Paragraph(String),
    Bullets(Vec<String>),
}

pub(super) fn parse(text: &str) -> Vec<NotesBlock> {
    let mut blocks = Vec::new();
    let mut paragraph = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            flush_paragraph(&mut blocks, &mut paragraph);
        } else if let Some(item) = bullet(line) {
            flush_paragraph(&mut blocks, &mut paragraph);
            if let Some(NotesBlock::Bullets(items)) = blocks.last_mut() {
                items.push(item.to_owned());
            } else {
                blocks.push(NotesBlock::Bullets(vec![item.to_owned()]));
            }
        } else if let Some(title) = heading(line) {
            flush_paragraph(&mut blocks, &mut paragraph);
            blocks.push(NotesBlock::Heading(title.to_owned()));
        } else {
            paragraph.push(line);
        }
    }
    flush_paragraph(&mut blocks, &mut paragraph);
    blocks
}

fn flush_paragraph(blocks: &mut Vec<NotesBlock>, lines: &mut Vec<&str>) {
    if !lines.is_empty() {
        blocks.push(NotesBlock::Paragraph(lines.join("\n")));
        lines.clear();
    }
}

fn bullet(line: &str) -> Option<&str> {
    ["- ", "* ", "• "]
        .into_iter()
        .find_map(|prefix| line.strip_prefix(prefix))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn heading(line: &str) -> Option<&str> {
    for prefix in ["### ", "## ", "# "] {
        if let Some(title) = line.strip_prefix(prefix).map(str::trim) {
            return (!title.is_empty()).then_some(title);
        }
    }
    // 已发布的中英文说明使用短行加冒号标记小标题，不能据此解释HTML。
    (line.chars().count() <= 80 && line.ends_with([':', '：']))
        .then(|| line.trim_end_matches([':', '：']).trim())
        .filter(|title| !title.is_empty())
}
